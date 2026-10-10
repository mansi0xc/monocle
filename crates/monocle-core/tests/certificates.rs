use bitvec::{order::Lsb0, vec::BitVec};
use monocle_core::{certificate::*, *};
use serde_json::{Value, json};
use solana_signer_store::{Decoded, decode, encode_base2};

#[path = "../../../tools/certificate-fixtures/src/reference.rs"]
mod reference;

fn corpus() -> Value {
    serde_json::from_str(include_str!("../../../fixtures/certificates/corpus.json")).unwrap()
}
fn fixture(name: &str) -> (Value, Value) {
    let data = corpus();
    let case = data["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap();
    (
        serde_json::from_str(case["bundle_json"].as_str().unwrap()).unwrap(),
        serde_json::from_str(case["checkpoint_json"].as_str().unwrap()).unwrap(),
    )
}
fn request(bundle: &Value, digest: Option<ContentDigest>) -> Vec<u8> {
    serde_json::to_vec(&VerifyRequest {
        schema_version: 1,
        run_id: "test".into(),
        profile_id: certificate::PROFILE_ID.into(),
        bundle_json: serde_json::to_string(bundle).unwrap(),
        selected_checkpoint: digest,
    })
    .unwrap()
}
fn run(mut bundle: Value, checkpoint: Value) -> CertificateVerificationReport {
    let mut trust = TrustStore::new();
    let digest = trust
        .pin(
            parse_checkpoint(&serde_json::to_vec(&checkpoint).unwrap()).unwrap(),
            TrustBasis::SyntheticFixture,
        )
        .unwrap();
    bundle["checkpoint_reference"] = serde_json::to_value(&digest).unwrap();
    verify_certificates(&request(&bundle, Some(digest)), &trust)
}
fn stage(report: &CertificateVerificationReport, stage: Stage) -> &CheckResult {
    report.certificates[0]
        .checks
        .iter()
        .find(|c| c.stage == stage)
        .unwrap()
}

#[test]
fn signed_corpus_has_expected_conclusions_and_failure_stages() {
    let data = corpus();
    assert_eq!(data["cases"].as_array().unwrap().len(), 32);
    for case in data["cases"].as_array().unwrap() {
        let (bundle, checkpoint) = fixture(case["name"].as_str().unwrap());
        let report = run(bundle, checkpoint);
        assert_eq!(
            serde_json::to_value(report.conclusion).unwrap(),
            case["expected"]["conclusion"],
            "{}",
            case["name"]
        );
        assert!(
            report
                .checks
                .iter()
                .chain(report.certificates.iter().flat_map(|c| &c.checks))
                .any(|check| {
                    serde_json::to_value(check.stage).unwrap() == case["expected"]["stage"]
                        && serde_json::to_value(check.diagnostic.code).unwrap()
                            == case["expected"]["code"]
                }),
            "{}",
            case["name"]
        );
        assert_eq!(report.evidence_scope, "individual_certificates");
        assert_eq!(
            report.checks.last().unwrap().status,
            CheckStatus::NotChecked
        );
        if report.conclusion == Conclusion::Verified {
            assert!(
                report.certificates[0]
                    .checks
                    .iter()
                    .all(|c| c.status == CheckStatus::Pass)
            );
        }
    }
}

#[test]
fn under_quorum_cases_are_genuinely_signed_not_changed_bitmaps() {
    for name in [
        "notar_under_quorum",
        "fast_under_quorum",
        "genesis_under_quorum",
    ] {
        let (bundle, cp) = fixture(name);
        let report = run(bundle, cp);
        assert_eq!(stage(&report, Stage::Signature).status, CheckStatus::Pass);
        assert_eq!(
            stage(&report, Stage::Quorum).diagnostic.code,
            Code::InsufficientStake
        );
    }
}

#[test]
fn wire_adapter_matches_reduced_upstream_schema_and_fixed_byte_layout() {
    for (kind, cert_tag, vote_tag) in [
        ("notarization", 9, 1),
        ("finalization", 7, 2),
        ("fast_finalization", 8, 1),
        ("genesis", 12, 6),
    ] {
        for slot in [0, 42, u64::MAX] {
            let signature = [0x37; 192];
            let bitmap = vec![0, 5, 0, 3];
            let wire =
                reference::message(kind, slot, [0x42; 32], 0x1234, signature, bitmap.clone());
            let decoded = decode_certificate(&wire).unwrap();
            let reference: reference::Versioned = wincode::deserialize(&wire).unwrap();
            assert_eq!(wincode::serialize(&reference).unwrap(), wire);
            assert_eq!(&wire[..2], &[1, cert_tag]);
            assert_eq!(&wire[2..10], &slot.to_le_bytes());
            let length_offset = if kind == "finalization" { 202 } else { 234 };
            assert_eq!(&wire[length_offset..length_offset + 8], &4u64.to_le_bytes());
            assert_eq!(&wire[wire.len() - 2..], &[0x34, 0x12]);
            assert_eq!(decoded.kind.name(), kind);
            assert_eq!(decoded.slot, slot);
            assert_eq!(decoded.signature, signature);
            assert_eq!(decoded.bitmap, bitmap);
            let payload = signed_payload(&decoded);
            let mut golden = vec![vote_tag];
            golden.extend_from_slice(&slot.to_le_bytes());
            if kind != "finalization" {
                golden.extend_from_slice(&[0x42; 32]);
            }
            golden.extend_from_slice(&[0x34, 0x12]);
            assert_eq!(payload, golden);
            assert_eq!(payload, reference::payload(kind, slot, [0x42; 32], 0x1234));
        }
    }
}

#[test]
fn every_wire_truncation_and_unbounded_length_is_rejected() {
    for kind in [
        "notarization",
        "finalization",
        "fast_finalization",
        "genesis",
    ] {
        let wire = reference::message(kind, 42, [0x42; 32], 1234, [0; 192], vec![0, 5, 0, 3]);
        for end in 0..wire.len() {
            assert!(decode_certificate(&wire[..end]).is_err(), "{kind} {end}");
        }
        let mut oversized = wire.clone();
        let offset = if kind == "finalization" { 202 } else { 234 };
        oversized[offset..offset + 8].copy_from_slice(&u64::MAX.to_le_bytes());
        assert_eq!(
            decode_certificate(&oversized).unwrap_err().code,
            Code::MalformedCertificate
        );
        let mut trailing = wire;
        trailing.push(0);
        assert_eq!(
            decode_certificate(&trailing).unwrap_err().code,
            Code::MalformedCertificate
        );
    }
}

#[test]
fn canonical_base2_decoding_agrees_with_upstream_across_byte_boundaries() {
    for count in [1, 5, 7, 8, 9, 16, 255, 256, 257, MAX_VALIDATORS] {
        let mut bits: BitVec<u8, Lsb0> = BitVec::repeat(false, count);
        bits.set(0, true);
        bits.set(count - 1, true);
        let encoded = encode_base2(&bits).unwrap();
        let Decoded::Base2(upstream) = decode(&encoded, count).unwrap() else {
            panic!()
        };
        let expected: Vec<u32> = upstream.iter_ones().map(|rank| rank as u32).collect();
        assert_eq!(decode_signers(&encoded, count).unwrap(), expected);
        assert!(decode_signers(&encoded[..encoded.len() - 1], count).is_err());
        let mut trailing = encoded;
        trailing.push(0);
        assert!(decode_signers(&trailing, count).is_err());
    }
}

#[test]
fn nonzero_padding_is_a_documented_stricter_rule_than_upstream() {
    let bitmap = [0, 5, 0, 0x83];
    let Decoded::Base2(upstream) = decode(&bitmap, 5).unwrap() else {
        panic!()
    };
    assert_eq!(upstream.iter_ones().collect::<Vec<_>>(), vec![0, 1]);
    assert_eq!(
        decode_signers(&bitmap, 5).unwrap_err().code,
        Code::MalformedBitmap
    );
}

#[test]
fn thresholds_are_exact_inclusive_and_safe_at_u64_max() {
    for kind in [
        CertificateKind::Notarization,
        CertificateKind::Finalization,
        CertificateKind::FastFinalization,
        CertificateKind::Genesis,
    ] {
        let percent = kind.threshold_percent();
        assert!(!meets_threshold(percent - 1, 100, kind));
        assert!(meets_threshold(percent, 100, kind));
        assert!(meets_threshold(percent + 1, 100, kind));
        let boundary = (u128::from(u64::MAX) * u128::from(percent)).div_ceil(100) as u64;
        assert!(!meets_threshold(boundary - 1, u64::MAX, kind));
        assert!(meets_threshold(boundary, u64::MAX, kind));
        assert!(!meets_threshold(0, 0, kind));
        assert!(!meets_threshold(101, 100, kind));
    }
}

#[test]
fn subgroup_negatives_are_on_curve_and_rejected_by_checked_decoding() {
    let (_, cp) = fixture("wrong_subgroup_key");
    let bytes: [u8; 96] = hex::decode(cp["validators"][0]["bls_pubkey_hex"].as_str().unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(bytes, reference::non_subgroup_key());
    let point =
        Option::<blstrs::G1Affine>::from(blstrs::G1Affine::from_uncompressed_unchecked(&bytes))
            .unwrap();
    assert!(bool::from(point.is_on_curve()));
    assert!(!bool::from(point.is_torsion_free()));
    assert!(bool::from(
        blstrs::G1Affine::from_uncompressed(&bytes).is_none()
    ));
    let signature = reference::non_subgroup_signature();
    let point =
        Option::<blstrs::G2Affine>::from(blstrs::G2Affine::from_uncompressed_unchecked(&signature))
            .unwrap();
    assert!(bool::from(point.is_on_curve()));
    assert!(!bool::from(point.is_torsion_free()));
    assert!(bool::from(
        blstrs::G2Affine::from_uncompressed(&signature).is_none()
    ));
}

#[test]
fn unselected_keys_do_not_need_pop_but_selected_keys_always_do() {
    let (bundle, mut cp) = fixture("valid_notarization");
    cp["validators"][4]["proof_of_possession_hex"] = Value::Null;
    assert_eq!(
        run(bundle.clone(), cp.clone()).conclusion,
        Conclusion::Verified
    );
    cp["validators"][0]["proof_of_possession_hex"] = Value::Null;
    let report = run(bundle, cp);
    assert_eq!(report.conclusion, Conclusion::Inconclusive);
    assert_eq!(
        stage(&report, Stage::Signature).status,
        CheckStatus::NotChecked
    );
}

#[test]
fn trust_is_separate_even_when_certificate_is_valid() {
    let (bundle, cp) = fixture("valid_notarization");
    let digest = parse_checkpoint(&serde_json::to_vec(&cp).unwrap())
        .unwrap()
        .digest()
        .clone();
    for selected in [None, Some(digest)] {
        let report = verify_certificates(&request(&bundle, selected), &TrustStore::new());
        assert_eq!(report.conclusion, Conclusion::Inconclusive);
        assert!(report.certificates.is_empty());
    }
}

#[test]
fn finalization_inspection_cannot_claim_a_block_id() {
    let (mut bundle, cp) = fixture("valid_finalization");
    bundle["claim"]["block_id_hex"] = json!("42".repeat(32));
    let report = run(bundle, cp);
    assert_eq!(
        stage(&report, Stage::CertificateDecoding).diagnostic.code,
        Code::ClaimMismatch
    );
}

#[test]
fn block_claims_stay_inconclusive_even_with_valid_certificates() {
    for name in ["block_claim_deferred", "genesis_claim_deferred"] {
        let (bundle, cp) = fixture(name);
        let report = run(bundle, cp);
        assert_eq!(report.conclusion, Conclusion::Inconclusive);
        assert_eq!(report.certificates[0].conclusion, Conclusion::Verified);
        assert_eq!(
            report.checks.last().unwrap().diagnostic.code,
            Code::FinalityNotEvaluated
        );
    }
}

#[test]
fn mixed_bundle_preserves_individual_results_and_never_hides_a_failure() {
    let (mut bundle, cp) = fixture("valid_notarization");
    let (mut bad, _) = fixture("changed_bitmap");
    bad["certificates"][0]["id"] = json!("cert-2");
    bundle["certificates"]
        .as_array_mut()
        .unwrap()
        .push(bad["certificates"][0].clone());
    bundle["claim"]["certificate_ids"] = json!(["cert-1", "cert-2"]);
    let report = run(bundle, cp);
    assert_eq!(report.conclusion, Conclusion::Rejected);
    assert_eq!(report.certificates[0].conclusion, Conclusion::Verified);
    assert_eq!(report.certificates[1].conclusion, Conclusion::Rejected);
}

#[test]
fn original_denominator_is_not_reduced_to_qualifiable_or_selected_stake() {
    let (bundle, mut cp) = fixture("valid_notarization");
    cp["total_stake"] = json!("200");
    cp["validators"][4]["proof_of_possession_hex"] = Value::Null;
    let report = run(bundle, cp);
    assert_eq!(report.certificates[0].selected_stake.unwrap().get(), 65);
    assert_eq!(report.certificates[0].total_stake.unwrap().get(), 200);
    assert_eq!(stage(&report, Stage::Signature).status, CheckStatus::Pass);
    assert_eq!(
        stage(&report, Stage::Quorum).diagnostic.code,
        Code::InsufficientStake
    );
}

#[test]
fn contracts_profile_is_not_silently_upgraded_to_a_crypto_profile() {
    let bundle = include_bytes!("../../../fixtures/contracts/bundle.json");
    let cp = include_bytes!("../../../fixtures/contracts/checkpoint.json");
    let mut trust = TrustStore::new();
    let digest = trust
        .pin(parse_checkpoint(cp).unwrap(), TrustBasis::SyntheticFixture)
        .unwrap();
    let request = VerifyRequest {
        schema_version: 1,
        run_id: "old".into(),
        profile_id: monocle_core::PROFILE_ID.into(),
        bundle_json: String::from_utf8(bundle.to_vec()).unwrap(),
        selected_checkpoint: Some(digest),
    };
    let report = verify_certificates(&serde_json::to_vec(&request).unwrap(), &trust);
    assert_eq!(report.conclusion, Conclusion::Inconclusive);
    assert!(report.certificates.is_empty());
    assert_eq!(report.checks[1].diagnostic.code, Code::UnsupportedProfile);
}

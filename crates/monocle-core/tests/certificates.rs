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
                        && check.diagnostic.path == case["expected"]["path"]
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

#[test]
fn checkpoint_key_defects_leave_the_evidence_unjudged() {
    // Keys and proofs come from the caller-trusted checkpoint, which the evidence
    // cannot change. Their failure is a trust-context defect, not a rejection.
    for (name, code) in [
        ("invalid_pop", Code::InvalidPop),
        ("wrong_key_pop", Code::InvalidPop),
        ("invalid_pubkey", Code::InvalidPublicKey),
        ("identity_pubkey", Code::InvalidPublicKey),
        ("wrong_subgroup_key", Code::InvalidPublicKey),
    ] {
        let (bundle, cp) = fixture(name);
        let report = run(bundle, cp);
        let cert = &report.certificates[0];
        assert_eq!(report.conclusion, Conclusion::Inconclusive, "{name}");
        assert_eq!(cert.conclusion, Conclusion::Inconclusive, "{name}");
        let stage = |stage| cert.checks.iter().find(|c| c.stage == stage).unwrap();
        let qualification = stage(Stage::KeyQualification);
        assert_eq!(qualification.status, CheckStatus::Fail, "{name}");
        assert_eq!(qualification.diagnostic.code, code, "{name}");
        assert!(
            qualification
                .diagnostic
                .message
                .contains("checkpoint defect"),
            "{name}"
        );
        assert_eq!(stage(Stage::Signature).status, CheckStatus::NotChecked);
        assert_eq!(stage(Stage::Quorum).status, CheckStatus::NotChecked);
    }
}

fn with_raw(mut bundle: Value, change: impl FnOnce(&mut Vec<u8>)) -> Value {
    let mut raw = hex::decode(bundle["certificates"][0]["raw_hex"].as_str().unwrap()).unwrap();
    change(&mut raw);
    bundle["certificates"][0]["raw_hex"] = json!(hex::encode(raw));
    bundle
}

#[test]
fn an_off_curve_signature_is_malformed_not_outside_the_subgroup() {
    // The tamper lab's preset: flipping a coordinate bit leaves a point off the curve.
    for (name, at) in [("valid_notarization", 42), ("valid_finalization", 10)] {
        for offset in [at, at + 100] {
            let (bundle, cp) = fixture(name);
            let report = run(with_raw(bundle, |raw| raw[offset] ^= 1), cp);
            let check = stage(&report, Stage::Signature);
            assert_eq!(report.conclusion, Conclusion::Rejected, "{name} {offset}");
            assert_eq!(
                check.diagnostic.code,
                Code::MalformedSignature,
                "{name} {offset}"
            );
            assert_eq!(
                check.diagnostic.path,
                format!("certificates[0].raw_hex[{at}..{}]", at + 192)
            );
            assert_eq!(
                stage(&report, Stage::Quorum).status,
                CheckStatus::NotChecked
            );
        }
    }
}

#[test]
fn decoded_layout_names_every_field_of_the_fixed_byte_layout() {
    for kind in [
        "notarization",
        "finalization",
        "fast_finalization",
        "genesis",
    ] {
        let wire = reference::message(kind, 42, [0x42; 32], 1234, [0x37; 192], vec![0, 5, 0, 3]);
        let layout = decode_certificate(&wire).unwrap().layout;
        let signature = if kind == "finalization" { 10 } else { 42 };
        assert_eq!(layout.version, 0..1);
        assert_eq!(layout.kind, 1..2);
        assert_eq!(layout.slot, 2..10);
        assert_eq!(layout.block_id, (kind != "finalization").then_some(10..42));
        assert_eq!(layout.signature, signature..signature + 192);
        assert_eq!(layout.bitmap_length, signature + 192..signature + 200);
        assert_eq!(layout.bitmap, signature + 200..signature + 204);
        assert_eq!(layout.shred_version, signature + 204..wire.len());
        assert_eq!(&wire[layout.signature], &[0x37; 192]);
        assert_eq!(&wire[layout.bitmap], &[0, 5, 0, 3]);
    }
}

#[test]
fn every_decoding_fault_names_bytes_inside_the_input() {
    for kind in [
        "notarization",
        "finalization",
        "fast_finalization",
        "genesis",
    ] {
        let wire = reference::message(kind, 42, [0x42; 32], 1234, [0; 192], vec![0, 5, 0, 3]);
        for end in 0..wire.len() {
            let fault = decode_certificate(&wire[..end]).unwrap_err();
            assert_eq!(fault.code, Code::MalformedCertificate, "{kind} {end}");
            // A truncation points from the incomplete field to the end of the input.
            assert!(
                fault.bytes.start <= end && fault.bytes.end == end,
                "{kind} {end}"
            );
        }
        let length = if kind == "finalization" { 202 } else { 234 };
        let mut oversized = wire.clone();
        oversized[length..length + 8].copy_from_slice(&u64::MAX.to_le_bytes());
        assert_eq!(
            decode_certificate(&oversized).unwrap_err().bytes,
            length..length + 8
        );
        let mut trailing = wire.clone();
        trailing.extend_from_slice(&[0, 0, 0]);
        let fault = decode_certificate(&trailing).unwrap_err();
        assert_eq!(fault.bytes, wire.len()..wire.len() + 3);
        assert!(fault.message.starts_with("3 trailing bytes"));
    }
}

#[test]
fn quorum_messages_state_exact_stake_and_the_required_minimum() {
    let (bundle, cp) = fixture("notar_under_quorum");
    assert_eq!(
        stage(&run(bundle, cp), Stage::Quorum).diagnostic.message,
        "Signers hold 58 of 100 stake; 60% needs at least 60. The signature itself is valid"
    );
    let (bundle, cp) = fixture("valid_fast");
    assert_eq!(
        stage(&run(bundle, cp), Stage::Quorum).diagnostic.message,
        "Signers hold 80 of 100 stake, meeting the exact 80% threshold of 80"
    );
    // The displayed minimum must agree with the acceptance rule at every boundary.
    for total in [1, 3, 7, 99, 100, 101, u64::MAX] {
        for kind in CertificateKind::ALL {
            let required = required_stake(total, *kind);
            assert!(meets_threshold(required, total, *kind), "{total} {kind:?}");
            assert!(
                !meets_threshold(required - 1, total, *kind),
                "{total} {kind:?}"
            );
        }
    }
}

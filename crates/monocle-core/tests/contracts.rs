use monocle_core::*;
use serde_json::{Value, json};

const BUNDLE: &[u8] = include_bytes!("../../../fixtures/contracts/bundle.json");
const CHECKPOINT: &[u8] = include_bytes!("../../../fixtures/contracts/checkpoint.json");
fn bundle() -> Value {
    serde_json::from_slice(BUNDLE).unwrap()
}
fn checkpoint() -> Value {
    serde_json::from_slice(CHECKPOINT).unwrap()
}
fn bytes(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}
fn request(raw: &[u8], selected: Option<ContentDigest>) -> Vec<u8> {
    serde_json::to_vec(&VerifyRequest {
        schema_version: 1,
        run_id: "test-run".into(),
        profile_id: PROFILE_ID.into(),
        bundle_json: String::from_utf8(raw.to_vec()).unwrap(),
        selected_checkpoint: selected,
    })
    .unwrap()
}
fn trusted() -> (TrustStore, ContentDigest) {
    let mut store = TrustStore::new();
    let digest = store
        .pin(
            parse_checkpoint(CHECKPOINT).unwrap(),
            TrustBasis::SyntheticFixture,
        )
        .unwrap();
    (store, digest)
}
fn code(report: &VerificationReport, stage: Stage) -> Code {
    report
        .checks
        .iter()
        .find(|c| c.stage == stage)
        .unwrap()
        .diagnostic
        .code
}

#[test]
fn preserves_original_bytes_and_exact_unsigned_integers() {
    let parsed = parse_bundle(BUNDLE).unwrap();
    assert_eq!(parsed.raw_bytes(), BUNDLE);
    assert_eq!(parsed.document().claim.slot.get(), u64::MAX);
    assert_eq!(
        parsed.document().certificates[0].raw_hex.as_slice(),
        &[1, 2, 3, 255]
    );
    let roundtrip = parse_bundle(&serde_json::to_vec(parsed.document()).unwrap()).unwrap();
    assert_eq!(
        roundtrip.document().certificates[0].raw_hex,
        parsed.document().certificates[0].raw_hex
    );
    assert_ne!(roundtrip.digest(), parsed.digest()); // Original formatting is part of file identity.
    let parsed = parse_checkpoint(CHECKPOINT).unwrap();
    assert_eq!(
        parsed.document().validators[0].stake.get(),
        9_007_199_254_740_993
    );
    assert_eq!(parsed.document().total_stake.get(), u64::MAX);
}
#[test]
fn sha256_has_an_independent_known_answer() {
    assert_eq!(
        serde_json::to_value(ContentDigest::of(b"abc")).unwrap(),
        json!("sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
    );
}
#[test]
fn rejects_numeric_and_noncanonical_u64_transport() {
    for value in [
        json!(9007199254740993u64),
        json!(1.5),
        json!("18446744073709551616"),
        json!("01"),
        json!("-1"),
        json!("1e3"),
        json!(""),
    ] {
        let mut b = bundle();
        b["claim"]["slot"] = value;
        let error = parse_bundle(&bytes(&b)).unwrap_err();
        assert_eq!(error.code, Code::InvalidInteger);
        assert_eq!(error.path, "claim.slot");
    }
}
#[test]
fn rejects_duplicate_keys_even_when_escaped_or_equal() {
    for text in [
        r#"{"schema_version":1,"schema_version":1}"#,
        r#"{"schema_version":1,"context":{"epoch":"1","\u0065poch":"2"}}"#,
    ] {
        assert_eq!(
            parse_bundle(text.as_bytes()).unwrap_err().code,
            Code::DuplicateField
        );
    }
}
#[test]
fn unsupported_versions_are_not_interpreted_as_v1() {
    let report = preflight(
        &request(br#"{"schema_version":2,"future":true}"#, None),
        &TrustStore::new(),
    );
    assert_eq!(report.conclusion, Conclusion::Inconclusive);
    assert_eq!(code(&report, Stage::Input), Code::UnsupportedVersion);
    let mut b = bundle();
    b["certificates"][0]["schema_version"] = json!(2);
    assert_eq!(
        parse_bundle(&bytes(&b)).unwrap_err().code,
        Code::UnsupportedVersion
    );
    b["schema_version"] = json!(1.0);
    assert_eq!(
        parse_bundle(&bytes(&b)).unwrap_err().code,
        Code::InvalidSchema
    );
}
#[test]
fn malformed_json_trailing_input_and_invalid_utf8_are_rejected() {
    for raw in [b"{".as_slice(), b"{}{}", &[0xff], b"null"] {
        assert!(parse_bundle(raw).is_err());
    }
    assert_eq!(parse_bundle(b"{}{}").unwrap_err().code, Code::MalformedJson);
}
#[test]
fn strict_limits_bound_bytes_depth_and_collection_counts() {
    assert_eq!(
        parse_bundle(&vec![b' '; MAX_BUNDLE_BYTES + 1])
            .unwrap_err()
            .code,
        Code::InputTooLarge
    );
    assert_eq!(
        parse_checkpoint(&vec![b' '; MAX_CHECKPOINT_BYTES + 1])
            .unwrap_err()
            .code,
        Code::InputTooLarge
    );
    assert_eq!(
        parse_request(&vec![b' '; MAX_REQUEST_BYTES + 1])
            .unwrap_err()
            .code,
        Code::InputTooLarge
    );
    let deep = format!("{}0{}", "[".repeat(34), "]".repeat(34));
    assert_eq!(
        parse_bundle(deep.as_bytes()).unwrap_err().code,
        Code::StructureLimit
    );
    let mut b = bundle();
    b["certificates"] = json!(vec![b["certificates"][0].clone(); MAX_CERTIFICATES + 1]);
    assert_eq!(
        parse_bundle(&bytes(&b)).unwrap_err().code,
        Code::LimitExceeded
    );
    let mut c = checkpoint();
    c["validators"] = json!(vec![c["validators"][0].clone(); MAX_VALIDATORS + 1]);
    assert_eq!(
        parse_checkpoint(&bytes(&c)).unwrap_err().code,
        Code::LimitExceeded
    );
}
#[test]
fn certificate_hex_has_size_and_canonical_form_checks() {
    for raw in ["", "0", "AA", "0x00", "zz"] {
        let mut b = bundle();
        b["certificates"][0]["raw_hex"] = json!(raw);
        assert_eq!(parse_bundle(&bytes(&b)).unwrap_err().code, Code::InvalidHex);
    }
    let mut b = bundle();
    b["certificates"][0]["raw_hex"] = json!("00".repeat(65536));
    assert!(parse_bundle(&bytes(&b)).is_ok());
    b["certificates"][0]["raw_hex"] = json!("00".repeat(65537));
    assert_eq!(parse_bundle(&bytes(&b)).unwrap_err().code, Code::InvalidHex);
}
#[test]
fn imported_flags_cannot_create_trust_or_pop_qualification() {
    let mut b = bundle();
    b["trusted"] = json!(true);
    assert_eq!(
        parse_bundle(&bytes(&b)).unwrap_err().code,
        Code::UnknownField
    );
    let mut c = checkpoint();
    c["validators"][0]["pop_verified"] = json!(true);
    assert_eq!(
        parse_checkpoint(&bytes(&c)).unwrap_err().code,
        Code::UnknownField
    );
    let parsed = parse_checkpoint(CHECKPOINT).unwrap(); // Parsing alone grants nothing.
    let report = preflight(
        &request(BUNDLE, Some(parsed.digest().clone())),
        &TrustStore::new(),
    );
    assert_eq!(code(&report, Stage::Context), Code::CheckpointNotTrusted);
}
#[test]
fn missing_context_keeps_all_dependent_checks_unperformed() {
    let report = preflight(&request(BUNDLE, None), &TrustStore::new());
    assert_eq!(report.conclusion, Conclusion::Inconclusive);
    assert_eq!(code(&report, Stage::Context), Code::MissingCheckpoint);
    assert!(
        report
            .checks
            .iter()
            .skip(3)
            .all(|c| c.status == CheckStatus::NotChecked)
    );
}
#[test]
fn valid_context_is_still_not_verified_and_unpinning_takes_effect() {
    let (mut store, digest) = trusted();
    let raw = request(BUNDLE, Some(digest.clone()));
    let report = preflight(&raw, &store);
    assert_eq!(code(&report, Stage::Context), Code::Ok);
    assert_eq!(report.conclusion, Conclusion::Inconclusive);
    assert_eq!(report.evidence_scope, "preflight_only");
    assert_eq!(
        code(&report, Stage::Signature),
        Code::VerifierNotImplemented
    );
    store.remove(&digest);
    assert_eq!(
        code(&preflight(&raw, &store), Stage::Context),
        Code::CheckpointNotTrusted
    );
}
#[test]
fn context_mismatches_and_reference_replacement_are_inconclusive() {
    let (store, digest) = trusted();
    for (field, value) in [
        ("epoch", json!("43")),
        ("shred_version", json!(1235)),
        ("genesis_hash_hex", json!("22".repeat(32))),
    ] {
        let mut b = bundle();
        b["context"][field] = value;
        let report = preflight(&request(&bytes(&b), Some(digest.clone())), &store);
        assert_eq!(code(&report, Stage::Context), Code::ContextMismatch);
        assert_eq!(report.conclusion, Conclusion::Inconclusive);
    }
    let mut b = bundle();
    b["checkpoint_reference"] = serde_json::to_value(ContentDigest::of(b"different")).unwrap();
    assert_eq!(
        code(
            &preflight(&request(&bytes(&b), Some(digest)), &store),
            Stage::Context
        ),
        Code::CheckpointMismatch
    );
}
#[test]
fn profile_and_format_fail_closed() {
    let mut raw: Value = serde_json::from_slice(&request(BUNDLE, None)).unwrap();
    raw["profile_id"] = json!("future");
    assert_eq!(
        code(&preflight(&bytes(&raw), &TrustStore::new()), Stage::Profile),
        Code::UnsupportedProfile
    );
    let mut b = bundle();
    b["profile_id"] = json!("future");
    assert_eq!(
        code(
            &preflight(&request(&bytes(&b), None), &TrustStore::new()),
            Stage::Profile
        ),
        Code::ProfileMismatch
    );
    b = bundle();
    b["certificates"][0]["format"] = json!("future-format");
    assert_eq!(
        code(
            &preflight(&request(&bytes(&b), None), &TrustStore::new()),
            Stage::Profile
        ),
        Code::UnsupportedCertificate
    );
}
#[test]
fn rank_order_and_duplicate_members_are_rejected_without_sorting() {
    let mut c = checkpoint();
    c["validators"].as_array_mut().unwrap().swap(0, 1);
    assert_eq!(
        parse_checkpoint(&bytes(&c)).unwrap_err().code,
        Code::InvalidRankMap
    );
    for field in ["identity_hex", "bls_pubkey_hex"] {
        c = checkpoint();
        c["validators"][1][field] = c["validators"][0][field].clone();
        assert_eq!(
            parse_checkpoint(&bytes(&c)).unwrap_err().code,
            Code::InvalidRankMap
        );
    }
}
#[test]
fn denominator_is_preserved_and_large_stake_sums_do_not_wrap() {
    let mut c = checkpoint();
    for validator in c["validators"].as_array_mut().unwrap() {
        validator["stake"] = json!(u64::MAX.to_string());
    }
    assert_eq!(
        parse_checkpoint(&bytes(&c)).unwrap_err().code,
        Code::InvalidStake
    );
    c = checkpoint();
    c["total_stake"] = json!("0");
    assert_eq!(
        parse_checkpoint(&bytes(&c)).unwrap_err().code,
        Code::InvalidStake
    );
    c = checkpoint();
    c["validators"][0]["stake"] = json!("0");
    assert_eq!(
        parse_checkpoint(&bytes(&c)).unwrap_err().code,
        Code::InvalidStake
    );
    assert_eq!(
        parse_checkpoint(CHECKPOINT)
            .unwrap()
            .document()
            .total_stake
            .get(),
        u64::MAX
    );
}
#[test]
fn claim_must_select_every_certificate_once() {
    let mut b = bundle();
    b["claim"]["certificate_ids"] = json!(["sample-1", "sample-1"]);
    assert_eq!(
        parse_bundle(&bytes(&b)).unwrap_err().code,
        Code::DuplicateId
    );
    b["claim"]["certificate_ids"] = json!(["absent"]);
    assert_eq!(
        parse_bundle(&bytes(&b)).unwrap_err().code,
        Code::InvalidSchema
    );
    b = bundle();
    let mut extra = b["certificates"][0].clone();
    extra["id"] = json!("extra");
    b["certificates"].as_array_mut().unwrap().push(extra);
    assert_eq!(
        parse_bundle(&bytes(&b)).unwrap_err().code,
        Code::InvalidSchema
    );
}
#[test]
fn mutation_metadata_has_no_self_referential_bundle_digest() {
    let mut b = bundle();
    b["parent_digest"] = serde_json::to_value(ContentDigest::of(b"parent bundle")).unwrap();
    b["mutations"] = json!([{"schema_version":1,"certificate_id":"sample-1","operation":"replace_bytes","offset":"1","before_hex":"00","after_hex":"02","parent_certificate_digest":ContentDigest::of(&[1,0,3,255]),"result_certificate_digest":ContentDigest::of(&[1,2,3,255])}]);
    assert!(parse_bundle(&bytes(&b)).is_ok());
    b["mutations"][0]["offset"] = json!(u64::MAX.to_string());
    assert_eq!(
        parse_bundle(&bytes(&b)).unwrap_err().code,
        Code::InvalidMutation
    );
    b["mutations"][0]["offset"] = json!("1");
    b["mutations"][0]["after_hex"] = json!("03");
    assert_eq!(
        parse_bundle(&bytes(&b)).unwrap_err().code,
        Code::InvalidMutation
    );
}
#[test]
fn synthetic_trust_cannot_be_used_for_network_capture_claims() {
    let (store, digest) = trusted();
    let mut b = bundle();
    b["provenance"]["kind"] = json!("capture");
    assert_eq!(
        code(
            &preflight(&request(&bytes(&b), Some(digest)), &store),
            Stage::Context
        ),
        Code::ContextMismatch
    );
    let mut c = checkpoint();
    c["provenance"]["kind"] = json!("capture");
    let mut store = TrustStore::new();
    assert!(
        store
            .pin(
                parse_checkpoint(&bytes(&c)).unwrap(),
                TrustBasis::SyntheticFixture
            )
            .is_err()
    );
    assert!(
        store
            .pin(
                parse_checkpoint(CHECKPOINT).unwrap(),
                TrustBasis::OperatorPinned { reason: " ".into() }
            )
            .is_err()
    );
}

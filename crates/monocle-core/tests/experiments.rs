use monocle_core::{experiment::*, *};
use serde_json::{Value, json};
fn case(name: &str) -> Value {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/finality/corpus.json")).unwrap();
    corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap()
        .clone()
}
fn mutate(source: &str, action: Value) -> Result<DerivedEvidence, Diagnostic> {
    apply_mutation(&serde_json::to_vec(&json!({"schema_version":1,"baseline_json":source,"certificate_id":"fast","action":action})).unwrap())
}
fn archive(source: &str, derived: Option<&str>) -> Value {
    json!({"schema_version":1,"format":"monocle-replay","profile_id":certificate::PROFILE_ID,"policy_id":finality::POLICY_ID,"baseline_json":source,"derived_json":derived,"checkpoint_json":null,"checkpoint_digest":null,"saved_reports":null})
}
fn parse(value: &Value) -> Result<ReplayArchive, Diagnostic> {
    parse_replay(&serde_json::to_vec(value).unwrap())
}
fn verify(source: &str, cp: Option<&str>) -> Value {
    let mut trust = TrustStore::new();
    let selected = cp.map(|raw| {
        trust
            .pin(
                parse_checkpoint(raw.as_bytes()).unwrap(),
                TrustBasis::SyntheticFixture,
            )
            .unwrap()
    });
    let req = json!({"schema_version":1,"run_id":"replay-test","profile_id":certificate::PROFILE_ID,"bundle_json":source,"selected_checkpoint":selected});
    serde_json::to_value(finality::verify_evidence(
        &serde_json::to_vec(&req).unwrap(),
        &trust,
    ))
    .unwrap()
}
#[test]
fn mutation_preserves_original_and_records_reproducible_bytes() {
    let f = case("fast_finality");
    let source = f["bundle_json"].as_str().unwrap();
    let original = source.to_owned();
    let result = mutate(source, json!({"kind":"signature_bit"})).unwrap();
    assert_eq!(source, original);
    let before = parse_bundle(source.as_bytes()).unwrap();
    let after = parse_bundle(result.bundle_json.as_bytes()).unwrap();
    assert_eq!(
        after.document().parent_digest.as_ref(),
        Some(before.digest())
    );
    assert_ne!(after.digest(), before.digest());
    assert_eq!(after.document().mutations[0].offset.get(), 42);
    validate_lineage(&before, &after).unwrap();
    let cp = f["checkpoint_json"].as_str().unwrap();
    assert_eq!(verify(source, Some(cp))["conclusion"], "VERIFIED");
    assert_eq!(
        verify(&result.bundle_json, Some(cp))["conclusion"],
        "REJECTED"
    );
}
#[test]
fn changed_signed_field_changes_claim_binding_check() {
    let f = case("fast_finality");
    let result = mutate(
        f["bundle_json"].as_str().unwrap(),
        json!({"kind":"replace_bytes","offset":"2","after_hex":"2b"}),
    )
    .unwrap();
    let report = verify(&result.bundle_json, f["checkpoint_json"].as_str());
    assert_eq!(report["conclusion"], "REJECTED");
    assert!(report.to_string().contains("CLAIM_MISMATCH"));
}
#[test]
fn mutation_rejects_noops_ranges_numeric_offsets_unknown_fields_and_chains() {
    let f = case("fast_finality");
    let s = f["bundle_json"].as_str().unwrap();
    for action in [
        json!({"kind":"replace_bytes","offset":"18446744073709551615","after_hex":"ff"}),
        json!({"kind":"replace_bytes","offset":2,"after_hex":"ff"}),
        json!({"kind":"replace_bytes","offset":"2","after_hex":"2a"}),
        json!({"kind":"signature_bit","trusted":true}),
    ] {
        assert!(mutate(s, action).is_err());
    }
    let d = mutate(s, json!({"kind":"signature_bit"})).unwrap();
    assert_eq!(
        mutate(&d.bundle_json, json!({"kind":"signature_bit"}))
            .err()
            .unwrap()
            .code,
        Code::InvalidMutation
    );
}
#[test]
fn signature_preset_handles_finalize_layout() {
    let f = case("slow_finality");
    let d=apply_mutation(&serde_json::to_vec(&json!({"schema_version":1,"baseline_json":f["bundle_json"],"certificate_id":"finalize","action":{"kind":"signature_bit"}})).unwrap()).unwrap();
    assert_eq!(d.document.mutations[0].offset.get(), 10);
    assert_eq!(
        verify(&d.bundle_json, f["checkpoint_json"].as_str())["conclusion"],
        "REJECTED"
    );
}
#[test]
fn forged_before_parent_or_unrelated_metadata_cannot_pass_replay() {
    let f = case("fast_finality");
    let s = f["bundle_json"].as_str().unwrap();
    let d = mutate(s, json!({"kind":"signature_bit"})).unwrap();
    let good: Value = serde_json::from_str(&d.bundle_json).unwrap();
    for path in ["before", "parent", "metadata", "other_certificate_bytes"] {
        let mut bad = good.clone();
        match path {
            "before" => bad["mutations"][0]["before_hex"] = json!("ff"),
            "parent" => {
                bad["mutations"][0]["parent_certificate_digest"] =
                    json!(format!("sha256:{}", "00".repeat(32)))
            }
            "metadata" => bad["provenance"]["description"] = json!("forged lineage"),
            _ => bad["claim"]["slot"] = json!("43"),
        }
        assert_eq!(
            parse(&archive(s, Some(&bad.to_string())))
                .err()
                .unwrap()
                .code,
            Code::InvalidMutation,
            "{path}"
        );
    }
    let padded = format!("{s}\n");
    assert!(parse(&archive(&padded, Some(&d.bundle_json))).is_err());
}
#[test]
fn export_reimport_keeps_exact_inputs_but_does_not_select_trust_or_use_saved_verdicts() {
    let f = case("fast_finality");
    let s = f["bundle_json"].as_str().unwrap();
    let cp = f["checkpoint_json"].as_str().unwrap();
    let d = mutate(s, json!({"kind":"signature_bit"})).unwrap();
    let mut a = archive(s, Some(&d.bundle_json));
    a["checkpoint_json"] = json!(cp);
    a["checkpoint_digest"] = json!(ContentDigest::of(cp.as_bytes()));
    a["saved_reports"] = json!({"baseline":{"conclusion":"REJECTED"},"derived":{"conclusion":"VERIFIED","trusted":true}});
    let replay = parse(&a).unwrap();
    assert_eq!(replay.baseline_json, s);
    assert_eq!(replay.checkpoint_json.as_deref(), Some(cp));
    assert_eq!(
        verify(replay.derived_json.as_ref().unwrap(), None)["conclusion"],
        "INCONCLUSIVE"
    );
    assert_eq!(
        verify(replay.derived_json.as_ref().unwrap(), Some(cp)),
        verify(&d.bundle_json, Some(cp))
    );
    let serialized = serde_json::to_vec(&replay).unwrap();
    assert_eq!(parse_replay(&serialized).unwrap().baseline_json, s);
}
#[test]
fn checkpoint_reference_is_checked_but_can_be_supplied_separately() {
    let f = case("fast_finality");
    let s = f["bundle_json"].as_str().unwrap();
    let cp = f["checkpoint_json"].as_str().unwrap();
    let mut a = archive(s, None);
    a["checkpoint_digest"] = json!(ContentDigest::of(cp.as_bytes()));
    assert!(parse(&a).is_ok(), "{:?}", parse(&a).err());
    a["checkpoint_json"] = json!(format!("{cp}\n"));
    assert_eq!(parse(&a).err().unwrap().code, Code::CheckpointMismatch);
    a["checkpoint_json"] = json!(cp);
    assert!(parse(&a).is_ok(), "{:?}", parse(&a).err());
    a["trusted"] = json!(true);
    assert_eq!(parse(&a).err().unwrap().code, Code::UnknownField);
}
#[test]
fn replay_versions_policy_profiles_duplicates_and_sizes_fail_closed() {
    let f = case("fast_finality");
    let a = archive(f["bundle_json"].as_str().unwrap(), None);
    for (field, value, code) in [
        ("schema_version", json!(2), Code::UnsupportedVersion),
        (
            "policy_id",
            json!("future-policy"),
            Code::UnsupportedProfile,
        ),
        ("profile_id", json!("other"), Code::ProfileMismatch),
    ] {
        let mut changed = a.clone();
        changed[field] = value;
        assert_eq!(parse(&changed).err().unwrap().code, code);
    }
    assert_eq!(
        parse_replay(br#"{"schema_version":1,"schema_version":1}"#)
            .err()
            .unwrap()
            .code,
        Code::DuplicateField
    );
    assert_eq!(
        parse_replay(&vec![0; MAX_REPLAY_BYTES + 1])
            .err()
            .unwrap()
            .code,
        Code::InputTooLarge
    );
    let mut too_big = a;
    too_big["baseline_json"] = json!(" ".repeat(MAX_BUNDLE_BYTES + 1));
    assert_eq!(parse(&too_big).err().unwrap().code, Code::InputTooLarge);
}
#[test]
fn archive_supports_large_checkpoint_strings_without_relaxing_bundle_parser() {
    let f = case("fast_finality");
    let cp = format!(
        "{}{}",
        f["checkpoint_json"].as_str().unwrap(),
        " ".repeat(MAX_BUNDLE_BYTES)
    );
    let mut source: Value = serde_json::from_str(f["bundle_json"].as_str().unwrap()).unwrap();
    source["checkpoint_reference"] = json!(ContentDigest::of(cp.as_bytes()));
    let mut a = archive(&source.to_string(), None);
    a["checkpoint_json"] = json!(cp);
    a["checkpoint_digest"] = json!(ContentDigest::of(cp.as_bytes()));
    assert!(parse(&a).is_ok(), "{:?}", parse(&a).err());
    let mut bundle: Value = serde_json::from_str(f["bundle_json"].as_str().unwrap()).unwrap();
    bundle["provenance"]["description"] = json!("x".repeat(MAX_BUNDLE_BYTES + 1));
    assert!(parse_bundle(bundle.to_string().as_bytes()).is_err());
}
#[test]
fn under_quorum_control_is_signed_independently() {
    let f = case("fast_under_quorum");
    let r = verify(
        f["bundle_json"].as_str().unwrap(),
        f["checkpoint_json"].as_str(),
    );
    let checks = r["certificates"][0]["checks"].as_array().unwrap();
    assert!(
        checks
            .iter()
            .any(|c| c["stage"] == "signature" && c["status"] == "PASS")
    );
    assert!(
        checks
            .iter()
            .any(|c| c["stage"] == "quorum" && c["status"] == "FAIL")
    );
}

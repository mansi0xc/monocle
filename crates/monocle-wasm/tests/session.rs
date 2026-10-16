use monocle_wasm::Session;
use serde_json::{Value, json};

fn fixture() -> Value {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/finality/corpus.json")).unwrap();
    corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "fast_finality")
        .unwrap()
        .clone()
}
fn request(case: &Value, digest: Value) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "schema_version": 1, "run_id": "bridge-test",
        "profile_id": monocle_core::certificate::PROFILE_ID,
        "bundle_json": case["bundle_json"], "selected_checkpoint": digest
    }))
    .unwrap()
}
#[test]
fn inspection_is_not_trust_and_selection_can_be_revoked() {
    let case = fixture();
    let mut s = Session::default();
    let cp = s.execute(2, case["checkpoint_json"].as_str().unwrap().as_bytes());
    let req = request(&case, cp["value"]["digest"].clone());
    assert_eq!(s.execute(5, &req)["value"]["conclusion"], "INCONCLUSIVE");
    assert_eq!(s.execute(3, b"")["ok"], true);
    assert_eq!(s.execute(5, &req)["value"]["conclusion"], "VERIFIED");
    s.execute(6, b"");
    assert_eq!(s.execute(5, &req)["value"]["conclusion"], "INCONCLUSIVE");
}
#[test]
fn malformed_replacement_and_failed_selection_revoke_trust() {
    let case = fixture();
    let mut s = Session::default();
    let cp = s.execute(2, case["checkpoint_json"].as_str().unwrap().as_bytes());
    let req = request(&case, cp["value"]["digest"].clone());
    s.execute(3, b"");
    assert_eq!(s.execute(4, b" ")["ok"], false);
    assert_eq!(s.execute(5, &req)["value"]["conclusion"], "INCONCLUSIVE");
    s.execute(3, b"");
    s.execute(2, b"{");
    assert_eq!(s.execute(3, b"")["ok"], false);
    assert_eq!(s.execute(5, &req)["value"]["conclusion"], "INCONCLUSIVE");
}
#[test]
fn preserves_exact_source_identity_and_u64() {
    let case = fixture();
    let mut s = Session::default();
    let mut bundle: Value = serde_json::from_str(case["bundle_json"].as_str().unwrap()).unwrap();
    bundle["claim"]["slot"] = json!("18446744073709551615");
    let bytes = serde_json::to_vec(&bundle).unwrap();
    let result = s.execute(1, &bytes);
    assert_eq!(
        result["value"]["document"]["claim"]["slot"],
        "18446744073709551615"
    );
    let mut padded = bytes.clone();
    padded.push(b' ');
    assert_ne!(
        result["value"]["digest"],
        s.execute(1, &padded)["value"]["digest"]
    );
}
#[test]
fn untrusted_flags_duplicates_bad_utf8_and_limits_are_rejected() {
    let case = fixture();
    let mut s = Session::default();
    let mut cp: Value = serde_json::from_str(case["checkpoint_json"].as_str().unwrap()).unwrap();
    cp["trusted"] = json!(true);
    assert_eq!(
        s.execute(2, &serde_json::to_vec(&cp).unwrap())["error"]["code"],
        "UNKNOWN_FIELD"
    );
    assert_eq!(
        s.execute(1, br#"{"schema_version":1,"schema_version":1}"#)["error"]["code"],
        "DUPLICATE_FIELD"
    );
    assert_eq!(s.execute(1, &[255])["ok"], false);
    assert_eq!(
        s.execute(1, &vec![0; monocle_core::MAX_BUNDLE_BYTES + 1])["error"]["code"],
        "INPUT_TOO_LARGE"
    );
    assert_eq!(s.execute(0, b"")["ok"], false);
}
#[test]
fn operator_reason_is_required_and_cannot_be_imported() {
    let case = fixture();
    let mut s = Session::default();
    let cp = s.execute(2, case["checkpoint_json"].as_str().unwrap().as_bytes());
    assert_eq!(s.execute(4, &[255])["ok"], false);
    assert_eq!(s.execute(4, &vec![b'x'; 1025])["ok"], false);
    assert_eq!(
        s.execute(4, b"Reviewed local synthetic checkpoint")["ok"],
        true
    );
    assert_eq!(
        s.execute(5, &request(&case, cp["value"]["digest"].clone()))["value"]["conclusion"],
        "VERIFIED"
    );
}
#[test]
fn operation_numbers_are_stable_and_unknown_numbers_fail_closed() {
    use monocle_wasm::Operation;
    let numbers: Vec<u32> = Operation::ALL.iter().map(|op| *op as u32).collect();
    assert_eq!(numbers, (1..=10).collect::<Vec<_>>());
    for op in Operation::ALL {
        assert_eq!(Operation::from_u32(op as u32), Some(op));
    }
    let mut s = Session::default();
    for unknown in [0, 11, u32::MAX] {
        let response = s.execute(unknown, b"");
        assert_eq!(response["ok"], false);
        assert_eq!(response["error"]["code"], "INVALID_SCHEMA");
        assert_eq!(response["error"]["path"], "operation");
    }
}
#[test]
fn registered_key_selection_is_explicit_and_revocable() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/finality/corpus.json")).unwrap();
    let case = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "missing_pop")
        .unwrap()
        .clone();
    let mut s = Session::default();
    let cp = s.execute(2, case["checkpoint_json"].as_str().unwrap().as_bytes());
    let req = request(&case, cp["value"]["digest"].clone());
    // Operator trust still demands proofs of possession.
    assert_eq!(s.execute(4, b"Reviewed locally")["ok"], true);
    assert_eq!(s.execute(5, &req)["value"]["conclusion"], "INCONCLUSIVE");
    // Registered-key trust accepts the keys without proofs.
    assert_eq!(s.execute(10, b"Rebuilt from vote accounts")["ok"], true);
    let report = s.execute(5, &req);
    assert_eq!(report["value"]["conclusion"], "VERIFIED");
    assert!(report.to_string().contains("REGISTERED_KEYS_ACCEPTED"));
    // An empty reason fails and revokes the earlier selection.
    assert_eq!(s.execute(10, b" ")["ok"], false);
    assert_eq!(s.execute(5, &req)["value"]["conclusion"], "INCONCLUSIVE");
}

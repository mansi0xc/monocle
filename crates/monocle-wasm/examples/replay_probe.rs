use monocle_wasm::Session;
use serde_json::{Value, json};
fn main() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/finality/corpus.json")).unwrap();
    let case = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "fast_finality")
        .unwrap();
    let request = json!({"schema_version":1,"baseline_json":case["bundle_json"],"certificate_id":"fast","action":{"kind":"signature_bit"}});
    let mutated = Session::default().execute(7, &serde_json::to_vec(&request).unwrap());
    let archive = json!({"schema_version":1,"format":"monocle-replay","profile_id":monocle_core::certificate::PROFILE_ID,"policy_id":monocle_core::finality::POLICY_ID,"baseline_json":case["bundle_json"],"derived_json":mutated["value"]["bundle_json"],"checkpoint_json":case["checkpoint_json"],"checkpoint_digest":monocle_core::ContentDigest::of(case["checkpoint_json"].as_str().unwrap().as_bytes()),"saved_reports":{"baseline":null,"derived":{"conclusion":"VERIFIED","trusted":true}}});
    let mut wrong = archive.clone();
    wrong["baseline_json"] = json!(format!("{}\n", case["bundle_json"].as_str().unwrap()));
    let mut future = archive.clone();
    future["policy_id"] = json!("future");
    let mut oversized_offset = request.clone();
    oversized_offset["action"] =
        json!({"kind":"replace_bytes","offset":"18446744073709551615","after_hex":"ff"});
    let mut extra = request.clone();
    extra["action"]["trusted"] = json!(true);
    let cases: Vec<_> = [
        (7, request),
        (8, archive),
        (8, wrong),
        (8, future),
        (7, oversized_offset),
        (7, extra),
    ]
    .into_iter()
    .map(|(operation, value)| {
        let input_json = value.to_string();
        let response = Session::default().execute(operation, input_json.as_bytes());
        json!({"operation":operation,"input_json":input_json,"response":response})
    })
    .collect();
    println!("{}", json!(cases));
}

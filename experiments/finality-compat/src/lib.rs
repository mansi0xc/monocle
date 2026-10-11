//! Fixed end-to-end C4 corpus: original bytes, explicit trust, real crypto, composition.
use monocle_core::{certificate::PROFILE_ID, finality::*, *};
use serde_json::{Value, json};
pub fn corpus_results() -> Value {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/finality/corpus.json")).unwrap();
    let cases: Vec<_> = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| {
            let mut trust = TrustStore::new();
            let digest = trust
                .pin(
                    parse_checkpoint(case["checkpoint_json"].as_str().unwrap().as_bytes()).unwrap(),
                    TrustBasis::SyntheticFixture,
                )
                .unwrap();
            let bundle: Value =
                serde_json::from_str(case["bundle_json"].as_str().unwrap()).unwrap();
            let request = VerifyRequest {
                schema_version: 1,
                run_id: case["name"].as_str().unwrap().into(),
                profile_id: bundle["profile_id"].as_str().unwrap_or(PROFILE_ID).into(),
                bundle_json: case["bundle_json"].as_str().unwrap().into(),
                selected_checkpoint: case["select_checkpoint"]
                    .as_bool()
                    .unwrap()
                    .then_some(digest),
            };
            let report = serde_json::to_value(verify_evidence(
                &serde_json::to_vec(&request).unwrap(),
                &trust,
            ))
            .unwrap();
            let expected = &case["expected"];
            for field in [
                "conclusion",
                "evidence_scope",
                "finality_path",
                "supporting_certificate_ids",
            ] {
                assert_eq!(report[field], expected[field], "{}: {field}", case["name"]);
            }
            assert!(
                report["checks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .chain(
                        report["certificates"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .flat_map(|cert| cert["checks"].as_array().unwrap())
                    )
                    .any(|check| check["diagnostic"]["code"] == expected["diagnostic"]),
                "{}",
                case["name"]
            );
            json!({"name":case["name"],"report":report})
        })
        .collect();
    json!({"policy_id":POLICY_ID,"cases":cases})
}
#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::cell::RefCell;
    thread_local! { static OUTPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) }; }
    #[unsafe(no_mangle)]
    pub extern "C" fn monocle_finality_run() -> u32 {
        let bytes = serde_json::to_vec(&super::corpus_results()).expect("fixed corpus serializes");
        let len = bytes.len() as u32;
        OUTPUT.with(|out| *out.borrow_mut() = bytes);
        len
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn monocle_finality_byte(index: u32) -> u32 {
        OUTPUT.with(|out| {
            out.borrow()
                .get(index as usize)
                .map_or(256, |byte| u32::from(*byte))
        })
    }
}

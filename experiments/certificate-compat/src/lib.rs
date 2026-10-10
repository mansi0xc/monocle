//! Fixed signed-certificate corpus for native/WASM interoperability checks.
use monocle_core::{
    certificate::{PROFILE_ID, protocol_profile, verify_certificates},
    *,
};
use serde_json::{Value, json};
const CORPUS: &str = include_str!("../../../fixtures/certificates/corpus.json");
pub fn corpus_results() -> Value {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let cases: Vec<_> = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| {
            let mut trust = TrustStore::new();
            let selected = trust
                .pin(
                    parse_checkpoint(case["checkpoint_json"].as_str().unwrap().as_bytes()).unwrap(),
                    TrustBasis::SyntheticFixture,
                )
                .unwrap();
            let request = VerifyRequest {
                schema_version: 1,
                run_id: case["name"].as_str().unwrap().into(),
                profile_id: PROFILE_ID.into(),
                bundle_json: case["bundle_json"].as_str().unwrap().into(),
                selected_checkpoint: Some(selected),
            };
            let report = serde_json::to_value(verify_certificates(
                &serde_json::to_vec(&request).unwrap(),
                &trust,
            ))
            .unwrap();
            let expected = &case["expected"];
            let checks: Vec<_> = report["checks"]
                .as_array()
                .unwrap()
                .iter()
                .chain(
                    report["certificates"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .flat_map(|cert| cert["checks"].as_array().unwrap()),
                )
                .collect();
            assert_eq!(
                report["conclusion"], expected["conclusion"],
                "{}",
                case["name"]
            );
            assert!(
                checks
                    .iter()
                    .any(|check| check["stage"] == expected["stage"]
                        && check["diagnostic"]["code"] == expected["code"]),
                "{}: missing expected diagnostic",
                case["name"]
            );
            json!({"name":case["name"],"report":report})
        })
        .collect();
    json!({"profile":protocol_profile(),"cases":cases})
}
#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::cell::RefCell;
    thread_local! { static OUTPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) }; }
    #[unsafe(no_mangle)]
    pub extern "C" fn monocle_certificates_run() -> u32 {
        let bytes = serde_json::to_vec(&super::corpus_results()).expect("fixed corpus serializes");
        let len = bytes.len() as u32;
        OUTPUT.with(|out| *out.borrow_mut() = bytes);
        len
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn monocle_certificates_byte(index: u32) -> u32 {
        OUTPUT.with(|out| {
            out.borrow()
                .get(index as usize)
                .map_or(256, |byte| u32::from(*byte))
        })
    }
}

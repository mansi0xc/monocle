//! Cross-target C2 fixture corpus. This fixed probe is not the product WASM API.
use monocle_core::*;
use serde_json::{Value, json};
pub const BUNDLE: &[u8] = include_bytes!("../../../fixtures/contracts/bundle.json");
pub const CHECKPOINT: &[u8] = include_bytes!("../../../fixtures/contracts/checkpoint.json");

pub fn corpus_results() -> Value {
    let parsed = parse_bundle(BUNDLE).unwrap();
    let mut trust = TrustStore::new();
    let selected = trust
        .pin(
            parse_checkpoint(CHECKPOINT).unwrap(),
            TrustBasis::SyntheticFixture,
        )
        .unwrap();
    let baseline: Value = serde_json::from_slice(BUNDLE).unwrap();
    let mut cases = Vec::new();
    for name in [
        "context_ready",
        "missing_checkpoint",
        "untrusted_checkpoint",
        "duplicate_key",
        "numeric_slot",
        "injected_trust",
        "unknown_version",
        "unknown_profile",
        "unknown_format",
        "wrong_epoch",
        "wrong_reference",
        "malformed_hex",
        "malformed_json",
    ] {
        let mut bundle = baseline.clone();
        let mut checkpoint = Some(selected.clone());
        let mut profile = PROFILE_ID.to_string();
        match name {
            "missing_checkpoint" => checkpoint = None,
            "untrusted_checkpoint" => checkpoint = Some(ContentDigest::of(b"unknown")),
            "numeric_slot" => bundle["claim"]["slot"] = json!(9007199254740993u64),
            "injected_trust" => bundle["trusted"] = json!(true),
            "unknown_version" => bundle["schema_version"] = json!(2),
            "unknown_profile" => profile = "unsupported".into(),
            "unknown_format" => bundle["certificates"][0]["format"] = json!("unknown"),
            "wrong_epoch" => bundle["context"]["epoch"] = json!("43"),
            "wrong_reference" => {
                bundle["checkpoint_reference"] = json!(ContentDigest::of(b"other"))
            }
            "malformed_hex" => bundle["certificates"][0]["raw_hex"] = json!("ABC"),
            _ => {}
        }
        let raw = match name {
            "context_ready" => String::from_utf8(BUNDLE.to_vec()).unwrap(),
            "duplicate_key" => r#"{"schema_version":1,"schema_version":2}"#.into(),
            "malformed_json" => "{".into(),
            _ => serde_json::to_string(&bundle).unwrap(),
        };
        let request = VerifyRequest {
            schema_version: 1,
            run_id: name.into(),
            profile_id: profile,
            bundle_json: raw,
            selected_checkpoint: checkpoint,
        };
        cases.push(
            json!({"name":name,"report":preflight(&serde_json::to_vec(&request).unwrap(), &trust)}),
        );
    }
    json!({"profile":protocol_profile(),"bundle_digest":parsed.digest(),"checkpoint_digest":selected,"slot":parsed.document().claim.slot,"certificate_hex":hex_text(parsed.document().certificates[0].raw_hex.as_slice()),"cases":cases})
}
fn hex_text(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::cell::RefCell;
    thread_local! { static OUTPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) }; }
    #[unsafe(no_mangle)]
    pub extern "C" fn monocle_contracts_run() -> u32 {
        let bytes =
            serde_json::to_vec(&super::corpus_results()).expect("bounded corpus serializes");
        let len = bytes.len() as u32;
        OUTPUT.with(|output| *output.borrow_mut() = bytes);
        len
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn monocle_contracts_byte(index: u32) -> u32 {
        OUTPUT.with(|output| {
            output
                .borrow()
                .get(index as usize)
                .map_or(256, |byte| u32::from(*byte))
        })
    }
}

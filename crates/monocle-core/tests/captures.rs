use monocle_core::{capture::*, *};
use serde_json::{Value, json};
const SOURCE: &[u8] = include_bytes!("../../../fixtures/captures/testnet-genesis-rpc.json");
fn inspect(v: &Value) -> Result<CaptureReport, Diagnostic> {
    inspect_genesis_rpc(&serde_json::to_vec(v).unwrap())
}
fn source() -> Value {
    serde_json::from_slice(SOURCE).unwrap()
}
#[test]
fn recorded_response_preserves_exact_fields_and_never_claims_verification() {
    let r = inspect_genesis_rpc(SOURCE).unwrap();
    assert_eq!(r.source_digest, ContentDigest::of(SOURCE));
    assert_eq!(r.source_bytes, SOURCE.len());
    assert_eq!(r.conclusion, Conclusion::Inconclusive);
    let c = r.certificate.unwrap();
    assert_eq!(c.slot.get(), 444625255);
    assert_eq!(
        c.block_id_hex,
        "59ad7c6f9810400fb7746fb84393a776d3080069d06224ace01bb3ae98a1196e"
    );
    for (field, text) in [("signature", c.signature_hex), ("bitmap", c.bitmap_hex)] {
        assert_eq!(
            hex::decode(text).unwrap(),
            serde_json::from_value::<Vec<u8>>(source()["result"]["signature"][field].clone())
                .unwrap()
        );
    }
    assert_eq!(r.requirements.len(), 4);
    for c in r
        .checks
        .iter()
        .filter(|c| !matches!(c.stage, Stage::Input | Stage::CertificateDecoding))
    {
        assert_eq!(c.status, CheckStatus::NotChecked);
    }
    let mut padded = SOURCE.to_vec();
    padded.push(b' ');
    assert_ne!(
        inspect_genesis_rpc(&padded).unwrap().source_digest,
        r.source_digest
    );
}
#[test]
fn null_method_unavailable_and_other_error_are_distinct_observations() {
    for (value, observation, code) in [
        (
            json!({"jsonrpc":"2.0","id":1,"result":null}),
            "no_certificate",
            Code::RpcCertificateUnavailable,
        ),
        (
            json!({"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}),
            "method_unavailable",
            Code::RpcMethodUnavailable,
        ),
        (
            json!({"jsonrpc":"2.0","id":null,"error":{"code":-32000,"message":"unavailable","data":{"retry":true}}}),
            "rpc_error",
            Code::RpcRequestFailed,
        ),
    ] {
        let r = inspect(&value).unwrap();
        assert_eq!(serde_json::to_value(&r.observation).unwrap(), observation);
        assert_eq!(r.conclusion, Conclusion::Inconclusive);
        assert!(r.certificate.is_none());
        assert_eq!(r.checks[1].diagnostic.code, code);
    }
}
#[test]
fn slot_numbers_remain_exact_through_rust_and_leave_as_decimal_strings() {
    let mut v = source();
    v["result"]["block"]["slot"] = json!(u64::MAX);
    let r = inspect(&v).unwrap();
    assert_eq!(
        serde_json::to_value(r).unwrap()["certificate"]["slot"],
        "18446744073709551615"
    );
    for slot in [json!(-1), json!(1.0), json!("42"), json!(null)] {
        v["result"]["block"]["slot"] = slot;
        assert!(inspect(&v).is_err());
    }
}
#[test]
fn binary_arrays_have_exact_sizes_integer_elements_and_bounded_bitmap() {
    for (path, value) in [
        ("block", json!({"slot":1,"blockId":[0]})),
        ("signature", json!({"signature":vec![0u8;191],"bitmap":[0]})),
        ("signature", json!({"signature":vec![0u8;192],"bitmap":[]})),
        (
            "signature",
            json!({"signature":vec![0u8;192],"bitmap":vec![0u8;414]}),
        ),
        (
            "signature",
            json!({"signature":vec![0u8;192],"bitmap":[256]}),
        ),
        (
            "signature",
            json!({"signature":vec![0u8;192],"bitmap":[0.0]}),
        ),
        (
            "signature",
            json!({"signature":vec![0u8;192],"bitmap":[-1]}),
        ),
    ] {
        let mut v = source();
        v["result"][path] = value;
        assert!(inspect(&v).is_err());
    }
}
#[test]
fn contradictory_envelopes_unknown_fields_and_saved_trust_are_rejected() {
    for v in [
        json!({"jsonrpc":"2.0","id":1}),
        json!({"jsonrpc":"2.0","id":1,"result":null,"error":null}),
        json!({"jsonrpc":"1.0","id":1,"result":null}),
        json!({"jsonrpc":"2.0","result":null}),
        json!({"jsonrpc":"2.0","id":{},"result":null}),
        json!({"jsonrpc":"2.0","id":1,"result":null,"trusted":true}),
        json!({"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"no","conclusion":"VERIFIED"}}),
    ] {
        assert!(inspect(&v).is_err());
    }
    let mut v = source();
    v["result"]["signature"]["trusted"] = json!(true);
    assert_eq!(inspect(&v).unwrap_err().code, Code::UnknownField);
}
#[test]
fn never_uses_footer_or_transport_slot_as_certificate_slot() {
    let mut v = source();
    v["slot"] = json!(999);
    assert!(inspect(&v).is_err());
    let mut v = source();
    v["result"]["slot"] = json!(999);
    assert!(inspect(&v).is_err());
    let v = json!({"slot":999,"certificate":source()["result"]});
    assert!(inspect(&v).is_err());
    assert!(
        inspect(&source()["result"]).is_err(),
        "bare results need an explicit supported envelope"
    );
}
#[test]
fn malformed_duplicate_oversized_and_deep_inputs_fail() {
    assert_eq!(
        inspect_genesis_rpc(b"{\"jsonrpc\":\"2.0\",\"jsonrpc\":\"2.0\"}")
            .unwrap_err()
            .code,
        Code::DuplicateField
    );
    assert!(inspect_genesis_rpc(&[255]).is_err());
    assert_eq!(
        inspect_genesis_rpc(&vec![b' '; MAX_CAPTURE_BYTES + 1])
            .unwrap_err()
            .code,
        Code::InputTooLarge
    );
    let deep = format!("{}0{}", "[".repeat(40), "]".repeat(40));
    assert_eq!(
        inspect_genesis_rpc(deep.as_bytes()).unwrap_err().code,
        Code::StructureLimit
    );
    let mut v = source();
    v["result"]["signature"]["bitmap"] = json!(vec![0u8; 4097]);
    assert_eq!(inspect(&v).unwrap_err().code, Code::StructureLimit);
}
#[test]
fn native_command_is_read_only_and_reports_parse_status_separately_from_validity() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let mut child = Command::new(env!("CARGO_BIN_EXE_monocle-capture"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(SOURCE).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&out.stdout).unwrap()["conclusion"],
        "INCONCLUSIVE"
    );
    let out = Command::new(env!("CARGO_BIN_EXE_monocle-capture"))
        .args(["unexpected", "extra"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

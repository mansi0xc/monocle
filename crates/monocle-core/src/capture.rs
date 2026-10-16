//! Read-only genesis RPC adapter. Never synthesizes absent signing/trust context.
use crate::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_CAPTURE_BYTES: usize = 64 * 1024;
pub const ADAPTER_ID: &str = "agave-genesis-rpc-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Block {
    #[serde(rename = "blockId")]
    block_id: Vec<u8>,
    slot: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Aggregate {
    signature: Vec<u8>,
    bitmap: Vec<u8>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Genesis {
    block: Block,
    signature: Aggregate,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RpcError {
    code: i64,
    message: String,
    // Transport details are preserved in the original file, not interpreted as instructions.
    #[serde(rename = "data")]
    _data: Option<Value>,
}
#[derive(Debug, Serialize)]
pub struct CapturedGenesis {
    pub slot: DecimalU64,
    pub block_id_hex: String,
    pub signature_hex: String,
    pub bitmap_hex: String,
}
listed_enum! {
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureObservation {
    CertificatePresent,
    NoCertificate,
    MethodUnavailable,
    RpcError,
}
}
#[derive(Debug, Serialize)]
pub struct CaptureReport {
    pub schema_version: u16,
    pub adapter_id: &'static str,
    pub source_digest: ContentDigest,
    pub source_bytes: usize,
    pub observation: CaptureObservation,
    pub conclusion: Conclusion,
    pub evidence_scope: &'static str,
    pub certificate: Option<CapturedGenesis>,
    pub rpc_error_code: Option<String>,
    pub rpc_error_message: Option<String>,
    pub checks: Vec<CheckResult>,
    pub requirements: Vec<&'static str>,
    pub assumptions: Vec<&'static str>,
}
fn invalid(path: &str, message: &str) -> Diagnostic {
    Diagnostic::new(Code::InvalidSchema, path, message)
}
fn typed<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, Diagnostic> {
    serde_path_to_error::deserialize(value).map_err(|e| {
        Diagnostic::from_serde(
            &e.inner().to_string(),
            e.path().to_string(),
            Code::InvalidSchema,
        )
    })
}
fn check(stage: Stage, status: CheckStatus, code: Code, message: &str) -> CheckResult {
    CheckResult {
        stage,
        status,
        diagnostic: Diagnostic::new(code, "$", message),
    }
}

/// The input must be a single JSON-RPC 2.0 response, not a footer, bare result,
/// Monocle bundle, or saved report. All source bytes are hashed before normalization.
pub fn inspect_genesis_rpc(bytes: &[u8]) -> Result<CaptureReport, Diagnostic> {
    let value = crate::json::strict_value(bytes, MAX_CAPTURE_BYTES)?;
    let object = value
        .as_object()
        .ok_or_else(|| invalid("$", "Expected a JSON-RPC response object"))?;
    if object
        .keys()
        .any(|key| !["jsonrpc", "id", "result", "error"].contains(&key.as_str()))
    {
        return Err(Diagnostic::new(
            Code::UnknownField,
            "$",
            "Unknown JSON-RPC response field; footers and saved reports are not supported",
        ));
    }
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(invalid("jsonrpc", "Expected JSON-RPC version 2.0"));
    }
    let id = object
        .get("id")
        .ok_or_else(|| invalid("id", "Missing JSON-RPC response ID"))?;
    if !(id.is_null() || id.as_str().is_some_and(|s| s.len() <= 128) || id.is_i64() || id.is_u64())
    {
        return Err(invalid(
            "id",
            "Response ID must be null, a short string, or an integer",
        ));
    }
    // Presence matters: null result is distinct from an absent result or an error.
    if object.contains_key("result") == object.contains_key("error") {
        return Err(invalid("$", "Supply exactly one of result or error"));
    }
    let mut report = CaptureReport {
        schema_version: 1,
        adapter_id: ADAPTER_ID,
        source_digest: ContentDigest::of(bytes),
        source_bytes: bytes.len(),
        observation: CaptureObservation::NoCertificate,
        conclusion: Conclusion::Inconclusive,
        evidence_scope: "capture_inspection",
        certificate: None,
        rpc_error_code: None,
        rpc_error_message: None,
        checks: vec![check(
            Stage::Input,
            CheckStatus::Pass,
            Code::Ok,
            "JSON-RPC envelope parsed; endpoint identity is not authenticated",
        )],
        requirements: vec![],
        assumptions: vec![
            "The response is untrusted local evidence. Its digest identifies this file, not its sender or capture time.",
            "The selected adapter describes getAgGenesisCert; a JSON-RPC response does not itself name the method.",
            "Capture inspection never selects a checkpoint, reconstructs a signed payload, or establishes network finality.",
        ],
    };
    if let Some(error) = object.get("error") {
        let error: RpcError = typed(error.clone())?;
        if error.message.len() > 1024 {
            return Err(invalid(
                "error.message",
                "RPC error message exceeds 1024 UTF-8 bytes",
            ));
        }
        let unavailable = error.code == -32601;
        report.observation = if unavailable {
            CaptureObservation::MethodUnavailable
        } else {
            CaptureObservation::RpcError
        };
        report.checks.push(check(Stage::CertificateDecoding, CheckStatus::NotChecked,
            if unavailable { Code::RpcMethodUnavailable } else { Code::RpcRequestFailed },
            "The endpoint returned an error, not a certificate or a statement that no certificate exists"));
        report.rpc_error_code = Some(error.code.to_string());
        report.rpc_error_message = Some(error.message);
    } else if object["result"].is_null() {
        report.checks.push(check(Stage::CertificateDecoding, CheckStatus::NotChecked, Code::RpcCertificateUnavailable,
            "The recorded endpoint returned null. This file does not independently establish the cluster's consensus state"));
    } else {
        let cert: Genesis = typed(object["result"].clone())?;
        if cert.block.block_id.len() != 32 || cert.signature.signature.len() != 192 {
            return Err(invalid(
                "result",
                "Expected a 32-byte block ID and 192-byte signature array",
            ));
        }
        if cert.signature.bitmap.is_empty()
            || cert.signature.bitmap.len() > 3 + MAX_VALIDATORS.div_ceil(5)
        {
            return Err(invalid(
                "result.signature.bitmap",
                "Expected 1..413 bitmap bytes within this adapter's bounds",
            ));
        }
        report.observation = CaptureObservation::CertificatePresent;
        report.certificate = Some(CapturedGenesis {
            slot: DecimalU64::new(cert.block.slot),
            block_id_hex: hex::encode(cert.block.block_id),
            signature_hex: hex::encode(cert.signature.signature),
            bitmap_hex: hex::encode(cert.signature.bitmap),
        });
        report.checks.push(check(Stage::CertificateDecoding, CheckStatus::Pass, Code::Ok,
            "Extracted result.block.slot, block ID, signature bytes and bitmap; cryptographic validity has not been checked"));
        report.requirements = vec![
            "Historical validator checkpoint: canonical rank order, BLS public keys, exact stakes and original denominator, authenticated independently.",
            "Historical signing context: shred version, epoch and genesis hash, with evidence linking them to this certificate; do not infer them from a current RPC bank.",
            "Key qualification compatible with the verifier's PoP policy; a registration flag or different proof payload is not a substitute.",
            "Producer/profile compatibility: confirm the capture's vote serialization and signer encoding against the pinned implementation before constructing verification input.",
        ];
    }
    for stage in [
        Stage::Profile,
        Stage::Context,
        Stage::Signers,
        Stage::KeyQualification,
        Stage::Signature,
        Stage::Quorum,
        Stage::Finality,
    ] {
        let (code, message) = if report.certificate.is_some() {
            (
                Code::CaptureContextMissing,
                "Raw RPC inspection lacks authenticated historical context and a confirmed verification profile; this stage did not run",
            )
        } else {
            (
                Code::PrerequisiteMissing,
                "No certificate was supplied by this response; this verification stage did not run",
            )
        };
        report
            .checks
            .push(check(stage, CheckStatus::NotChecked, code, message));
    }
    Ok(report)
}

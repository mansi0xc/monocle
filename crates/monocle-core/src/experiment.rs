//! One controlled replacement from an immutable baseline, plus untrusted replay packaging.
use crate::{
    certificate::{self, CertificateKind},
    finality, *,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const MAX_REPLAY_BYTES: usize = 16 * 1024 * 1024;
pub const REPLAY_FORMAT: &str = "monocle-replay";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationRequest {
    pub schema_version: u16,
    pub baseline_json: String,
    pub certificate_id: String,
    pub action: MutationAction,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MutationAction {
    SignatureBit {},
    ReplaceBytes {
        offset: DecimalU64,
        after_hex: Bytes<1, 4096>,
    },
}
#[derive(Serialize)]
pub struct DerivedEvidence {
    pub bundle_json: String,
    pub digest: ContentDigest,
    pub document: ProofBundle,
}

/// Informational data from another run. Never deserialized into a verifier report.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedReports {
    pub baseline: Option<Value>,
    pub derived: Option<Value>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayArchive {
    pub schema_version: u16,
    pub format: String,
    pub profile_id: String,
    pub policy_id: String,
    pub baseline_json: String,
    pub derived_json: Option<String>,
    pub checkpoint_json: Option<String>,
    pub checkpoint_digest: Option<ContentDigest>,
    pub saved_reports: Option<SavedReports>,
}
fn invalid(path: &str, message: &str) -> Diagnostic {
    Diagnostic::new(Code::InvalidMutation, path, message)
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
fn version(value: &Value) -> Result<(), Diagnostic> {
    if value.get("schema_version").and_then(Value::as_u64) != Some(1) {
        return Err(Diagnostic::new(
            Code::UnsupportedVersion,
            "schema_version",
            "Expected experiment/replay schema version 1",
        ));
    }
    Ok(())
}
fn baseline(bytes: &[u8]) -> Result<Parsed<ProofBundle>, Diagnostic> {
    let parsed = parse_bundle(bytes)?;
    if parsed.document().parent_digest.is_some() || !parsed.document().mutations.is_empty() {
        return Err(invalid(
            "baseline",
            "Use an unmodified baseline; chained experiments are not supported",
        ));
    }
    Ok(parsed)
}
fn patch(
    original: &Parsed<ProofBundle>,
    certificate_id: &str,
    offset: u64,
    after: &[u8],
) -> Result<ProofBundle, Diagnostic> {
    let mut result = original.document().clone();
    let cert = result
        .certificates
        .iter_mut()
        .find(|c| c.id == certificate_id)
        .ok_or_else(|| {
            invalid(
                "certificate_id",
                "Certificate does not exist in the baseline",
            )
        })?;
    let mut raw = cert.raw_hex.as_slice().to_vec();
    let end = offset
        .checked_add(after.len() as u64)
        .ok_or_else(|| invalid("offset", "Replacement range overflows"))?;
    if after.is_empty() || after.len() > 4096 || end > raw.len() as u64 {
        return Err(invalid(
            "offset",
            "Replacement must fit within the certificate and contain 1..4096 bytes",
        ));
    }
    let before = raw[offset as usize..end as usize].to_vec();
    if before == after {
        return Err(invalid(
            "after_hex",
            "The replacement must change at least one byte",
        ));
    }
    let parent = ContentDigest::of(&raw);
    raw[offset as usize..end as usize].copy_from_slice(after);
    let mutation = typed(json!({
        "schema_version": 1, "certificate_id": certificate_id, "operation": "replace_bytes",
        "offset": offset.to_string(), "before_hex": hex::encode(before), "after_hex": hex::encode(after),
        "parent_certificate_digest": parent, "result_certificate_digest": ContentDigest::of(&raw)
    }))?;
    cert.raw_hex = typed(json!(hex::encode(raw)))?;
    result.parent_digest = Some(original.digest().clone());
    result.mutations = vec![mutation];
    Ok(result)
}
pub fn apply_mutation(bytes: &[u8]) -> Result<DerivedEvidence, Diagnostic> {
    let value = crate::json::strict_value(bytes, MAX_REQUEST_BYTES)?;
    version(&value)?;
    let request: MutationRequest = typed(value)?;
    let original = baseline(request.baseline_json.as_bytes())?;
    if original.document().profile_id != certificate::PROFILE_ID {
        return Err(Diagnostic::new(
            Code::UnsupportedProfile,
            "profile_id",
            "Mutation tools require the pinned Base2 profile",
        ));
    }
    let (offset, after) = match request.action {
        MutationAction::ReplaceBytes { offset, after_hex } => {
            (offset.get(), after_hex.as_slice().to_vec())
        }
        MutationAction::SignatureBit {} => {
            let cert = original
                .document()
                .certificates
                .iter()
                .find(|c| c.id == request.certificate_id)
                .ok_or_else(|| {
                    invalid(
                        "certificate_id",
                        "Certificate does not exist in the baseline",
                    )
                })?;
            if cert.format != certificate::WIRE_FORMAT {
                return Err(Diagnostic::new(
                    Code::UnsupportedCertificate,
                    "format",
                    "Signature preset requires the pinned complete wire format",
                ));
            }
            let decoded = certificate::decode_certificate(cert.raw_hex.as_slice())?;
            // V1 tag (1), kind (1), slot (8), optional block ID (32), then signature.
            let offset = if decoded.kind == CertificateKind::Finalization {
                10
            } else {
                42
            };
            (offset, vec![decoded.signature[0] ^ 1])
        }
    };
    let document = patch(&original, &request.certificate_id, offset, &after)?;
    let bundle_json = serde_json::to_string(&document)
        .map_err(|_| invalid("derived", "Could not serialize experiment"))?;
    let parsed = parse_bundle(bundle_json.as_bytes())?;
    Ok(DerivedEvidence {
        bundle_json,
        digest: parsed.digest().clone(),
        document,
    })
}

/// Check complete derivation, not only the after-bytes that C2 can check alone.
/// This proves a content relationship, not author identity or capture authenticity.
pub fn validate_lineage(
    original: &Parsed<ProofBundle>,
    derived: &Parsed<ProofBundle>,
) -> Result<(), Diagnostic> {
    let doc = derived.document();
    if original.document().parent_digest.is_some()
        || doc.parent_digest.as_ref() != Some(original.digest())
        || doc.mutations.len() != 1
    {
        return Err(invalid(
            "parent_digest",
            "Replay requires exactly one replacement from the supplied unmodified baseline",
        ));
    }
    let mutation = &doc.mutations[0];
    let expected = patch(
        original,
        &mutation.certificate_id,
        mutation.offset.get(),
        mutation.after_hex.as_slice(),
    )?;
    // Rebuild all fields, including before-bytes and certificate digests. This
    // also rejects unrelated edits to claims, context, provenance, or other bytes.
    if serde_json::to_value(expected)
        .map_err(|_| invalid("derived_json", "Could not compare experiment"))?
        != serde_json::to_value(doc)
            .map_err(|_| invalid("derived_json", "Could not compare experiment"))?
    {
        return Err(invalid(
            "derived_json",
            "Mutation history or derived content does not reproduce from the baseline",
        ));
    }
    Ok(())
}
pub fn parse_replay(bytes: &[u8]) -> Result<ReplayArchive, Diagnostic> {
    let value =
        crate::json::strict_value_with_string_limit(bytes, MAX_REPLAY_BYTES, MAX_CHECKPOINT_BYTES)?;
    version(&value)?;
    let archive: ReplayArchive = typed(value)?;
    if archive.format != REPLAY_FORMAT {
        return Err(Diagnostic::new(
            Code::InvalidSchema,
            "format",
            "Expected monocle-replay archive",
        ));
    }
    if archive.policy_id != finality::POLICY_ID {
        return Err(Diagnostic::new(
            Code::UnsupportedProfile,
            "policy_id",
            "This replay requires a different finality policy",
        ));
    }
    let original = baseline(archive.baseline_json.as_bytes())?;
    if archive.profile_id != original.document().profile_id {
        return Err(Diagnostic::new(
            Code::ProfileMismatch,
            "profile_id",
            "Replay profile must identify its original bundle",
        ));
    }
    if let Some(derived) = &archive.derived_json {
        validate_lineage(&original, &parse_bundle(derived.as_bytes())?)?;
    }
    if let Some(reference) = &original.document().checkpoint_reference
        && archive.checkpoint_digest.as_ref() != Some(reference)
    {
        return Err(Diagnostic::new(
            Code::CheckpointMismatch,
            "checkpoint_digest",
            "Preserve the bundle's required checkpoint reference",
        ));
    }
    if let Some(raw) = &archive.checkpoint_json {
        let parsed = parse_checkpoint(raw.as_bytes())?;
        if archive.checkpoint_digest.as_ref() != Some(parsed.digest()) {
            return Err(Diagnostic::new(
                Code::CheckpointMismatch,
                "checkpoint_digest",
                "Included checkpoint bytes do not match the replay reference",
            ));
        }
    }
    if let Some(reports) = &archive.saved_reports {
        for report in [&reports.baseline, &reports.derived].into_iter().flatten() {
            if serde_json::to_vec(report).map_or(true, |v| v.len() > MAX_BUNDLE_BYTES) {
                return Err(Diagnostic::new(
                    Code::InputTooLarge,
                    "saved_reports",
                    "Each untrusted saved report is limited to 1 MiB",
                ));
            }
        }
    }
    Ok(archive)
}

use crate::*;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashSet;

/// Validated, immutable document plus original source bytes. Re-serialization is not identity.
#[derive(Clone, Debug)]
pub struct Parsed<T> {
    document: T,
    raw: Box<[u8]>,
    digest: ContentDigest,
}
impl<T> Parsed<T> {
    pub fn document(&self) -> &T {
        &self.document
    }
    pub fn raw_bytes(&self) -> &[u8] {
        &self.raw
    }
    pub fn digest(&self) -> &ContentDigest {
        &self.digest
    }
}

fn version(value: &Value, path: &str) -> Result<(), Diagnostic> {
    match value.get("schema_version").and_then(Value::as_u64) {
        Some(1) => Ok(()),
        Some(_) => Err(Diagnostic::new(
            Code::UnsupportedVersion,
            path,
            "Unsupported schema version; no fallback interpretation",
        )),
        None => Err(Diagnostic::new(
            Code::InvalidSchema,
            path,
            "schema_version must be an integer",
        )),
    }
}
fn typed<T: DeserializeOwned>(value: Value) -> Result<T, Diagnostic> {
    serde_path_to_error::deserialize(value).map_err(|e| {
        Diagnostic::from_serde(
            &e.inner().to_string(),
            e.path().to_string(),
            Code::InvalidSchema,
        )
    })
}
fn parsed<T>(document: T, bytes: &[u8]) -> Parsed<T> {
    Parsed {
        document,
        raw: bytes.into(),
        digest: ContentDigest::of(bytes),
    }
}
fn token(value: &str, path: &str) -> Result<(), Diagnostic> {
    if value.is_empty()
        || value.len() > 96
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-.:".contains(&b))
    {
        return Err(Diagnostic::new(
            Code::InvalidSchema,
            path,
            "Expected a 1..96 byte ASCII identifier",
        ));
    }
    Ok(())
}
fn count(len: usize, min: usize, max: usize, path: &str) -> Result<(), Diagnostic> {
    if !(min..=max).contains(&len) {
        return Err(Diagnostic::new(
            Code::LimitExceeded,
            path,
            format!("Expected {min}..{max} entries"),
        ));
    }
    Ok(())
}
fn provenance(p: &Provenance) -> Result<(), Diagnostic> {
    if p.description.len() > 1024 {
        return Err(Diagnostic::new(
            Code::LimitExceeded,
            "provenance.description",
            "Description exceeds 1024 UTF-8 bytes",
        ));
    }
    Ok(())
}

pub fn parse_bundle(bytes: &[u8]) -> Result<Parsed<ProofBundle>, Diagnostic> {
    let value = crate::json::strict_value(bytes, MAX_BUNDLE_BYTES)?;
    version(&value, "schema_version")?;
    // Versions are checked before interpreting nested bodies under v1 semantics.
    for field in ["certificates", "mutations"] {
        if let Some(items) = value.get(field).and_then(Value::as_array) {
            for (index, item) in items.iter().enumerate() {
                version(item, &format!("{field}[{index}].schema_version"))?;
            }
        }
    }
    let bundle: ProofBundle = typed(value)?;
    token(&bundle.profile_id, "profile_id")?;
    token(&bundle.claim.kind, "claim.kind")?;
    provenance(&bundle.provenance)?;
    count(
        bundle.certificates.len(),
        1,
        MAX_CERTIFICATES,
        "certificates",
    )?;
    count(
        bundle.claim.certificate_ids.len(),
        1,
        MAX_CERTIFICATES,
        "claim.certificate_ids",
    )?;
    count(bundle.mutations.len(), 0, MAX_MUTATIONS, "mutations")?;
    let mut ids = HashSet::new();
    for (index, cert) in bundle.certificates.iter().enumerate() {
        token(&cert.id, &format!("certificates[{index}].id"))?;
        token(&cert.kind, &format!("certificates[{index}].kind"))?;
        token(&cert.format, &format!("certificates[{index}].format"))?;
        if !ids.insert(&cert.id) {
            return Err(Diagnostic::new(
                Code::DuplicateId,
                "certificates",
                "Certificate IDs must be unique",
            ));
        }
    }
    let mut selected = HashSet::new();
    for id in &bundle.claim.certificate_ids {
        if !selected.insert(id) {
            return Err(Diagnostic::new(
                Code::DuplicateId,
                "claim.certificate_ids",
                "Claim selects a certificate more than once",
            ));
        }
        if !ids.contains(id) {
            return Err(Diagnostic::new(
                Code::InvalidSchema,
                "claim.certificate_ids",
                "Claim references an absent certificate",
            ));
        }
    }
    // C2 requires every supplied certificate to be selected: no silently ignored extras.
    if selected != ids {
        return Err(Diagnostic::new(
            Code::InvalidSchema,
            "claim.certificate_ids",
            "Select every supplied certificate exactly once",
        ));
    }
    if bundle.parent_digest.is_some() == bundle.mutations.is_empty() {
        return Err(Diagnostic::new(
            Code::InvalidMutation,
            "parent_digest",
            "Parent digest and mutation records must be supplied together",
        ));
    }
    let mut mutated = HashSet::new();
    for (index, mutation) in bundle.mutations.iter().enumerate() {
        let fail = || {
            Diagnostic::new(
                Code::InvalidMutation,
                format!("mutations[{index}]"),
                "Invalid replacement, reference, range, or resulting certificate digest",
            )
        };
        let cert = bundle
            .certificates
            .iter()
            .find(|cert| cert.id == mutation.certificate_id)
            .ok_or_else(fail)?;
        let after = mutation.after_hex.as_slice();
        let raw = cert.raw_hex.as_slice();
        let start = mutation.offset.get();
        let end = start.checked_add(after.len() as u64).ok_or_else(fail)?;
        if mutation.operation != "replace_bytes"
            || !mutated.insert(&mutation.certificate_id)
            || mutation.before_hex.as_slice().len() != after.len()
            || end > raw.len() as u64
            || mutation.result_certificate_digest != ContentDigest::of(raw)
            || &raw[start as usize..end as usize] != after
        {
            return Err(fail());
        }
    }
    Ok(parsed(bundle, bytes))
}

pub fn parse_checkpoint(bytes: &[u8]) -> Result<Parsed<EpochCheckpoint>, Diagnostic> {
    let value = crate::json::strict_value(bytes, MAX_CHECKPOINT_BYTES)?;
    version(&value, "schema_version")?;
    let checkpoint: EpochCheckpoint = typed(value)?;
    token(&checkpoint.profile_id, "profile_id")?;
    provenance(&checkpoint.provenance)?;
    count(checkpoint.validators.len(), 1, MAX_VALIDATORS, "validators")?;
    if checkpoint.total_stake.get() == 0 {
        return Err(Diagnostic::new(
            Code::InvalidStake,
            "total_stake",
            "Stake denominator must be positive",
        ));
    }
    let mut identities = HashSet::new();
    let mut keys = HashSet::new();
    let mut sum = 0u128;
    for (index, entry) in checkpoint.validators.iter().enumerate() {
        if entry.rank as usize != index
            || !identities.insert(&entry.identity_hex)
            || !keys.insert(&entry.bls_pubkey_hex)
        {
            return Err(Diagnostic::new(
                Code::InvalidRankMap,
                format!("validators[{index}]"),
                "Ranks must be contiguous in source order, with unique identities and keys",
            ));
        }
        if entry.stake.get() == 0 {
            return Err(Diagnostic::new(
                Code::InvalidStake,
                format!("validators[{index}].stake"),
                "Ranked validator stake must be positive",
            ));
        }
        sum += u128::from(entry.stake.get()); // At most 2048 u64 values: bounded below u128::MAX.
    }
    if sum > u128::from(checkpoint.total_stake.get()) {
        return Err(Diagnostic::new(
            Code::InvalidStake,
            "total_stake",
            "Listed stake exceeds the declared denominator",
        ));
    }
    Ok(parsed(checkpoint, bytes))
}

pub fn parse_request(bytes: &[u8]) -> Result<Parsed<VerifyRequest>, Diagnostic> {
    let value = crate::json::strict_value(bytes, MAX_REQUEST_BYTES)?;
    version(&value, "schema_version")?;
    let request: VerifyRequest = typed(value)?;
    token(&request.run_id, "run_id")?;
    token(&request.profile_id, "profile_id")?;
    if request.bundle_json.len() > MAX_BUNDLE_BYTES {
        return Err(Diagnostic::new(
            Code::InputTooLarge,
            "bundle_json",
            "Bundle exceeds size limit",
        ));
    }
    Ok(parsed(request, bytes))
}

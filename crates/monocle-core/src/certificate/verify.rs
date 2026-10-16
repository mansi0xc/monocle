use super::{
    CertificateReport, CertificateVerificationReport, PROFILE_ID, byte_path, decode_certificate,
    decode_signers, meets_threshold, required_stake, signed_payload,
};
use crate::*;
use solana_bls_signatures::{
    ProofOfPossession, Pubkey, PubkeyProjective, Signature, SignatureAffineUnchecked,
    SignatureProjective, VerifyPop, VerifySignature,
    pubkey::{PopVerified, PubkeyAffine},
    signature::SignatureAffine,
};
use std::collections::HashMap;

const CERT_STAGES: [Stage; 5] = [
    Stage::CertificateDecoding,
    Stage::Signers,
    Stage::KeyQualification,
    Stage::Signature,
    Stage::Quorum,
];
type KeyCache = HashMap<u32, Result<PopVerified<PubkeyAffine>, Diagnostic>>;
fn check(stage: Stage, status: CheckStatus, code: Code, message: &str) -> CheckResult {
    CheckResult {
        stage,
        status,
        diagnostic: Diagnostic::new(code, "$", message),
    }
}
/// Who a stopped check blames. Only evidence the bundle controls can be rejected.
/// Unsupported or missing context leaves the check unrun; a defect in the
/// caller-trusted checkpoint means the check ran and failed on data the
/// evidence cannot change, so the evidence itself is left unjudged.
fn outcome(code: Code) -> (CheckStatus, Conclusion) {
    match code {
        Code::UnsupportedWireVersion
        | Code::UnsupportedCertificate
        | Code::UnsupportedSignerEncoding
        | Code::ContextMismatch
        | Code::MissingPop => (CheckStatus::NotChecked, Conclusion::Inconclusive),
        Code::InvalidPublicKey | Code::InvalidPop => (CheckStatus::Fail, Conclusion::Inconclusive),
        _ => (CheckStatus::Fail, Conclusion::Rejected),
    }
}
impl CertificateReport {
    fn new(envelope: &CertificateEnvelope) -> Self {
        Self {
            certificate_id: envelope.id.clone(),
            raw_digest: ContentDigest::of(envelope.raw_hex.as_slice()),
            conclusion: Conclusion::Inconclusive,
            kind: None,
            slot: None,
            block_id_hex: None,
            signed_payload_hex: None,
            signer_ranks: Vec::new(),
            selected_stake: None,
            total_stake: None,
            threshold_percent: None,
            checks: CERT_STAGES
                .into_iter()
                .map(|stage| {
                    check(
                        stage,
                        CheckStatus::NotChecked,
                        Code::PrerequisiteMissing,
                        "Earlier certificate prerequisites have not passed",
                    )
                })
                .collect(),
        }
    }
    fn set(&mut self, stage: Stage, status: CheckStatus, diagnostic: Diagnostic) {
        if let Some(check) = self.checks.iter_mut().find(|check| check.stage == stage) {
            check.status = status;
            check.diagnostic = diagnostic;
        }
    }
    fn pass(&mut self, stage: Stage, message: &str) {
        self.set(
            stage,
            CheckStatus::Pass,
            Diagnostic::new(Code::Ok, "$", message),
        );
    }
    fn stop(mut self, stage: Stage, error: Diagnostic) -> Self {
        let (status, conclusion) = outcome(error.code);
        self.conclusion = conclusion;
        self.set(stage, status, error);
        self
    }
}

/// Explain a signature the checked conversion has already rejected. This
/// classification never accepts anything; it only chooses the diagnostic.
fn rejected_signature(signature: &Signature) -> (Code, &'static str) {
    match SignatureAffineUnchecked::try_from(signature) {
        Ok(point) if point.verify_subgroup().is_err() => (
            Code::SignatureOutsideSubgroup,
            "Signature is a curve point outside the prime-order subgroup G2, so it cannot be a BLS signature",
        ),
        _ => (
            Code::MalformedSignature,
            "Signature bytes are not a valid uncompressed G2 point: the encoding flags or coordinates are invalid, or the point is not on the curve",
        ),
    }
}

/// `registered` is true only under the caller's explicit `TrustBasis::RegisteredKeys`.
fn qualify(
    entry: &ValidatorEntry,
    registered: bool,
) -> Result<PopVerified<PubkeyAffine>, Diagnostic> {
    let mut key_bytes = [0; 96];
    key_bytes.copy_from_slice(entry.bls_pubkey_hex.as_slice());
    // Checked decoding applies under every basis: on curve, prime-order subgroup, not identity.
    let key = PubkeyAffine::try_from(&Pubkey(key_bytes)).map_err(|_| {
        Diagnostic::new(
            Code::InvalidPublicKey,
            format!("validators[{}].bls_pubkey_hex", entry.rank),
            "The selected checkpoint's public key for this signer is malformed, the identity, off curve, or outside the prime-order subgroup. This is a checkpoint defect; the evidence was not judged",
        )
    })?;
    if registered {
        // SAFETY: not memory safety but the library's PoP contract. The caller
        // explicitly selected registered-key trust: every key is the one the vote
        // program accepted after verifying its proof (SIMD-0387). This mirrors
        // Agave's `bls_pubkey_compressed_bytes_to_bls_pubkey`. No file field reaches here.
        return Ok(unsafe { PopVerified::new_unchecked(key) });
    }
    let proof = entry.proof_of_possession_hex.as_ref().ok_or_else(|| {
        Diagnostic::new(
            Code::MissingPop,
            format!("validators[{}].proof_of_possession_hex", entry.rank),
            "Selected signer has no actual proof of possession; a trust label cannot substitute",
        )
    })?;
    let mut proof_bytes = [0; 192];
    proof_bytes.copy_from_slice(proof.as_slice());
    // Standard public-key-bound PoP. No imported flag or unsafe qualification bypass.
    key.verify_and_wrap_pop(&ProofOfPossession(proof_bytes), None)
        .map_err(|_| {
            Diagnostic::new(
                Code::InvalidPop,
                format!("validators[{}].proof_of_possession_hex", entry.rank),
                "The selected checkpoint's proof of possession does not verify for this signer's key. This is a checkpoint defect; the evidence was not judged",
            )
        })
}

/// Paths name the bundle field and, for certificate bytes, the half-open byte
/// range within it, e.g. `certificates[0].raw_hex[42..234]` for a signature.
fn verify_one(
    index: usize,
    envelope: &CertificateEnvelope,
    bundle: &ProofBundle,
    checkpoint: &EpochCheckpoint,
    registered: bool,
    cache: &mut KeyCache,
) -> CertificateReport {
    let mut report = CertificateReport::new(envelope);
    let raw = format!("certificates[{index}].raw_hex");
    let bytes = |range| byte_path(&raw, range);
    let decoded = match decode_certificate(envelope.raw_hex.as_slice()) {
        Ok(cert) => cert,
        Err(fault) => return report.stop(Stage::CertificateDecoding, fault.at(&raw, 0)),
    };
    let layout = &decoded.layout;
    report.kind = Some(decoded.kind);
    report.slot = Some(DecimalU64::new(decoded.slot));
    report.block_id_hex = decoded.block_id.map(hex::encode);
    report.threshold_percent = Some(decoded.kind.threshold_percent() as u8);
    if decoded.kind.name() != envelope.kind {
        return report.stop(
            Stage::CertificateDecoding,
            Diagnostic::new(
                Code::EnvelopeMismatch,
                format!("certificates[{index}].kind"),
                format!(
                    "Envelope kind contradicts the wire tag at {}, which encodes {}",
                    bytes(&layout.kind),
                    decoded.kind.name()
                ),
            ),
        );
    }
    if decoded.shred_version != checkpoint.context.shred_version {
        return report.stop(
            Stage::CertificateDecoding,
            Diagnostic::new(
                Code::ContextMismatch,
                bytes(&layout.shred_version),
                format!(
                    "Wire shred version {} differs from the selected checkpoint's {}",
                    decoded.shred_version, checkpoint.context.shred_version
                ),
            ),
        );
    }
    if decoded.slot != bundle.claim.slot.get() {
        return report.stop(
            Stage::CertificateDecoding,
            Diagnostic::new(
                Code::ClaimMismatch,
                bytes(&layout.slot),
                format!(
                    "Certificate slot {} contradicts claim.slot {}",
                    decoded.slot,
                    bundle.claim.slot.get()
                ),
            ),
        );
    }
    if let (Some(claim), Some(id), Some(at)) = (
        bundle.claim.block_id_hex.as_ref(),
        decoded.block_id,
        layout.block_id.as_ref(),
    ) && claim.as_slice() != id
    {
        return report.stop(
            Stage::CertificateDecoding,
            Diagnostic::new(
                Code::ClaimMismatch,
                bytes(at),
                "Certificate block ID contradicts claim.block_id_hex",
            ),
        );
    }
    if bundle.claim.kind == "certificate_inspection"
        && decoded.block_id.is_none()
        && bundle.claim.block_id_hex.is_some()
    {
        return report.stop(Stage::CertificateDecoding, Diagnostic::new(Code::ClaimMismatch, "claim.block_id_hex", "A slot-only finalization certificate cannot bind an inspection claim to a block ID"));
    }
    let payload = signed_payload(&decoded);
    report.signed_payload_hex = Some(hex::encode(&payload));
    report.set(
        Stage::CertificateDecoding,
        CheckStatus::Pass,
        Diagnostic::new(
            Code::Ok,
            raw.clone(),
            "Supported complete wire message and declared context agree",
        ),
    );
    let bitmap_path = bytes(&layout.bitmap);
    let ranks = match decode_signers(&decoded.bitmap, checkpoint.validators.len()) {
        Ok(ranks) => ranks,
        Err(fault) => return report.stop(Stage::Signers, fault.at(&raw, layout.bitmap.start)),
    };
    let mut stake = 0u64;
    for rank in &ranks {
        let Some(entry) = checkpoint.validators.get(*rank as usize) else {
            return report.stop(
                Stage::Signers,
                Diagnostic::new(
                    Code::UnknownSignerRank,
                    bitmap_path,
                    format!("Signer position {rank} has no entry in the selected checkpoint"),
                ),
            );
        };
        let Some(sum) = stake.checked_add(entry.stake.get()) else {
            return report.stop(
                Stage::Signers,
                Diagnostic::new(
                    Code::StakeOverflow,
                    bitmap_path,
                    "Selected stake accumulation overflow",
                ),
            );
        };
        stake = sum;
    }
    report.signer_ranks = ranks.clone();
    report.selected_stake = Some(DecimalU64::new(stake));
    report.total_stake = Some(checkpoint.total_stake);
    report.set(
        Stage::Signers,
        CheckStatus::Pass,
        Diagnostic::new(
            Code::Ok,
            bitmap_path.clone(),
            "Unique Base2 signer positions resolve in the selected immutable rank map",
        ),
    );
    let mut keys = Vec::with_capacity(ranks.len());
    for rank in ranks {
        let result = cache
            .entry(rank)
            .or_insert_with(|| qualify(&checkpoint.validators[rank as usize], registered));
        match result {
            Ok(key) => keys.push(*key),
            Err(error) => return report.stop(Stage::KeyQualification, error.clone()),
        }
    }
    if registered {
        report.set(
            Stage::KeyQualification,
            CheckStatus::Pass,
            Diagnostic::new(
                Code::RegisteredKeysAccepted,
                "$",
                "All selected keys passed checked point decoding. Their proofs of possession were not re-verified: under the registered-key trust you selected, each key is accepted as the one registered on chain, where the vote program verified its proof",
            ),
        );
    } else {
        report.pass(Stage::KeyQualification, "All selected keys passed checked point decoding and actual public-key-bound PoP verification");
    }
    let signature_path = bytes(&layout.signature);
    let signature = Signature(decoded.signature);
    // The checked conversion alone decides acceptance; `rejected_signature` only
    // explains a rejection it has already made.
    let affine = match SignatureAffine::try_from(&signature) {
        Ok(affine) => affine,
        Err(_) => {
            let (code, message) = rejected_signature(&signature);
            return report.stop(
                Stage::Signature,
                Diagnostic::new(code, signature_path, message),
            );
        }
    };
    if affine == SignatureAffine::from(SignatureProjective::identity()) {
        return report.stop(
            Stage::Signature,
            Diagnostic::new(
                Code::IdentitySignature,
                signature_path,
                "Signature is the identity point (point at infinity), which is never accepted",
            ),
        );
    }
    let aggregate = match PubkeyProjective::aggregate(keys.iter()) {
        Ok(aggregate) => aggregate,
        Err(_) => {
            return report.stop(
                Stage::Signature,
                Diagnostic::new(
                    Code::InvalidSignature,
                    signature_path,
                    "Qualified key aggregation failed",
                ),
            );
        }
    };
    if aggregate.verify_signature(&affine, &payload).is_err() {
        return report.stop(Stage::Signature, Diagnostic::new(Code::InvalidSignature, signature_path, "Signature is a valid G2 point but does not verify for the reconstructed signed payload and the selected signers' keys. BLS cannot tell which changed: the payload, the signer set, or the signature"));
    }
    report.set(
        Stage::Signature,
        CheckStatus::Pass,
        Diagnostic::new(
            Code::Ok,
            signature_path,
            "BLS aggregate verifies against the reconstructed source vote bytes",
        ),
    );
    let total = checkpoint.total_stake.get();
    let required = required_stake(total, decoded.kind);
    let percent = decoded.kind.threshold_percent();
    if !meets_threshold(stake, total, decoded.kind) {
        return report.stop(
            Stage::Quorum,
            Diagnostic::new(
                Code::InsufficientStake,
                bitmap_path,
                format!("Signers hold {stake} of {total} stake; {percent}% needs at least {required}. The signature itself is valid"),
            ),
        );
    }
    report.set(
        Stage::Quorum,
        CheckStatus::Pass,
        Diagnostic::new(
            Code::Ok,
            bitmap_path,
            format!("Signers hold {stake} of {total} stake, meeting the exact {percent}% threshold of {required}"),
        ),
    );
    report.conclusion = Conclusion::Verified;
    report
}

/// Verify each supplied certificate, preserving C2's explicit trust boundary.
/// Block-finality and migration conclusions are deferred to finality::verify_evidence.
pub fn verify_certificates(
    request_bytes: &[u8],
    trust: &TrustStore,
) -> CertificateVerificationReport {
    let prepared = preflight(request_bytes, trust);
    let mut report = CertificateVerificationReport {
        schema_version: 1,
        run_id: prepared.run_id,
        bundle_digest: prepared.bundle_digest,
        profile_id: prepared.profile_id,
        checkpoint_digest: prepared.checkpoint_digest,
        conclusion: prepared.conclusion,
        evidence_scope: "individual_certificates",
        checks: prepared.checks.into_iter().take(3).collect(),
        certificates: Vec::new(),
        assumptions: prepared.assumptions,
    };
    report.checks.push(check(Stage::Finality, CheckStatus::NotChecked, Code::FinalityNotEvaluated, "This certificate-only API does not evaluate block finality or genesis handover; use finality::verify_evidence"));
    if report
        .checks
        .iter()
        .take(3)
        .any(|check| check.status != CheckStatus::Pass)
    {
        return report;
    }
    if report.profile_id.as_deref() != Some(PROFILE_ID) {
        report.checks[1] = check(
            Stage::Profile,
            CheckStatus::NotChecked,
            Code::UnsupportedProfile,
            "Certificate verification requires the explicit Base2 profile, not the contracts-only profile",
        );
        return report;
    }
    // Reparse bounded immutable bytes after preparation; never accept a caller-created DTO as validated.
    let Ok(request) = parse_request(request_bytes) else {
        return report;
    };
    let Ok(bundle) = parse_bundle(request.document().bundle_json.as_bytes()) else {
        return report;
    };
    let Some(checkpoint) = request
        .document()
        .selected_checkpoint
        .as_ref()
        .and_then(|digest| trust.get(digest))
    else {
        return report;
    };
    let registered = checkpoint.basis.accepts_registered_keys();
    let mut cache = KeyCache::new();
    for (index, envelope) in bundle.document().certificates.iter().enumerate() {
        report.certificates.push(verify_one(
            index,
            envelope,
            bundle.document(),
            checkpoint.parsed.document(),
            registered,
            &mut cache,
        ));
    }
    report.assumptions.push(if registered {
        "PoP policy: registered-key trust. Selected keys pass checked point decoding but their proofs of possession are not re-verified; this relies on the checkpoint listing exactly the keys the vote program accepted (SIMD-0387), as Agave assumes.".into()
    } else {
        "PoP policy: verify each selected key's standard compressed-public-key-bound proof with no additional application payload. This does not authenticate the checkpoint or prove a vote-account registration.".into()
    });
    report.conclusion = if report
        .certificates
        .iter()
        .any(|cert| cert.conclusion == Conclusion::Rejected)
    {
        Conclusion::Rejected
    } else if report
        .certificates
        .iter()
        .any(|cert| cert.conclusion == Conclusion::Inconclusive)
        || bundle.document().claim.kind != "certificate_inspection"
    {
        Conclusion::Inconclusive
    } else {
        Conclusion::Verified
    };
    report
}

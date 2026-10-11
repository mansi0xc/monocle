use super::{
    CertificateReport, CertificateVerificationReport, PROFILE_ID, decode_certificate,
    decode_signers, meets_threshold, signed_payload,
};
use crate::*;
use solana_bls_signatures::{
    ProofOfPossession, Pubkey, PubkeyProjective, Signature, SignatureProjective, VerifyPop,
    VerifySignature,
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
fn unsupported(code: Code) -> bool {
    matches!(
        code,
        Code::UnsupportedWireVersion
            | Code::UnsupportedCertificate
            | Code::UnsupportedSignerEncoding
            | Code::ContextMismatch
            | Code::MissingPop
    )
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
        let incomplete = unsupported(error.code);
        self.conclusion = if incomplete {
            Conclusion::Inconclusive
        } else {
            Conclusion::Rejected
        };
        self.set(
            stage,
            if incomplete {
                CheckStatus::NotChecked
            } else {
                CheckStatus::Fail
            },
            error,
        );
        self
    }
}

fn qualify(entry: &ValidatorEntry) -> Result<PopVerified<PubkeyAffine>, Diagnostic> {
    let mut key_bytes = [0; 96];
    key_bytes.copy_from_slice(entry.bls_pubkey_hex.as_slice());
    let key = PubkeyAffine::try_from(&Pubkey(key_bytes)).map_err(|_| {
        Diagnostic::new(
            Code::InvalidPublicKey,
            format!("validators[{}].bls_pubkey_hex", entry.rank),
            "Public key is malformed, identity, off curve, or outside the prime-order subgroup",
        )
    })?;
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
                "Proof of possession failed for this selected public key",
            )
        })
}

fn verify_one(
    envelope: &CertificateEnvelope,
    bundle: &ProofBundle,
    checkpoint: &EpochCheckpoint,
    cache: &mut KeyCache,
) -> CertificateReport {
    let mut report = CertificateReport::new(envelope);
    let decoded = match decode_certificate(envelope.raw_hex.as_slice()) {
        Ok(cert) => cert,
        Err(error) => return report.stop(Stage::CertificateDecoding, error),
    };
    report.kind = Some(decoded.kind);
    report.slot = Some(DecimalU64::new(decoded.slot));
    report.block_id_hex = decoded.block_id.map(hex::encode);
    report.threshold_percent = Some(decoded.kind.threshold_percent() as u8);
    if decoded.kind.name() != envelope.kind {
        return report.stop(
            Stage::CertificateDecoding,
            Diagnostic::new(
                Code::EnvelopeMismatch,
                "kind",
                "Envelope kind contradicts the certificate's actual wire tag",
            ),
        );
    }
    if decoded.shred_version != checkpoint.context.shred_version {
        return report.stop(
            Stage::CertificateDecoding,
            Diagnostic::new(
                Code::ContextMismatch,
                "shred_version",
                "Wire message shred version differs from the selected checkpoint",
            ),
        );
    }
    if decoded.slot != bundle.claim.slot.get()
        || bundle
            .claim
            .block_id_hex
            .as_ref()
            .is_some_and(|claim| decoded.block_id.is_some_and(|id| claim.as_slice() != id))
    {
        return report.stop(
            Stage::CertificateDecoding,
            Diagnostic::new(
                Code::ClaimMismatch,
                "claim",
                "Decoded slot/block contradicts the declared claim",
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
    report.pass(
        Stage::CertificateDecoding,
        "Supported complete wire message and declared context agree",
    );
    let ranks = match decode_signers(&decoded.bitmap, checkpoint.validators.len()) {
        Ok(ranks) => ranks,
        Err(error) => return report.stop(Stage::Signers, error),
    };
    let mut stake = 0u64;
    for rank in &ranks {
        let Some(entry) = checkpoint.validators.get(*rank as usize) else {
            return report.stop(
                Stage::Signers,
                Diagnostic::new(
                    Code::UnknownSignerRank,
                    "bitmap",
                    "No entry for selected rank",
                ),
            );
        };
        let Some(sum) = stake.checked_add(entry.stake.get()) else {
            return report.stop(
                Stage::Signers,
                Diagnostic::new(
                    Code::StakeOverflow,
                    "stake",
                    "Selected stake accumulation overflow",
                ),
            );
        };
        stake = sum;
    }
    report.signer_ranks = ranks.clone();
    report.selected_stake = Some(DecimalU64::new(stake));
    report.total_stake = Some(checkpoint.total_stake);
    report.pass(
        Stage::Signers,
        "Unique Base2 signer positions resolve in the selected immutable rank map",
    );
    let mut keys = Vec::with_capacity(ranks.len());
    for rank in ranks {
        let result = cache
            .entry(rank)
            .or_insert_with(|| qualify(&checkpoint.validators[rank as usize]));
        match result {
            Ok(key) => keys.push(*key),
            Err(error) => return report.stop(Stage::KeyQualification, error.clone()),
        }
    }
    report.pass(Stage::KeyQualification, "All selected keys passed checked point decoding and actual public-key-bound PoP verification");
    let signature = Signature(decoded.signature);
    let affine = match SignatureAffine::try_from(&signature) {
        Ok(affine) => affine,
        Err(_) => {
            return report.stop(
                Stage::Signature,
                Diagnostic::new(
                    Code::InvalidSignature,
                    "signature",
                    "Signature is malformed, off curve, or outside the prime-order subgroup",
                ),
            );
        }
    };
    if affine == SignatureAffine::from(SignatureProjective::identity()) {
        return report.stop(
            Stage::Signature,
            Diagnostic::new(
                Code::InvalidSignature,
                "signature",
                "Identity signature is not accepted",
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
                    "signature",
                    "Qualified key aggregation failed",
                ),
            );
        }
    };
    if aggregate.verify_signature(&affine, &payload).is_err() {
        return report.stop(Stage::Signature, Diagnostic::new(Code::InvalidSignature, "signature", "Aggregate signature does not verify for the exact signed payload and selected keys"));
    }
    report.pass(
        Stage::Signature,
        "BLS aggregate verifies against the reconstructed source vote bytes",
    );
    if !meets_threshold(stake, checkpoint.total_stake.get(), decoded.kind) {
        return report.stop(
            Stage::Quorum,
            Diagnostic::new(
                Code::InsufficientStake,
                "stake",
                "Valid signature has insufficient stake under the original checkpoint denominator",
            ),
        );
    }
    report.pass(Stage::Quorum, "Exact integer stake threshold is met");
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
    let mut cache = KeyCache::new();
    for envelope in &bundle.document().certificates {
        report.certificates.push(verify_one(
            envelope,
            bundle.document(),
            checkpoint.parsed.document(),
            &mut cache,
        ));
    }
    report.assumptions.push("PoP policy: verify each selected key's standard compressed-public-key-bound proof with no additional application payload. This does not authenticate the checkpoint or prove a vote-account registration.".into());
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

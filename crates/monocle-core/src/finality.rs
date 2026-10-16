//! Direct evidence composition under a fixed checkpoint. Reports are outputs,
//! never inputs: the public API always reruns the certificate verifier on bytes.
use crate::{certificate::*, *};
use serde::Serialize;

pub const POLICY_ID: &str = "agave-98535e50-direct-finality-v1";

listed_enum! {
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimKind {
    CertificateInspection,
    BlockFinality,
    GenesisMigration,
}
}
listed_enum! {
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceScope {
    Unestablished,
    IndividualCertificates,
    BlockFinality,
    GenesisMigration,
}
}
listed_enum! {
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FinalityPath {
    Fast,
    Slow,
}
}
#[derive(Debug, Serialize)]
pub struct EvaluatedClaim {
    pub kind: ClaimKind,
    pub slot: DecimalU64,
    pub block_id_hex: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct EvidenceVerificationReport {
    pub schema_version: u16,
    pub policy_id: &'static str,
    pub run_id: Option<String>,
    pub bundle_digest: Option<ContentDigest>,
    pub profile_id: Option<String>,
    pub checkpoint_digest: Option<ContentDigest>,
    pub conclusion: Conclusion,
    /// Established scope, never the requested scope when evidence is incomplete.
    pub evidence_scope: EvidenceScope,
    pub claim: Option<EvaluatedClaim>,
    pub finality_path: Option<FinalityPath>,
    /// Deterministic witness IDs. All supplied certificates are still checked.
    pub supporting_certificate_ids: Vec<String>,
    pub checks: Vec<CheckResult>,
    pub certificates: Vec<CertificateReport>,
    pub assumptions: Vec<String>,
}
impl EvidenceVerificationReport {
    fn finish_check(&mut self, status: CheckStatus, code: Code, message: &str) {
        self.checks.push(CheckResult {
            stage: Stage::Finality,
            status,
            diagnostic: Diagnostic::new(code, "claim", message),
        });
    }
    fn not_checked(mut self, code: Code, message: &str) -> Self {
        self.finish_check(CheckStatus::NotChecked, code, message);
        self
    }
    fn reject(mut self, code: Code, message: &str) -> Self {
        self.conclusion = Conclusion::Rejected;
        self.finish_check(CheckStatus::Fail, code, message);
        self
    }
    fn establish(&mut self, scope: EvidenceScope, ids: Vec<String>) {
        self.conclusion = Conclusion::Verified;
        self.evidence_scope = scope;
        self.supporting_certificate_ids = ids;
    }
    fn first_id(&self, kind: CertificateKind) -> Option<String> {
        self.certificates
            .iter()
            .filter(|cert| cert.kind == Some(kind))
            .map(|cert| &cert.certificate_id)
            .min()
            .cloned()
    }
}

/// Check original evidence and compose only direct supported paths. Never takes
/// caller-constructed reports or accepts saved VERIFIED values as proof.
pub fn verify_evidence(request_bytes: &[u8], trust: &TrustStore) -> EvidenceVerificationReport {
    let individual = verify_certificates(request_bytes, trust);
    let mut report = EvidenceVerificationReport {
        schema_version: 1,
        policy_id: POLICY_ID,
        run_id: individual.run_id,
        bundle_digest: individual.bundle_digest,
        profile_id: individual.profile_id,
        checkpoint_digest: individual.checkpoint_digest,
        conclusion: if individual.conclusion == Conclusion::Rejected {
            Conclusion::Rejected
        } else {
            Conclusion::Inconclusive
        },
        evidence_scope: EvidenceScope::Unestablished,
        claim: None,
        finality_path: None,
        supporting_certificate_ids: Vec::new(),
        checks: individual
            .checks
            .into_iter()
            .filter(|check| check.stage != Stage::Finality)
            .collect(),
        certificates: individual.certificates,
        assumptions: individual.assumptions,
    };
    report.assumptions.push("Direct evidence only, conditional on the pinned protocol's consensus safety assumptions and the selected checkpoint. No ancestry, execution, epoch-transition, or network-activation proof is evaluated. REJECTED describes supplied evidence, not proof that a block is non-final.".into());
    if report.checks.len() != 3
        || report
            .checks
            .iter()
            .any(|check| check.status != CheckStatus::Pass)
    {
        return report.not_checked(Code::PrerequisiteMissing, "Input, profile, and independently selected context must pass before evidence composition");
    }
    // Bounded immutable bytes already passed preparation. Reparse locally rather
    // than exposing a composable, caller-forgeable certificate report API.
    let Ok(request) = parse_request(request_bytes) else {
        return report.not_checked(Code::PrerequisiteMissing, "Request could not be resolved");
    };
    let Ok(bundle) = parse_bundle(request.document().bundle_json.as_bytes()) else {
        return report.not_checked(Code::PrerequisiteMissing, "Bundle could not be resolved");
    };
    let claim = &bundle.document().claim;
    let kind = match claim.kind.as_str() {
        "certificate_inspection" => ClaimKind::CertificateInspection,
        "block_finality" => ClaimKind::BlockFinality,
        "genesis_migration" => ClaimKind::GenesisMigration,
        _ => {
            return report.not_checked(
                Code::UnsupportedClaim,
                "No composition policy exists for this claim kind",
            );
        }
    };
    report.claim = Some(EvaluatedClaim {
        kind,
        slot: claim.slot,
        block_id_hex: claim
            .block_id_hex
            .as_ref()
            .map(|id| hex::encode(id.as_slice())),
    });
    if report
        .certificates
        .iter()
        .any(|cert| cert.conclusion == Conclusion::Rejected)
    {
        report.conclusion = Conclusion::Rejected;
        return report.not_checked(Code::CertificateChecksFailed, "A selected certificate failed; inspect its check trace. Finality composition was not performed");
    }
    if report.certificates.len() != bundle.document().certificates.len()
        || report.certificates.is_empty()
        || report
            .certificates
            .iter()
            .any(|cert| cert.conclusion != Conclusion::Verified)
    {
        return report.not_checked(
            Code::CertificateChecksIncomplete,
            "Every selected certificate must pass under the same checkpoint before composition",
        );
    }
    if kind == ClaimKind::CertificateInspection {
        let mut ids: Vec<_> = report
            .certificates
            .iter()
            .map(|cert| cert.certificate_id.clone())
            .collect();
        ids.sort();
        report.establish(EvidenceScope::IndividualCertificates, ids);
        return report.not_checked(Code::FinalityNotRequested, "Individual certificate inspection succeeded; no finality or migration conclusion was requested");
    }
    if claim.block_id_hex.is_none() {
        return report.not_checked(Code::MissingBlockId, "Specify the block ID being claimed; a slot alone does not identify a block or migration anchor");
    }
    // Every verified certificate already agrees with this claim's slot and,
    // where signed, block ID; wire shred/context also agrees with one checkpoint.
    match kind {
        ClaimKind::BlockFinality => {
            if report.first_id(CertificateKind::Genesis).is_some() {
                return report.reject(Code::IncompatibleEvidence, "Genesis evidence belongs to the migration scope and cannot be selected for an ordinary block-finality claim");
            }
            if let Some(fast) = report.first_id(CertificateKind::FastFinalization) {
                report.establish(EvidenceScope::BlockFinality, vec![fast]);
                report.finality_path = Some(FinalityPath::Fast);
                report.finish_check(CheckStatus::Pass, Code::FastFinalityEstablished, "A valid fast-finalization certificate binds this block and meets the 80% threshold");
            } else {
                let Some(notar) = report.first_id(CertificateKind::Notarization) else {
                    return report.not_checked(Code::MissingNotarization, "A slot-only finalize certificate needs matching notarization to identify the block; no fast certificate is present");
                };
                let Some(finalize) = report.first_id(CertificateKind::Finalization) else {
                    return report.not_checked(Code::MissingFinalization, "Notarization alone is insufficient; provide matching finalization or a supported fast-finalization certificate");
                };
                report.establish(EvidenceScope::BlockFinality, vec![notar, finalize]);
                report.finality_path = Some(FinalityPath::Slow);
                report.finish_check(CheckStatus::Pass, Code::SlowFinalityEstablished, "Notarization and slot-only finalization each meet 60%, agree on slot/context, and notarization binds the claimed block. Their stakes are not added");
            }
        }
        ClaimKind::GenesisMigration => {
            if report
                .certificates
                .iter()
                .any(|cert| cert.kind != Some(CertificateKind::Genesis))
            {
                return report.reject(Code::IncompatibleEvidence, "This migration-anchor claim requires genesis certificates; ordinary finality evidence has a separate scope");
            }
            let Some(genesis) = report.first_id(CertificateKind::Genesis) else {
                return report.not_checked(
                    Code::MissingGenesis,
                    "No valid genesis certificate identifies this migration anchor",
                );
            };
            report.establish(EvidenceScope::GenesisMigration, vec![genesis]);
            report.assumptions.push("Genesis migration scope establishes the certificate's anchor only. It does not prove that a node or the cluster completed migration, feature activation, or the TowerBFT transition.".into());
            report.finish_check(CheckStatus::Pass, Code::GenesisAnchorEstablished, "A valid genesis certificate binds the claimed migration anchor and meets 82%; this is not ordinary block finality");
        }
        ClaimKind::CertificateInspection => {
            return report.not_checked(
                Code::PrerequisiteMissing,
                "No composition rule was selected",
            );
        }
    }
    report
}

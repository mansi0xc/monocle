use crate::{Bytes, ContentDigest, DecimalU64, Diagnostic};
use serde::{Deserialize, Serialize};

pub const PROFILE_ID: &str = "agave-98535e50-contracts-v1";
pub const AGAVE_REVISION: &str = "98535e50e9bb38574e6b0c31ff284cd864d3ca88";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChainContext {
    pub genesis_hash_hex: Bytes<32, 32>,
    pub epoch: DecimalU64,
    pub shred_version: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub kind: SourceKind,
    pub description: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Synthetic,
    Capture,
}

/// All descriptive fields are claims until a later decoder verifies their binding.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CertificateEnvelope {
    pub schema_version: u16,
    pub id: String,
    pub kind: String,
    pub format: String,
    pub raw_hex: Bytes<1, 65536>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceClaim {
    pub kind: String,
    pub slot: DecimalU64,
    pub block_id_hex: Option<Bytes<32, 32>>,
    pub certificate_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationRecord {
    pub schema_version: u16,
    pub certificate_id: String,
    pub operation: String,
    pub offset: DecimalU64,
    pub before_hex: Bytes<1, 4096>,
    pub after_hex: Bytes<1, 4096>,
    pub parent_certificate_digest: ContentDigest,
    pub result_certificate_digest: ContentDigest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofBundle {
    pub schema_version: u16,
    pub profile_id: String,
    pub context: ChainContext,
    pub claim: EvidenceClaim,
    pub certificates: Vec<CertificateEnvelope>,
    pub checkpoint_reference: Option<ContentDigest>,
    pub provenance: Provenance,
    pub parent_digest: Option<ContentDigest>,
    pub mutations: Vec<MutationRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidatorEntry {
    pub rank: u32,
    pub identity_hex: Bytes<32, 32>,
    pub bls_pubkey_hex: Bytes<96, 96>,
    pub stake: DecimalU64,
    pub proof_of_possession_hex: Option<Bytes<192, 192>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpochCheckpoint {
    pub schema_version: u16,
    pub profile_id: String,
    pub context: ChainContext,
    pub total_stake: DecimalU64,
    pub validators: Vec<ValidatorEntry>,
    pub provenance: Provenance,
}

/// Supplied by the local controller. The selected digest is resolved only in TrustStore.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifyRequest {
    pub schema_version: u16,
    pub run_id: String,
    pub profile_id: String,
    pub bundle_json: String,
    pub selected_checkpoint: Option<ContentDigest>,
}

/// Compiled-in policy metadata; never loaded from a ProofBundle.
#[derive(Clone, Debug, Serialize)]
pub struct ProtocolProfile {
    pub schema_version: u16,
    pub id: &'static str,
    pub agave_revision: &'static str,
    pub capability: &'static str,
    pub recognized_formats: &'static [&'static str],
    pub recognized_certificate_kinds: &'static [&'static str],
    pub recognized_claim_kinds: &'static [&'static str],
}
pub fn protocol_profile() -> ProtocolProfile {
    ProtocolProfile {
        schema_version: 1,
        id: PROFILE_ID,
        agave_revision: AGAVE_REVISION,
        capability: "contracts_only",
        recognized_formats: &["agave-certificate-wincode"],
        recognized_certificate_kinds: &[
            "notarization",
            "finalization",
            "fast_finalization",
            "genesis",
        ],
        recognized_claim_kinds: &[
            "certificate_inspection",
            "block_finality",
            "genesis_migration",
        ],
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Conclusion {
    Verified,
    Rejected,
    Inconclusive,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckStatus {
    Pass,
    Fail,
    NotChecked,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Input,
    Profile,
    Context,
    CertificateDecoding,
    Signers,
    KeyQualification,
    Signature,
    Quorum,
    Finality,
}
#[derive(Clone, Debug, Serialize)]
pub struct CheckResult {
    pub stage: Stage,
    pub status: CheckStatus,
    pub diagnostic: Diagnostic,
}
#[derive(Clone, Debug, Serialize)]
pub struct VerificationReport {
    pub schema_version: u16,
    pub run_id: Option<String>,
    pub bundle_digest: Option<ContentDigest>,
    pub profile_id: Option<String>,
    pub checkpoint_digest: Option<ContentDigest>,
    pub conclusion: Conclusion,
    pub evidence_scope: &'static str,
    pub checks: Vec<CheckResult>,
    pub assumptions: Vec<String>,
}

mod signers;
mod verify;
mod wire;
use crate::{AGAVE_REVISION, CheckResult, Conclusion, ContentDigest, DecimalU64, ProtocolProfile};
use serde::Serialize;
pub use signers::decode_signers;
pub use verify::verify_certificates;
pub use wire::{DecodedCertificate, decode_certificate, signed_payload};

pub const PROFILE_ID: &str = "agave-98535e50-base2-v1";
pub const WIRE_FORMAT: &str = "agave-versioned-consensus-v1";
pub fn protocol_profile() -> ProtocolProfile {
    ProtocolProfile {
        schema_version: 1,
        id: PROFILE_ID,
        agave_revision: AGAVE_REVISION,
        capability: "base2_certificate_verification",
        recognized_formats: &[WIRE_FORMAT],
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

listed_enum! {
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificateKind {
    Notarization,
    Finalization,
    FastFinalization,
    Genesis,
}
}
impl CertificateKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Notarization => "notarization",
            Self::Finalization => "finalization",
            Self::FastFinalization => "fast_finalization",
            Self::Genesis => "genesis",
        }
    }
    pub fn threshold_percent(self) -> u64 {
        match self {
            Self::FastFinalization => 80,
            Self::Genesis => 82,
            _ => 60,
        }
    }
}

/// Exact comparison; no floating-point rounding and no overflowing u64 products.
pub fn meets_threshold(signed: u64, total: u64, kind: CertificateKind) -> bool {
    total != 0
        && signed <= total
        && u128::from(signed) * 100 >= u128::from(total) * u128::from(kind.threshold_percent())
}

#[derive(Debug, Serialize)]
pub struct CertificateReport {
    pub certificate_id: String,
    pub raw_digest: ContentDigest,
    pub conclusion: Conclusion,
    pub kind: Option<CertificateKind>,
    pub slot: Option<DecimalU64>,
    pub block_id_hex: Option<String>,
    pub signed_payload_hex: Option<String>,
    pub signer_ranks: Vec<u32>,
    pub selected_stake: Option<DecimalU64>,
    pub total_stake: Option<DecimalU64>,
    pub threshold_percent: Option<u8>,
    pub checks: Vec<CheckResult>,
}

/// Separate versioned output from C2's preflight contract. VERIFIED is scoped to
/// individual certificate checks; use finality::verify_evidence for composition.
#[derive(Debug, Serialize)]
pub struct CertificateVerificationReport {
    pub schema_version: u16,
    pub run_id: Option<String>,
    pub bundle_digest: Option<ContentDigest>,
    pub profile_id: Option<String>,
    pub checkpoint_digest: Option<ContentDigest>,
    pub conclusion: Conclusion,
    pub evidence_scope: &'static str,
    pub checks: Vec<CheckResult>,
    pub certificates: Vec<CertificateReport>,
    pub assumptions: Vec<String>,
}

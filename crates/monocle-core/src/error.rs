use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Code {
    InputTooLarge,
    MalformedJson,
    DuplicateField,
    StructureLimit,
    InvalidSchema,
    InvalidInteger,
    InvalidHex,
    UnknownField,
    UnsupportedVersion,
    LimitExceeded,
    DuplicateId,
    InvalidRankMap,
    InvalidStake,
    InvalidMutation,
    UnsupportedProfile,
    ProfileMismatch,
    UnsupportedCertificate,
    MissingCheckpoint,
    CheckpointNotTrusted,
    CheckpointMismatch,
    ContextMismatch,
    VerifierNotImplemented,
    PrerequisiteMissing,
    MalformedCertificate,
    UnsupportedWireVersion,
    EnvelopeMismatch,
    ClaimMismatch,
    UnsupportedSignerEncoding,
    MalformedBitmap,
    UnknownSignerRank,
    EmptySignerSet,
    InvalidPublicKey,
    MissingPop,
    InvalidPop,
    InvalidSignature,
    InsufficientStake,
    StakeOverflow,
    FinalityNotEvaluated,
    UnsupportedClaim,
    CertificateChecksFailed,
    CertificateChecksIncomplete,
    FinalityNotRequested,
    MissingBlockId,
    MissingNotarization,
    MissingFinalization,
    MissingGenesis,
    IncompatibleEvidence,
    FastFinalityEstablished,
    SlowFinalityEstablished,
    GenesisAnchorEstablished,
    Ok,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: Code,
    pub path: String,
    pub message: String,
}
impl Diagnostic {
    pub(crate) fn new(code: Code, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code,
            path: path.into(),
            message: message.into(),
        }
    }
    pub(crate) fn from_serde(error: &str, path: impl Into<String>, fallback: Code) -> Self {
        let code = if error.starts_with("MONOCLE_DUPLICATE") {
            Code::DuplicateField
        } else if error.starts_with("MONOCLE_STRUCTURE") {
            Code::StructureLimit
        } else if error.starts_with("MONOCLE_INTEGER") {
            Code::InvalidInteger
        } else if error.starts_with("MONOCLE_HEX") {
            Code::InvalidHex
        } else if error.starts_with("unknown field") {
            Code::UnknownField
        } else {
            fallback
        };
        // Never echo a potentially enormous attacker-controlled field or value.
        Self::new(
            code,
            path.into().chars().take(256).collect::<String>(),
            error.chars().take(256).collect::<String>(),
        )
    }
}

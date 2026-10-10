use crate::{Code, ContentDigest, Diagnostic, EpochCheckpoint, Parsed, SourceKind};
use std::collections::HashMap;

/// A local caller decision, deliberately not deserializable from evidence JSON.
#[derive(Clone, Debug)]
pub enum TrustBasis {
    SyntheticFixture,
    OperatorPinned { reason: String },
}
#[derive(Debug)]
pub(crate) struct TrustedCheckpoint {
    pub parsed: Parsed<EpochCheckpoint>,
    pub basis: TrustBasis,
}
/// Session-local registry. Parsing, labels, and checkpoint references never populate it.
#[derive(Default, Debug)]
pub struct TrustStore {
    checkpoints: HashMap<ContentDigest, TrustedCheckpoint>,
}
impl TrustStore {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn pin(
        &mut self,
        checkpoint: Parsed<EpochCheckpoint>,
        basis: TrustBasis,
    ) -> Result<ContentDigest, Diagnostic> {
        match &basis {
            TrustBasis::SyntheticFixture
                if checkpoint.document().provenance.kind != SourceKind::Synthetic =>
            {
                return Err(Diagnostic::new(
                    Code::InvalidSchema,
                    "trust_basis",
                    "Synthetic selection requires a synthetic checkpoint",
                ));
            }
            TrustBasis::OperatorPinned { reason }
                if reason.trim().is_empty() || reason.len() > 1024 =>
            {
                return Err(Diagnostic::new(
                    Code::InvalidSchema,
                    "trust_basis",
                    "Record a nonempty local trust reason, at most 1024 UTF-8 bytes",
                ));
            }
            _ => {}
        }
        let digest = checkpoint.digest().clone();
        self.checkpoints.insert(
            digest.clone(),
            TrustedCheckpoint {
                parsed: checkpoint,
                basis,
            },
        );
        Ok(digest)
    }
    pub fn remove(&mut self, digest: &ContentDigest) {
        self.checkpoints.remove(digest);
    }
    pub(crate) fn get(&self, digest: &ContentDigest) -> Option<&TrustedCheckpoint> {
        self.checkpoints.get(digest)
    }
}

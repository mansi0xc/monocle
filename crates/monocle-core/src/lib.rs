//! Versioned evidence contracts and preflight checks. No certificate verification yet.
mod error;
mod json;
mod model;
mod parse;
mod preflight;
mod scalar;
mod trust;

pub use error::{Code, Diagnostic};
pub use model::*;
pub use parse::{Parsed, parse_bundle, parse_checkpoint, parse_request};
pub use preflight::preflight;
pub use scalar::{Bytes, ContentDigest, DecimalU64};
pub use trust::{TrustBasis, TrustStore};

pub const MAX_BUNDLE_BYTES: usize = 1024 * 1024;
pub const MAX_CHECKPOINT_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_REQUEST_BYTES: usize = 6 * MAX_BUNDLE_BYTES + 4096;
pub const MAX_CERTIFICATES: usize = 16;
pub const MAX_VALIDATORS: usize = 2048;
pub const MAX_MUTATIONS: usize = 64;

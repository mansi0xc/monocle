//! Bounded adapter for the pinned VersionedWireConsensusMessage V1 schema.
//! Layout adapted from Agave's Apache-2.0 wire.rs; see docs/certificate-verification.md.
use super::CertificateKind;
use crate::{Code, Diagnostic, MAX_VALIDATORS};

#[derive(Clone, Debug)]
pub struct DecodedCertificate {
    pub kind: CertificateKind,
    pub slot: u64,
    pub block_id: Option<[u8; 32]>,
    pub signature: [u8; 192],
    pub bitmap: Vec<u8>,
    pub shred_version: u16,
}
struct Cursor<'a> {
    remaining: &'a [u8],
}
impl Cursor<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], Diagnostic> {
        if self.remaining.len() < N {
            return Err(malformed("Truncated certificate field"));
        }
        let (head, tail) = self.remaining.split_at(N);
        self.remaining = tail;
        // Length is established above; copy avoids a fallible conversion/panic.
        let mut bytes = [0; N];
        bytes.copy_from_slice(head);
        Ok(bytes)
    }
}
fn malformed(message: &str) -> Diagnostic {
    Diagnostic::new(Code::MalformedCertificate, "raw_hex", message)
}

/// Decode one whole message; transport prefixes, footers, RPC JSON and trailers are not accepted.
pub fn decode_certificate(bytes: &[u8]) -> Result<DecodedCertificate, Diagnostic> {
    let mut cursor = Cursor { remaining: bytes };
    if cursor.take::<1>()?[0] != 1 {
        return Err(Diagnostic::new(
            Code::UnsupportedWireVersion,
            "raw_hex",
            "Unsupported versioned consensus message",
        ));
    }
    let kind = match cursor.take::<1>()?[0] {
        7 => CertificateKind::Finalization,
        8 => CertificateKind::FastFinalization,
        9 => CertificateKind::Notarization,
        12 => CertificateKind::Genesis,
        _ => {
            return Err(Diagnostic::new(
                Code::UnsupportedCertificate,
                "raw_hex",
                "Vote, fallback, skip, or unknown wire tag is unsupported",
            ));
        }
    };
    let slot = u64::from_le_bytes(cursor.take()?);
    let block_id = if kind == CertificateKind::Finalization {
        None
    } else {
        Some(cursor.take()?)
    };
    let signature = cursor.take()?;
    // wincode DefaultConfig uses a little-endian u64 sequence length.
    let bitmap_len = u64::from_le_bytes(cursor.take()?);
    // Bound before conversion or allocation on both 32- and 64-bit targets.
    if bitmap_len > (3 + MAX_VALIDATORS.div_ceil(5)) as u64 {
        return Err(malformed(
            "Bitmap exceeds this profile's bounded input size",
        ));
    }
    let bitmap_len = bitmap_len as usize;
    if cursor.remaining.len() != bitmap_len + 2 {
        return Err(malformed(
            "Bitmap length, shred suffix, or trailing bytes do not match",
        ));
    }
    let bitmap = cursor.remaining[..bitmap_len].to_vec();
    cursor.remaining = &cursor.remaining[bitmap_len..];
    let shred_version = u16::from_le_bytes(cursor.take()?);
    Ok(DecodedCertificate {
        kind,
        slot,
        block_id,
        signature,
        bitmap,
        shred_version,
    })
}

/// Source vote bytes are distinct from the containing certificate's wire bytes.
pub fn signed_payload(cert: &DecodedCertificate) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(43);
    bytes.push(match cert.kind {
        CertificateKind::Finalization => 2,
        CertificateKind::Genesis => 6,
        _ => 1,
    });
    bytes.extend_from_slice(&cert.slot.to_le_bytes());
    if let Some(block_id) = cert.block_id {
        bytes.extend_from_slice(&block_id);
    }
    bytes.extend_from_slice(&cert.shred_version.to_le_bytes());
    bytes
}

//! Bounded adapter for the pinned VersionedWireConsensusMessage V1 schema.
//! Layout adapted from Agave's Apache-2.0 wire.rs; see docs/certificate-verification.md.
use super::CertificateKind;
use crate::{Code, Diagnostic, MAX_VALIDATORS};
use serde::Serialize;
use std::ops::Range;

/// Half-open byte ranges of each field within the certificate's raw bytes, so a
/// diagnostic can name the exact bytes behind a check. Reports carry it so a
/// viewer labels fields from the decoder itself; each range serializes as
/// `{start, end}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WireLayout {
    pub version: Range<usize>,
    pub kind: Range<usize>,
    pub slot: Range<usize>,
    pub block_id: Option<Range<usize>>,
    pub signature: Range<usize>,
    pub bitmap_length: Range<usize>,
    pub bitmap: Range<usize>,
    pub shred_version: Range<usize>,
}

#[derive(Clone, Debug)]
pub struct DecodedCertificate {
    pub kind: CertificateKind,
    pub slot: u64,
    pub block_id: Option<[u8; 32]>,
    pub signature: [u8; 192],
    pub bitmap: Vec<u8>,
    pub shred_version: u16,
    pub layout: WireLayout,
}

/// A failure located at a half-open byte range of the bytes being decoded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ByteFault {
    pub code: Code,
    pub bytes: Range<usize>,
    pub message: String,
}
impl ByteFault {
    pub(crate) fn new(code: Code, bytes: Range<usize>, message: impl Into<String>) -> Self {
        Self {
            code,
            bytes,
            message: message.into(),
        }
    }
    /// Locate the fault in a document field whose bytes start `offset` bytes
    /// before the fault's own range, e.g. `certificates[0].raw_hex[42..234]`.
    pub fn at(self, field: &str, offset: usize) -> Diagnostic {
        let bytes = offset + self.bytes.start..offset + self.bytes.end;
        Diagnostic::new(self.code, byte_path(field, &bytes), self.message)
    }
}
/// Byte offsets index the decoded bytes, not hex characters.
pub fn byte_path(field: &str, bytes: &Range<usize>) -> String {
    format!("{field}[{}..{}]", bytes.start, bytes.end)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl Cursor<'_> {
    fn take<const N: usize>(&mut self, field: &str) -> Result<([u8; N], Range<usize>), ByteFault> {
        let start = self.offset;
        let Some(head) = self.bytes.get(start..start + N) else {
            let remaining = self.bytes.len() - start;
            return Err(ByteFault::new(
                Code::MalformedCertificate,
                start..self.bytes.len(),
                format!("Truncated {field}: needs {N} bytes at offset {start}, {remaining} remain"),
            ));
        };
        // Length is established above; copy avoids a fallible conversion/panic.
        let mut bytes = [0; N];
        bytes.copy_from_slice(head);
        self.offset += N;
        Ok((bytes, start..self.offset))
    }
}

/// Decode one whole message; transport prefixes, footers, RPC JSON and trailers are not accepted.
pub fn decode_certificate(bytes: &[u8]) -> Result<DecodedCertificate, ByteFault> {
    let mut cursor = Cursor { bytes, offset: 0 };
    let ([version], version_at) = cursor.take::<1>("wire version")?;
    if version != 1 {
        return Err(ByteFault::new(
            Code::UnsupportedWireVersion,
            version_at,
            format!(
                "Versioned consensus message {version} is unsupported; this profile reads version 1"
            ),
        ));
    }
    let ([tag], kind_at) = cursor.take::<1>("certificate tag")?;
    let kind = match tag {
        7 => CertificateKind::Finalization,
        8 => CertificateKind::FastFinalization,
        9 => CertificateKind::Notarization,
        12 => CertificateKind::Genesis,
        _ => {
            return Err(ByteFault::new(
                Code::UnsupportedCertificate,
                kind_at,
                format!(
                    "Wire tag {tag} is a vote, fallback, skip, or unknown message; this profile reads certificate tags 7, 8, 9 and 12"
                ),
            ));
        }
    };
    let (slot, slot_at) = cursor.take("slot")?;
    let (block_id, block_id_at) = if kind == CertificateKind::Finalization {
        (None, None)
    } else {
        let (id, at) = cursor.take("block ID")?;
        (Some(id), Some(at))
    };
    let (signature, signature_at) = cursor.take("signature")?;
    // wincode DefaultConfig uses a little-endian u64 sequence length.
    let (length, bitmap_length_at) = cursor.take("bitmap length")?;
    let bitmap_len = u64::from_le_bytes(length);
    // Bound before conversion or allocation on both 32- and 64-bit targets.
    let bound = 3 + MAX_VALIDATORS.div_ceil(5);
    if bitmap_len > bound as u64 {
        return Err(ByteFault::new(
            Code::MalformedCertificate,
            bitmap_length_at,
            format!(
                "Declared bitmap length {bitmap_len} exceeds this profile's bound of {bound} bytes"
            ),
        ));
    }
    let bitmap_len = bitmap_len as usize;
    let start = cursor.offset;
    let remaining = bytes.len() - start;
    if remaining < bitmap_len + 2 {
        return Err(ByteFault::new(
            Code::MalformedCertificate,
            start..bytes.len(),
            format!(
                "Truncated bitmap and shred version: the declared {bitmap_len}-byte bitmap needs {} more bytes, {remaining} remain",
                bitmap_len + 2
            ),
        ));
    }
    if remaining > bitmap_len + 2 {
        let end = start + bitmap_len + 2;
        return Err(ByteFault::new(
            Code::MalformedCertificate,
            end..bytes.len(),
            format!(
                "{} trailing bytes follow the shred version; a complete message ends there",
                bytes.len() - end
            ),
        ));
    }
    let bitmap_at = start..start + bitmap_len;
    let bitmap = bytes[bitmap_at.clone()].to_vec();
    cursor.offset = bitmap_at.end;
    let (shred_version, shred_version_at) = cursor.take("shred version")?;
    Ok(DecodedCertificate {
        kind,
        slot: u64::from_le_bytes(slot),
        block_id,
        signature,
        bitmap,
        shred_version: u16::from_le_bytes(shred_version),
        layout: WireLayout {
            version: version_at,
            kind: kind_at,
            slot: slot_at,
            block_id: block_id_at,
            signature: signature_at,
            bitmap_length: bitmap_length_at,
            bitmap: bitmap_at,
            shred_version: shred_version_at,
        },
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

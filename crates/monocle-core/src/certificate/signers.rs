use crate::{Code, Diagnostic};
use solana_signer_store::{Decoded, decode};

pub fn decode_signers(bitmap: &[u8], validators: usize) -> Result<Vec<u32>, Diagnostic> {
    if bitmap.len() < 3 {
        return Err(Diagnostic::new(
            Code::MalformedBitmap,
            "bitmap",
            "Missing signer-store header",
        ));
    }
    if bitmap[0] != 0 {
        return Err(Diagnostic::new(
            Code::UnsupportedSignerEncoding,
            "bitmap",
            "This profile supports Base2 only; no Base3/fallback interpretation",
        ));
    }
    let bits = u16::from_le_bytes([bitmap[1], bitmap[2]]) as usize;
    if bits > validators {
        return Err(Diagnostic::new(
            Code::UnknownSignerRank,
            "bitmap",
            "Declared signer ranks exceed the selected checkpoint",
        ));
    }
    let decoded = decode(bitmap, validators).map_err(|_| {
        Diagnostic::new(
            Code::MalformedBitmap,
            "bitmap",
            "Invalid Base2 payload length",
        )
    })?;
    // Upstream truncates unused bits. This profile deliberately rejects nonzero padding.
    if !bits.is_multiple_of(8) && bitmap.last().is_some_and(|last| last >> (bits % 8) != 0) {
        return Err(Diagnostic::new(
            Code::MalformedBitmap,
            "bitmap",
            "Nonzero unused Base2 padding bits are not canonical",
        ));
    }
    let Decoded::Base2(bits) = decoded else {
        return Err(Diagnostic::new(
            Code::UnsupportedSignerEncoding,
            "bitmap",
            "Expected Base2",
        ));
    };
    let ranks: Vec<u32> = bits.iter_ones().map(|rank| rank as u32).collect();
    if ranks.is_empty() {
        return Err(Diagnostic::new(
            Code::EmptySignerSet,
            "bitmap",
            "An empty signer set cannot certify anything",
        ));
    }
    Ok(ranks)
}

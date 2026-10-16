use super::wire::ByteFault;
use crate::Code;
use solana_signer_store::{Decoded, decode};

/// Byte ranges in any fault are relative to the start of `bitmap`.
pub fn decode_signers(bitmap: &[u8], validators: usize) -> Result<Vec<u32>, ByteFault> {
    let len = bitmap.len();
    if len < 3 {
        return Err(ByteFault::new(
            Code::MalformedBitmap,
            0..len,
            format!("Bitmap is {len} bytes; the signer-store header alone needs 3"),
        ));
    }
    if bitmap[0] != 0 {
        return Err(ByteFault::new(
            Code::UnsupportedSignerEncoding,
            0..1,
            format!(
                "Signer encoding {} is not Base2 (0); this profile has no Base3 or fallback interpretation",
                bitmap[0]
            ),
        ));
    }
    let bits = u16::from_le_bytes([bitmap[1], bitmap[2]]) as usize;
    if bits > validators {
        return Err(ByteFault::new(
            Code::UnknownSignerRank,
            1..3,
            format!(
                "Bitmap declares {bits} signer positions; the selected checkpoint has {validators}"
            ),
        ));
    }
    let decoded = decode(bitmap, validators).map_err(|_| {
        ByteFault::new(
            Code::MalformedBitmap,
            3..len,
            format!(
                "Base2 payload is {} bytes; {bits} declared positions need {}",
                len - 3,
                bits.div_ceil(8)
            ),
        )
    })?;
    // Upstream truncates unused bits. This profile deliberately rejects nonzero padding.
    if !bits.is_multiple_of(8) && bitmap.last().is_some_and(|last| last >> (bits % 8) != 0) {
        return Err(ByteFault::new(
            Code::MalformedBitmap,
            len - 1..len,
            "Unused padding bits in the last byte are nonzero; this profile requires canonical zero padding",
        ));
    }
    let Decoded::Base2(bits) = decoded else {
        return Err(ByteFault::new(
            Code::UnsupportedSignerEncoding,
            0..1,
            "Expected Base2",
        ));
    };
    let ranks: Vec<u32> = bits.iter_ones().map(|rank| rank as u32).collect();
    if ranks.is_empty() {
        return Err(ByteFault::new(
            Code::EmptySignerSet,
            3..len,
            "No signer bit is set; an empty signer set cannot certify anything",
        ));
    }
    Ok(ranks)
}

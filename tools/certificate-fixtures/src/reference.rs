//! Reduced reference schemas adapted from Apache-2.0 Agave wire.rs at 98535e50.
//! Used only by fixture generation and tests; the runtime decoder does not use these.
use wincode::{SchemaRead, SchemaWrite};
#[derive(Clone, Debug, PartialEq, Eq, SchemaRead, SchemaWrite)]
pub struct Block {
    pub slot: u64,
    pub block_id: [u8; 32],
}
#[derive(Clone, Debug, PartialEq, Eq, SchemaRead, SchemaWrite)]
pub struct CertSignature {
    pub signature: [u8; 192],
    pub bitmap: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, SchemaRead, SchemaWrite)]
pub struct BlockCert {
    pub block: Block,
    pub signature: CertSignature,
}
#[derive(Clone, Debug, PartialEq, Eq, SchemaRead, SchemaWrite)]
pub struct SlotCert {
    pub slot: u64,
    pub signature: CertSignature,
}
#[derive(Clone, Debug, PartialEq, Eq, SchemaRead, SchemaWrite)]
#[wincode(tag_encoding = "u8")]
pub enum Kind {
    #[wincode(tag = 7)]
    Finalize(SlotCert),
    #[wincode(tag = 8)]
    FastFinalize(BlockCert),
    #[wincode(tag = 9)]
    Notar(BlockCert),
    #[wincode(tag = 12)]
    Genesis(BlockCert),
}
#[derive(Clone, Debug, PartialEq, Eq, SchemaRead, SchemaWrite)]
pub struct V1 {
    pub kind: Kind,
    pub shred_version: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, SchemaRead, SchemaWrite)]
#[wincode(tag_encoding = "u8")]
pub enum Versioned {
    #[wincode(tag = 1)]
    V1(V1),
}
#[derive(Clone, Debug, SchemaRead, SchemaWrite)]
#[wincode(tag_encoding = "u8")]
pub enum Vote {
    #[wincode(tag = 1)]
    Notar { block: Block, shred_version: u16 },
    #[wincode(tag = 2)]
    Finalize { slot: u64, shred_version: u16 },
    #[wincode(tag = 6)]
    Genesis { block: Block, shred_version: u16 },
}
pub fn payload(kind: &str, slot: u64, block_id: [u8; 32], shred: u16) -> Vec<u8> {
    let block = Block { slot, block_id };
    let vote = match kind {
        "finalization" => Vote::Finalize {
            slot,
            shred_version: shred,
        },
        "genesis" => Vote::Genesis {
            block,
            shred_version: shred,
        },
        "notarization" | "fast_finalization" => Vote::Notar {
            block,
            shred_version: shred,
        },
        _ => panic!("fixture kind unsupported"),
    };
    wincode::serialize(&vote).unwrap()
}
pub fn message(
    kind: &str,
    slot: u64,
    block_id: [u8; 32],
    shred: u16,
    signature: [u8; 192],
    bitmap: Vec<u8>,
) -> Vec<u8> {
    let signature = CertSignature { signature, bitmap };
    let block = Block { slot, block_id };
    let kind = match kind {
        "finalization" => Kind::Finalize(SlotCert { slot, signature }),
        "fast_finalization" => Kind::FastFinalize(BlockCert { block, signature }),
        "notarization" => Kind::Notar(BlockCert { block, signature }),
        "genesis" => Kind::Genesis(BlockCert { block, signature }),
        _ => panic!("fixture kind unsupported"),
    };
    wincode::serialize(&Versioned::V1(V1 {
        kind,
        shred_version: shred,
    }))
    .unwrap()
}

/// Test-only on-curve G2 point outside the prime-order subgroup. Never accepted as a signature.
pub fn non_subgroup_signature() -> [u8; 192] {
    for x in 0..=u16::MAX {
        let mut compressed = [0; 96];
        compressed[0] = 0x80;
        compressed[94..].copy_from_slice(&x.to_be_bytes());
        if let Some(point) = Option::<blstrs::G2Affine>::from(
            blstrs::G2Affine::from_compressed_unchecked(&compressed),
        ) && bool::from(point.is_on_curve())
            && !bool::from(point.is_torsion_free())
        {
            return point.to_uncompressed();
        }
    }
    panic!("no deterministic non-subgroup test point found");
}

/// Test-only G1 counterpart; avoids encodings rejected before the subgroup check.
pub fn non_subgroup_key() -> [u8; 96] {
    for x in 1..=u16::MAX {
        let mut compressed = [0; 48];
        compressed[0] = 0x80;
        compressed[46..].copy_from_slice(&x.to_be_bytes());
        if let Some(point) = Option::<blstrs::G1Affine>::from(
            blstrs::G1Affine::from_compressed_unchecked(&compressed),
        ) && bool::from(point.is_on_curve())
            && !bool::from(point.is_torsion_free())
        {
            return point.to_uncompressed();
        }
    }
    panic!("no deterministic non-subgroup test point found");
}

//! C1 compatibility experiment, not a finality verifier or an untrusted bundle API.
//! Real Solana BLS primitives verify a fixed synthetic fixture on both targets.

use serde::{Deserialize, Serialize};
use solana_bls_signatures::{
    ProofOfPossession, Pubkey, PubkeyProjective, Signature, VerifyPop, VerifySignature,
    pubkey::PubkeyAffine,
};

pub const PROFILE_ID: &str = "agave-4.4.0-beta.0-c1-synthetic";
pub const AGAVE_REVISION: &str = "98535e50e9bb38574e6b0c31ff284cd864d3ca88";
pub const CASE_NAMES: [&str; 6] = [
    "valid_aggregate",
    "changed_block_id",
    "changed_shred_version",
    "missing_signer",
    "invalid_pop",
    "invalid_signature",
];
pub const FIXTURE_JSON: &str = include_str!("../fixtures/aggregate.json");

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub profile_id: String,
    pub provenance: String,
    pub payload_hex: String,
    pub public_keys_hex: Vec<String>,
    pub proofs_of_possession_hex: Vec<String>,
    pub signature_hex: String,
}

/// The pinned Notar payload: tag 1, little-endian slot, 32-byte block ID, LE u16.
/// This small adapter is checked against the upstream wincode schema in tests.
pub fn notarization_payload(slot: u64, block_id: [u8; 32], shred_version: u16) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(43);
    bytes.push(1);
    bytes.extend_from_slice(&slot.to_le_bytes());
    bytes.extend_from_slice(&block_id);
    bytes.extend_from_slice(&shred_version.to_le_bytes());
    bytes
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProbeError {
    Fixture,
    Encoding,
    ProofOfPossession,
    Signature,
    UnknownCase,
}

fn decode<const N: usize>(value: &str) -> Result<[u8; N], ProbeError> {
    hex::decode(value)
        .map_err(|_| ProbeError::Encoding)?
        .try_into()
        .map_err(|_| ProbeError::Encoding)
}

/// Only called with this experiment's bounded, embedded, synthetic fixture.
/// C2 will introduce the separate general-purpose evidence/trust boundary.
fn verify_fixture(
    fixture: &Fixture,
    payload: &[u8],
    signer_count: usize,
) -> Result<(), ProbeError> {
    if fixture.profile_id != PROFILE_ID
        || fixture.public_keys_hex.len() != 3
        || fixture.proofs_of_possession_hex.len() != 3
        || !(1..=3).contains(&signer_count)
    {
        return Err(ProbeError::Fixture);
    }
    let mut keys = Vec::with_capacity(signer_count);
    for index in 0..signer_count {
        let key = Pubkey(decode(&fixture.public_keys_hex[index])?);
        let key = PubkeyAffine::try_from(&key).map_err(|_| ProbeError::Encoding)?;
        let proof = ProofOfPossession(decode(&fixture.proofs_of_possession_hex[index])?);
        keys.push(
            key.verify_and_wrap_pop(&proof, None)
                .map_err(|_| ProbeError::ProofOfPossession)?,
        );
    }
    let aggregate = PubkeyProjective::aggregate(keys.iter()).map_err(|_| ProbeError::Encoding)?;
    let signature = Signature(decode(&fixture.signature_hex)?);
    aggregate
        .verify_signature(&signature, payload)
        .map_err(|_| ProbeError::Signature)
}

pub fn run_case(index: u32) -> Result<(), ProbeError> {
    let mut fixture: Fixture =
        serde_json::from_str(FIXTURE_JSON).map_err(|_| ProbeError::Fixture)?;
    let mut payload = hex::decode(&fixture.payload_hex).map_err(|_| ProbeError::Fixture)?;
    if payload.len() != 43 {
        return Err(ProbeError::Fixture);
    }
    let mut signers = 3;
    match index {
        0 => {}
        1 => payload[9] ^= 1,
        2 => payload[41] ^= 1,
        3 => signers = 2,
        4 => fixture.proofs_of_possession_hex[0] = "00".repeat(192),
        5 => fixture.signature_hex = "00".repeat(192),
        _ => return Err(ProbeError::UnknownCase),
    }
    verify_fixture(&fixture, &payload, signers)
}

/// Stable, pointer-free ABI for this fixed-case experiment only.
/// 1 = accepted; 0 = cryptographic rejection; 2 = experiment/input failure.
pub fn case_result(index: u32) -> u32 {
    match run_case(index) {
        Ok(()) => 1,
        Err(ProbeError::ProofOfPossession | ProbeError::Signature) => 0,
        Err(_) => 2,
    }
}

#[cfg(target_arch = "wasm32")]
fn unavailable_entropy(_: &mut [u8]) -> Result<(), getrandom::Error> {
    Err(getrandom::Error::UNSUPPORTED)
}
#[cfg(target_arch = "wasm32")]
getrandom::register_custom_getrandom!(unavailable_entropy);

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn monocle_case_result(index: u32) -> u32 {
    case_result(index)
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn monocle_case_count() -> u32 {
    CASE_NAMES.len() as u32
}

// Serialization reference adapted from Apache-2.0 licensed Agave; see THIRD_PARTY.md.
use monocle_bls_compat::{FIXTURE_JSON, Fixture, ProbeError, notarization_payload, run_case};
use wincode::SchemaWrite;

// Minimal independent serialization reference matching the pinned upstream schema.
// Upstream: Agave 98535e50, votor-messages/src/wire.rs and consensus_message.rs.
#[derive(SchemaWrite)]
struct ReferenceBlock {
    slot: u64,
    block_id: [u8; 32],
}
#[derive(SchemaWrite)]
#[wincode(tag_encoding = "u8")]
enum ReferenceVote {
    #[wincode(tag = 1)]
    Notar {
        block: ReferenceBlock,
        shred_version: u16,
    },
}

#[test]
fn payload_matches_upstream_schema_and_fixed_bytes() {
    let payload = notarization_payload(42, [0x42; 32], 1234);
    let reference = ReferenceVote::Notar {
        block: ReferenceBlock {
            slot: 42,
            block_id: [0x42; 32],
        },
        shred_version: 1234,
    };
    assert_eq!(payload, wincode::serialize(&reference).unwrap());
    assert_eq!(
        hex::encode(&payload),
        concat!(
            "012a00000000000000",
            "4242424242424242424242424242424242424242424242424242424242424242",
            "d204"
        )
    );
    let fixture: Fixture = serde_json::from_str(FIXTURE_JSON).unwrap();
    assert_eq!(fixture.payload_hex, hex::encode(payload));
}

#[test]
fn serialized_aggregate_verifies_after_real_pop_checks() {
    assert_eq!(run_case(0), Ok(()));
}
#[test]
fn changed_block_id_is_rejected() {
    assert_eq!(run_case(1), Err(ProbeError::Signature));
}
#[test]
fn wrong_shred_context_is_rejected() {
    assert_eq!(run_case(2), Err(ProbeError::Signature));
}
#[test]
fn removing_a_signer_breaks_the_old_signature() {
    assert_eq!(run_case(3), Err(ProbeError::Signature));
}
#[test]
fn invalid_pop_is_rejected_before_signature_verification() {
    assert_eq!(run_case(4), Err(ProbeError::ProofOfPossession));
}
#[test]
fn invalid_signature_bytes_are_rejected() {
    assert_eq!(run_case(5), Err(ProbeError::Signature));
}
#[test]
fn unknown_probe_case_is_not_treated_as_a_crypto_rejection() {
    assert_eq!(run_case(6), Err(ProbeError::UnknownCase));
}

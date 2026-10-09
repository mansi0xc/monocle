//! Public deterministic test keys only. Never use these seeds for real validators.
use monocle_bls_compat::{Fixture, PROFILE_ID, notarization_payload};
use solana_bls_signatures::{Keypair, ProofOfPossession, Pubkey, Signature, SignatureProjective};

fn main() {
    let payload = notarization_payload(42, [0x42; 32], 1234);
    let keys: Vec<_> = (1..=3)
        .map(|seed| Keypair::derive(&[seed; 32]).expect("fixed test seed"))
        .collect();
    let signatures: Vec<_> = keys.iter().map(|key| key.sign(&payload)).collect();
    let aggregate = SignatureProjective::aggregate(signatures.iter()).expect("nonempty signatures");
    let signature: Signature = aggregate.into();
    let fixture = Fixture {
        profile_id: PROFILE_ID.into(),
        provenance:
            "SYNTHETIC: public test seeds [1;32], [2;32], [3;32]; no real validator or stake claim"
                .into(),
        payload_hex: hex::encode(payload),
        public_keys_hex: keys
            .iter()
            .map(|key| hex::encode(Pubkey::from(*key.public).0))
            .collect(),
        proofs_of_possession_hex: keys
            .iter()
            .map(|key| hex::encode(ProofOfPossession::from(key.proof_of_possession(None)).0))
            .collect(),
        signature_hex: hex::encode(signature.0),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&fixture).expect("fixture serializes")
    );
}

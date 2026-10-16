//! Public deterministic fixture generator. It never calls the runtime certificate encoder/decoder.
mod reference;
use bitvec::{order::Lsb0, vec::BitVec};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use solana_bls_signatures::{
    Keypair, ProofOfPossession, Pubkey, PubkeyProjective, Signature, SignatureProjective,
};
use solana_signer_store::encode_base2;

const PROFILE: &str = "agave-98535e50-base2-v1";
fn digest(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}
/// Where the expected diagnostic points, from the reduced wire layout rather than
/// the runtime decoder: tag, kind, slot, optional block ID, 192-byte signature,
/// 8-byte bitmap length, bitmap, 2-byte shred version, then any trailing bytes.
fn expected_path(name: &str, kind: &str, bitmap_len: usize, wire_len: usize) -> String {
    let signature = if kind == "finalization" { 10 } else { 42 };
    let bitmap = signature + 192 + 8;
    let shred = bitmap + bitmap_len;
    let range = |start: usize, end: usize| format!("certificates[0].raw_hex[{start}..{end}]");
    match name {
        "wrong_subgroup_signature"
        | "changed_payload"
        | "changed_bitmap"
        | "malformed_signature"
        | "identity_signature"
        | "wrong_signed_context" => range(signature, signature + 192),
        "wrong_key_pop" | "missing_pop" | "invalid_pop" => {
            "validators[0].proof_of_possession_hex".into()
        }
        "invalid_pubkey" | "identity_pubkey" | "wrong_subgroup_key" => {
            "validators[0].bls_pubkey_hex".into()
        }
        "wrong_wire_context" => range(shred, shred + 2),
        "unknown_bitmap_version" | "base3_bitmap" => range(bitmap, bitmap + 1),
        "unknown_rank" => range(bitmap + 1, bitmap + 3),
        "empty_bitmap" => range(bitmap + 3, shred),
        "nonzero_padding" => range(shred - 1, shred),
        "unknown_wire_version" => range(0, 1),
        "unsupported_wire_kind" => range(1, 2),
        "trailing_wire" => range(shred + 2, wire_len),
        "envelope_mismatch" => "certificates[0].kind".into(),
        "claim_mismatch" => range(2, 10),
        "block_claim_deferred" | "genesis_claim_deferred" => "$".into(),
        // Verified and under-quorum cases: the quorum check names the signer set.
        _ => range(bitmap, shred),
    }
}
fn main() {
    let keys: Vec<_> = (11..=15)
        .map(|seed| Keypair::derive(&[seed; 32]).unwrap())
        .collect();
    let stakes = [40, 25, 18, 15, 2];
    let checkpoint = json!({"schema_version":1,"profile_id":PROFILE,"context":{"genesis_hash_hex":"11".repeat(32),"epoch":"42","shred_version":1234},"total_stake":"100","validators":keys.iter().enumerate().map(|(rank,key)| json!({"rank":rank,"identity_hex":format!("{:02x}",rank+1).repeat(32),"bls_pubkey_hex":hex::encode(Pubkey::from(*key.public).0),"stake":stakes[rank].to_string(),"proof_of_possession_hex":hex::encode(ProofOfPossession::from(key.proof_of_possession(None)).0)})).collect::<Vec<_>>(),"provenance":{"kind":"synthetic","description":"Public seeds [11;32] through [15;32]; invented epoch/stake context; no operational keys."}});
    let definitions = [
        (
            "wrong_subgroup_signature",
            "notarization",
            vec![0, 1],
            "REJECTED",
            "signature",
            "SIGNATURE_OUTSIDE_SUBGROUP",
        ),
        (
            "wrong_key_pop",
            "notarization",
            vec![0, 1],
            "INCONCLUSIVE",
            "key_qualification",
            "INVALID_POP",
        ),
        (
            "valid_notarization",
            "notarization",
            vec![0, 1],
            "VERIFIED",
            "quorum",
            "OK",
        ),
        (
            "valid_finalization",
            "finalization",
            vec![0, 1],
            "VERIFIED",
            "quorum",
            "OK",
        ),
        (
            "valid_fast",
            "fast_finalization",
            vec![0, 1, 3],
            "VERIFIED",
            "quorum",
            "OK",
        ),
        (
            "valid_genesis",
            "genesis",
            vec![0, 1, 3, 4],
            "VERIFIED",
            "quorum",
            "OK",
        ),
        (
            "notar_under_quorum",
            "notarization",
            vec![0, 2],
            "REJECTED",
            "quorum",
            "INSUFFICIENT_STAKE",
        ),
        (
            "fast_under_quorum",
            "fast_finalization",
            vec![0, 1],
            "REJECTED",
            "quorum",
            "INSUFFICIENT_STAKE",
        ),
        (
            "genesis_under_quorum",
            "genesis",
            vec![0, 1, 3],
            "REJECTED",
            "quorum",
            "INSUFFICIENT_STAKE",
        ),
        (
            "changed_payload",
            "notarization",
            vec![0, 1],
            "REJECTED",
            "signature",
            "INVALID_SIGNATURE",
        ),
        (
            "changed_bitmap",
            "notarization",
            vec![0, 1],
            "REJECTED",
            "signature",
            "INVALID_SIGNATURE",
        ),
        (
            "malformed_signature",
            "notarization",
            vec![0, 1],
            "REJECTED",
            "signature",
            "MALFORMED_SIGNATURE",
        ),
        (
            "identity_signature",
            "notarization",
            vec![0, 1],
            "REJECTED",
            "signature",
            "IDENTITY_SIGNATURE",
        ),
        (
            "wrong_wire_context",
            "notarization",
            vec![0, 1],
            "INCONCLUSIVE",
            "certificate_decoding",
            "CONTEXT_MISMATCH",
        ),
        (
            "wrong_signed_context",
            "notarization",
            vec![0, 1],
            "REJECTED",
            "signature",
            "INVALID_SIGNATURE",
        ),
        (
            "unknown_bitmap_version",
            "notarization",
            vec![0, 1],
            "INCONCLUSIVE",
            "signers",
            "UNSUPPORTED_SIGNER_ENCODING",
        ),
        (
            "base3_bitmap",
            "notarization",
            vec![0, 1],
            "INCONCLUSIVE",
            "signers",
            "UNSUPPORTED_SIGNER_ENCODING",
        ),
        (
            "empty_bitmap",
            "notarization",
            vec![0, 1],
            "REJECTED",
            "signers",
            "EMPTY_SIGNER_SET",
        ),
        (
            "unknown_rank",
            "notarization",
            vec![0, 1],
            "REJECTED",
            "signers",
            "UNKNOWN_SIGNER_RANK",
        ),
        (
            "nonzero_padding",
            "notarization",
            vec![0, 1],
            "REJECTED",
            "signers",
            "MALFORMED_BITMAP",
        ),
        (
            "missing_pop",
            "notarization",
            vec![0, 1],
            "INCONCLUSIVE",
            "key_qualification",
            "MISSING_POP",
        ),
        (
            "invalid_pop",
            "notarization",
            vec![0, 1],
            "INCONCLUSIVE",
            "key_qualification",
            "INVALID_POP",
        ),
        (
            "invalid_pubkey",
            "notarization",
            vec![0, 1],
            "INCONCLUSIVE",
            "key_qualification",
            "INVALID_PUBLIC_KEY",
        ),
        (
            "identity_pubkey",
            "notarization",
            vec![0, 1],
            "INCONCLUSIVE",
            "key_qualification",
            "INVALID_PUBLIC_KEY",
        ),
        (
            "wrong_subgroup_key",
            "notarization",
            vec![0, 1],
            "INCONCLUSIVE",
            "key_qualification",
            "INVALID_PUBLIC_KEY",
        ),
        (
            "unknown_wire_version",
            "notarization",
            vec![0, 1],
            "INCONCLUSIVE",
            "certificate_decoding",
            "UNSUPPORTED_WIRE_VERSION",
        ),
        (
            "unsupported_wire_kind",
            "notarization",
            vec![0, 1],
            "INCONCLUSIVE",
            "certificate_decoding",
            "UNSUPPORTED_CERTIFICATE",
        ),
        (
            "trailing_wire",
            "notarization",
            vec![0, 1],
            "REJECTED",
            "certificate_decoding",
            "MALFORMED_CERTIFICATE",
        ),
        (
            "envelope_mismatch",
            "notarization",
            vec![0, 1],
            "REJECTED",
            "certificate_decoding",
            "ENVELOPE_MISMATCH",
        ),
        (
            "claim_mismatch",
            "notarization",
            vec![0, 1],
            "REJECTED",
            "certificate_decoding",
            "CLAIM_MISMATCH",
        ),
        (
            "block_claim_deferred",
            "fast_finalization",
            vec![0, 1, 3],
            "INCONCLUSIVE",
            "finality",
            "FINALITY_NOT_EVALUATED",
        ),
        (
            "genesis_claim_deferred",
            "genesis",
            vec![0, 1, 3, 4],
            "INCONCLUSIVE",
            "finality",
            "FINALITY_NOT_EVALUATED",
        ),
    ];
    let mut cases = Vec::new();
    for (name, kind, ranks, conclusion, stage, code) in definitions {
        let payload = reference::payload(kind, 42, [0x42; 32], 1234);
        let signatures: Vec<_> = ranks
            .iter()
            .map(|rank| keys[*rank].sign(&payload))
            .collect();
        let mut signature =
            Signature::from(SignatureProjective::aggregate(signatures.iter()).unwrap()).0;
        let mut bits: BitVec<u8, Lsb0> = BitVec::repeat(false, 5);
        for rank in &ranks {
            bits.set(*rank, true);
        }
        let mut bitmap = encode_base2(&bits).unwrap();
        let mut cp = checkpoint.clone();
        let mut block = [0x42; 32];
        let mut shred = 1234;
        match name {
            "changed_payload" => block[0] ^= 1,
            "wrong_subgroup_signature" => signature = reference::non_subgroup_signature(),
            "wrong_key_pop" => {
                cp["validators"][0]["proof_of_possession_hex"] =
                    cp["validators"][1]["proof_of_possession_hex"].clone()
            }
            "changed_bitmap" => bitmap[3] = 5,
            "malformed_signature" => signature = [0; 192],
            "identity_signature" => signature = Signature::from(SignatureProjective::identity()).0,
            "wrong_wire_context" => shred = 1235,
            "wrong_signed_context" => {
                shred = 1235;
                cp["context"]["shred_version"] = json!(1235);
            }
            "unknown_bitmap_version" => bitmap[0] = 9,
            "base3_bitmap" => bitmap[0] = 1,
            "empty_bitmap" => bitmap[3] = 0,
            "unknown_rank" => {
                bitmap[1] = 6;
                bitmap[3] = 32;
            }
            "nonzero_padding" => bitmap[3] |= 128,
            "missing_pop" => cp["validators"][0]["proof_of_possession_hex"] = Value::Null,
            "invalid_pop" => {
                cp["validators"][0]["proof_of_possession_hex"] = json!("00".repeat(192))
            }
            "invalid_pubkey" => cp["validators"][0]["bls_pubkey_hex"] = json!("00".repeat(96)),
            "identity_pubkey" => {
                cp["validators"][0]["bls_pubkey_hex"] =
                    json!(hex::encode(Pubkey::from(PubkeyProjective::identity()).0))
            }
            "wrong_subgroup_key" => {
                cp["validators"][0]["bls_pubkey_hex"] =
                    json!(hex::encode(reference::non_subgroup_key()));
            }
            _ => {}
        }
        let mut wire = reference::message(kind, 42, block, shred, signature, bitmap.clone());
        match name {
            "unknown_wire_version" => wire[0] = 2,
            "unsupported_wire_kind" => wire[1] = 10,
            "trailing_wire" => wire.push(0),
            _ => {}
        }
        let path = expected_path(name, kind, bitmap.len(), wire.len());
        let checkpoint_json = serde_json::to_string(&cp).unwrap();
        let claim_kind = match name {
            "block_claim_deferred" => "block_finality",
            "genesis_claim_deferred" => "genesis_migration",
            _ => "certificate_inspection",
        };
        let bundle = json!({"schema_version":1,"profile_id":PROFILE,"context":cp["context"],"claim":{"kind":claim_kind,"slot":if name=="claim_mismatch" {"43"} else {"42"},"block_id_hex":if kind=="finalization" {Value::Null} else {json!(hex::encode(block))},"certificate_ids":["cert-1"]},"certificates":[{"schema_version":1,"id":"cert-1","kind":if name=="envelope_mismatch" {"genesis"} else {kind},"format":"agave-versioned-consensus-v1","raw_hex":hex::encode(&wire)}],"checkpoint_reference":digest(checkpoint_json.as_bytes()),"provenance":{"kind":"synthetic","description":format!("C3 signed public test fixture: {name}. No live network claim.")},"parent_digest":null,"mutations":[]});
        cases.push(json!({"name":name,"bundle_json":serde_json::to_string(&bundle).unwrap(),"checkpoint_json":checkpoint_json,"expected":{"conclusion":conclusion,"stage":stage,"code":code,"path":path},"signed_payload_hex":hex::encode(payload),"original_signer_ranks":ranks}));
    }
    println!("{}",serde_json::to_string_pretty(&json!({"schema_version":1,"profile_id":PROFILE,"provenance":"Generated independently of the runtime decoder using reduced pinned wincode schemas and upstream signer-store encoder. Public test seeds only.","cases":cases})).unwrap());
}

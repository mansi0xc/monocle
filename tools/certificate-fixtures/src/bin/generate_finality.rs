//! Compose independently encoded C3 fixtures into C4 claim scenarios.
//! The only newly signed certificate uses the same reduced reference schema.
#[allow(dead_code)]
#[path = "../reference.rs"]
mod reference;
use bitvec::{order::Lsb0, vec::BitVec};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use solana_bls_signatures::{Keypair, Signature, SignatureProjective};
use solana_signer_store::encode_base2;

fn seed(name: &str) -> (Value, Value) {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../../fixtures/certificates/corpus.json"
    ))
    .unwrap();
    let c = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap();
    (
        serde_json::from_str(c["bundle_json"].as_str().unwrap()).unwrap(),
        serde_json::from_str(c["checkpoint_json"].as_str().unwrap()).unwrap(),
    )
}
fn cert(name: &str, id: &str) -> Value {
    let (bundle, _) = seed(name);
    let mut cert = bundle["certificates"][0].clone();
    cert["id"] = json!(id);
    cert
}
fn finalizer(ranks: &[usize]) -> Value {
    let payload = reference::payload("finalization", 42, [0x42; 32], 1234);
    let keys: Vec<_> = ranks
        .iter()
        .map(|rank| Keypair::derive(&[11 + *rank as u8; 32]).unwrap())
        .collect();
    let signatures: Vec<_> = keys.iter().map(|key| key.sign(&payload)).collect();
    let signature = Signature::from(SignatureProjective::aggregate(signatures.iter()).unwrap()).0;
    let mut bits: BitVec<u8, Lsb0> = BitVec::repeat(false, 5);
    for rank in ranks {
        bits.set(*rank, true);
    }
    let wire = reference::message(
        "finalization",
        42,
        [0x42; 32],
        1234,
        signature,
        encode_base2(&bits).unwrap(),
    );
    let mut cert = cert("valid_finalization", "finalize");
    cert["raw_hex"] = json!(hex::encode(wire));
    cert
}
fn main() {
    let cases = [
        (
            "inspection_notar",
            "VERIFIED",
            "individual_certificates",
            None,
            "FINALITY_NOT_REQUESTED",
        ),
        (
            "fast_finality",
            "VERIFIED",
            "block_finality",
            Some("fast"),
            "FAST_FINALITY_ESTABLISHED",
        ),
        (
            "slow_finality",
            "VERIFIED",
            "block_finality",
            Some("slow"),
            "SLOW_FINALITY_ESTABLISHED",
        ),
        (
            "slow_reversed",
            "VERIFIED",
            "block_finality",
            Some("slow"),
            "SLOW_FINALITY_ESTABLISHED",
        ),
        (
            "slow_different_signers",
            "VERIFIED",
            "block_finality",
            Some("slow"),
            "SLOW_FINALITY_ESTABLISHED",
        ),
        (
            "genesis_anchor",
            "VERIFIED",
            "genesis_migration",
            None,
            "GENESIS_ANCHOR_ESTABLISHED",
        ),
        (
            "notar_only",
            "INCONCLUSIVE",
            "unestablished",
            None,
            "MISSING_FINALIZATION",
        ),
        (
            "finalize_only",
            "INCONCLUSIVE",
            "unestablished",
            None,
            "MISSING_NOTARIZATION",
        ),
        (
            "genesis_as_block",
            "REJECTED",
            "unestablished",
            None,
            "INCOMPATIBLE_EVIDENCE",
        ),
        (
            "fast_as_genesis",
            "REJECTED",
            "unestablished",
            None,
            "INCOMPATIBLE_EVIDENCE",
        ),
        (
            "mixed_genesis_block",
            "REJECTED",
            "unestablished",
            None,
            "INCOMPATIBLE_EVIDENCE",
        ),
        (
            "mixed_genesis_migration",
            "REJECTED",
            "unestablished",
            None,
            "INCOMPATIBLE_EVIDENCE",
        ),
        (
            "missing_block_id",
            "INCONCLUSIVE",
            "unestablished",
            None,
            "MISSING_BLOCK_ID",
        ),
        (
            "missing_genesis_block_id",
            "INCONCLUSIVE",
            "unestablished",
            None,
            "MISSING_BLOCK_ID",
        ),
        (
            "slow_wrong_slot",
            "REJECTED",
            "unestablished",
            None,
            "CLAIM_MISMATCH",
        ),
        (
            "slow_wrong_block",
            "REJECTED",
            "unestablished",
            None,
            "CLAIM_MISMATCH",
        ),
        (
            "wrong_context",
            "INCONCLUSIVE",
            "unestablished",
            None,
            "CONTEXT_MISMATCH",
        ),
        (
            "notar_under_quorum",
            "REJECTED",
            "unestablished",
            None,
            "INSUFFICIENT_STAKE",
        ),
        (
            "finalize_under_quorum",
            "REJECTED",
            "unestablished",
            None,
            "INSUFFICIENT_STAKE",
        ),
        (
            "fast_under_quorum",
            "REJECTED",
            "unestablished",
            None,
            "INSUFFICIENT_STAKE",
        ),
        (
            "genesis_under_quorum",
            "REJECTED",
            "unestablished",
            None,
            "INSUFFICIENT_STAKE",
        ),
        (
            "changed_signature",
            "REJECTED",
            "unestablished",
            None,
            "INVALID_SIGNATURE",
        ),
        (
            "missing_pop",
            "INCONCLUSIVE",
            "unestablished",
            None,
            "MISSING_POP",
        ),
        (
            "unsupported_bitmap",
            "INCONCLUSIVE",
            "unestablished",
            None,
            "UNSUPPORTED_SIGNER_ENCODING",
        ),
        (
            "unsupported_kind",
            "INCONCLUSIVE",
            "unestablished",
            None,
            "UNSUPPORTED_CERTIFICATE",
        ),
        (
            "saved_report_injection",
            "REJECTED",
            "unestablished",
            None,
            "UNKNOWN_FIELD",
        ),
        (
            "no_checkpoint",
            "INCONCLUSIVE",
            "unestablished",
            None,
            "MISSING_CHECKPOINT",
        ),
        (
            "unknown_profile",
            "INCONCLUSIVE",
            "unestablished",
            None,
            "UNSUPPORTED_PROFILE",
        ),
        (
            "fast_with_bad_extra",
            "REJECTED",
            "unestablished",
            None,
            "INVALID_SIGNATURE",
        ),
        (
            "fast_with_unverified_extra",
            "INCONCLUSIVE",
            "unestablished",
            None,
            "UNSUPPORTED_SIGNER_ENCODING",
        ),
        (
            "slow_duplicate_notar",
            "INCONCLUSIVE",
            "unestablished",
            None,
            "MISSING_FINALIZATION",
        ),
        (
            "fast_and_slow",
            "VERIFIED",
            "block_finality",
            Some("fast"),
            "FAST_FINALITY_ESTABLISHED",
        ),
        (
            "genesis_wrong_target",
            "REJECTED",
            "unestablished",
            None,
            "CLAIM_MISMATCH",
        ),
        (
            "unknown_format",
            "INCONCLUSIVE",
            "unestablished",
            None,
            "UNSUPPORTED_CERTIFICATE",
        ),
        (
            "unknown_claim",
            "INCONCLUSIVE",
            "unestablished",
            None,
            "UNSUPPORTED_CERTIFICATE",
        ),
    ];
    let mut output = Vec::new();
    for (name, conclusion, scope, path, diagnostic) in cases {
        let (mut bundle, mut checkpoint) = seed("valid_notarization");
        let n = cert("valid_notarization", "notar");
        let f = cert("valid_finalization", "finalize");
        let fast = cert("valid_fast", "fast");
        let g = cert("valid_genesis", "genesis");
        let mut certificates = match name {
            "inspection_notar" | "notar_only" => vec![n.clone()],
            "finalize_only" => vec![f.clone()],
            "genesis_anchor"
            | "genesis_as_block"
            | "missing_genesis_block_id"
            | "genesis_wrong_target" => vec![g.clone()],
            "mixed_genesis_block" | "mixed_genesis_migration" => vec![g.clone(), fast.clone()],
            "slow_finality" | "slow_wrong_slot" | "slow_wrong_block" => vec![n.clone(), f.clone()],
            "slow_reversed" => vec![f.clone(), n.clone()],
            "slow_different_signers" => vec![n.clone(), finalizer(&[0, 2, 3, 4])],
            "notar_under_quorum" => vec![cert("notar_under_quorum", "notar"), f.clone()],
            "finalize_under_quorum" => vec![n.clone(), finalizer(&[0, 2])],
            "fast_under_quorum" => vec![cert("fast_under_quorum", "fast")],
            "genesis_under_quorum" => vec![cert("genesis_under_quorum", "genesis")],
            "changed_signature" => vec![cert("changed_bitmap", "notar"), f.clone()],
            "missing_pop" => {
                checkpoint = seed("missing_pop").1;
                vec![fast.clone()]
            }
            "unsupported_bitmap" => vec![cert("base3_bitmap", "notar"), f.clone()],
            "unsupported_kind" => vec![cert("unsupported_wire_kind", "notar")],
            "fast_with_bad_extra" => vec![fast.clone(), cert("changed_bitmap", "bad")],
            "fast_with_unverified_extra" => vec![fast.clone(), cert("base3_bitmap", "unsupported")],
            "slow_duplicate_notar" => vec![n.clone(), cert("valid_notarization", "notar-copy")],
            "fast_and_slow" => vec![n.clone(), f.clone(), fast.clone()],
            _ => vec![fast.clone()],
        };
        match name {
            "slow_wrong_slot" => {
                let mut bytes = hex::decode(certificates[1]["raw_hex"].as_str().unwrap()).unwrap();
                bytes[2] ^= 1;
                certificates[1]["raw_hex"] = json!(hex::encode(bytes));
            }
            "slow_wrong_block" => {
                let mut bytes = hex::decode(certificates[0]["raw_hex"].as_str().unwrap()).unwrap();
                bytes[10] ^= 1;
                certificates[0]["raw_hex"] = json!(hex::encode(bytes));
            }
            "unknown_format" => certificates[0]["format"] = json!("unknown"),
            _ => {}
        }
        bundle["claim"]["kind"] = json!(match name {
            "inspection_notar" => "certificate_inspection",
            "genesis_anchor"
            | "fast_as_genesis"
            | "mixed_genesis_migration"
            | "missing_genesis_block_id"
            | "genesis_under_quorum"
            | "genesis_wrong_target" => "genesis_migration",
            "unknown_claim" => "ancestry",
            _ => "block_finality",
        });
        bundle["claim"]["certificate_ids"] = json!(
            certificates
                .iter()
                .map(|c| c["id"].clone())
                .collect::<Vec<_>>()
        );
        bundle["certificates"] = json!(certificates);
        bundle["provenance"]["description"] = json!(format!(
            "C4 synthetic composition case: {name}; C3 public keys/context."
        ));
        match name {
            "missing_block_id" | "missing_genesis_block_id" => {
                bundle["claim"]["block_id_hex"] = Value::Null
            }
            "wrong_context" => bundle["context"]["epoch"] = json!("43"),
            "genesis_wrong_target" => bundle["claim"]["block_id_hex"] = json!("43".repeat(32)),
            "saved_report_injection" => {
                bundle["report"] =
                    json!({"conclusion":"VERIFIED","evidence_scope":"block_finality"})
            }
            "unknown_profile" => bundle["profile_id"] = json!("unknown"),
            _ => {}
        }
        let checkpoint_json = serde_json::to_string(&checkpoint).unwrap();
        bundle["checkpoint_reference"] = json!(format!(
            "sha256:{}",
            hex::encode(Sha256::digest(checkpoint_json.as_bytes()))
        ));
        let supporting = match name {
            "inspection_notar" => vec!["notar"],
            "fast_finality" | "fast_and_slow" => vec!["fast"],
            "slow_finality" | "slow_reversed" | "slow_different_signers" => {
                vec!["notar", "finalize"]
            }
            "genesis_anchor" => vec!["genesis"],
            _ => vec![],
        };
        output.push(json!({"name":name,"bundle_json":serde_json::to_string(&bundle).unwrap(),"checkpoint_json":checkpoint_json,"select_checkpoint":name!="no_checkpoint","expected":{"conclusion":conclusion,"evidence_scope":scope,"finality_path":path,"diagnostic":diagnostic,"supporting_certificate_ids":supporting}}));
    }
    println!("{}",serde_json::to_string_pretty(&json!({"schema_version":1,"policy_id":"agave-98535e50-direct-finality-v1","provenance":"Synthetic C3 public fixtures composed into C4 claims; no runtime verifier dependency. Additional finalize votes signed with the reduced reference schema.","cases":output})).unwrap());
}

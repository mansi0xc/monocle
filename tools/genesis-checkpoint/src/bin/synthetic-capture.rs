//! synthetic-capture <out-dir>
//!
//! Writes deterministic `validators` and `genesis-cert` captures in exactly the
//! format `scripts/capture-rpc.mjs` records, for a synthetic cluster with public
//! test keys. It exercises every admission and rank-map exclusion and a stake tie,
//! and its genesis certificate is genuinely signed. No live network claim.
use bitvec::{order::Lsb0, vec::BitVec};
use monocle_genesis_checkpoint::VOTE_PROGRAM;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use solana_bls_signatures::{Keypair, PubkeyCompressed, Signature, SignatureProjective};
use solana_signer_store::encode_base2;
use std::{fs, path::Path};

const SLOTS_PER_EPOCH: u64 = 432_000;
const CAPTURE_EPOCH: u64 = 99;
const SHRED_VERSION: u16 = 1234;
const RENT_MINIMUM: u64 = 19_761_200;
const GENESIS_SLOT: u64 = (CAPTURE_EPOCH + 1) * SLOTS_PER_EPOCH + 5_000;

enum Key {
    None,
    Seed(u8),
    Invalid,
}
struct Validator {
    stake: u64,
    lamports: u64,
    v3: bool,
    key: Key,
    node: u8,
    delinquent: bool,
}

fn validators() -> Vec<Validator> {
    let v = |stake, key, node| Validator {
        stake,
        lamports: 50_000_000,
        v3: false,
        key,
        node,
        delinquent: false,
    };
    vec![
        v(400, Key::Seed(21), 1),
        v(250, Key::Seed(22), 2),
        // Equal stake: rank order falls back to compressed key bytes.
        v(150, Key::Seed(23), 3),
        v(150, Key::Seed(24), 4),
        Validator {
            v3: true,
            ..v(90, Key::None, 5)
        },
        v(0, Key::Seed(26), 6),
        Validator {
            lamports: RENT_MINIMUM - 1,
            ..v(70, Key::Seed(27), 7)
        },
        // One BLS key registered by two vote accounts: both are dropped.
        v(60, Key::Seed(28), 8),
        v(55, Key::Seed(28), 9),
        // One node identity behind two vote accounts: both are dropped.
        v(40, Key::Seed(30), 10),
        v(35, Key::Seed(31), 10),
        v(20, Key::Invalid, 12),
        Validator {
            delinquent: true,
            ..v(10, Key::Seed(33), 13)
        },
    ]
}

fn keypair(seed: u8) -> Keypair {
    Keypair::derive(&[seed; 32]).expect("public test seed")
}
fn compressed(seed: u8) -> [u8; 48] {
    let affine = *keypair(seed).public;
    PubkeyCompressed::from(&affine).0
}
fn b58(bytes: &[u8]) -> String {
    bs58::encode(bytes).into_string()
}
fn vote_pubkey(index: usize) -> [u8; 32] {
    [index as u8 + 1; 32]
}

/// A 3762-byte vote account: version tag, node, three zeroed pubkeys, commissions,
/// pending rewards, then (V4 only) the BLS key option.
fn vote_state(validator: &Validator) -> Vec<u8> {
    let mut data = vec![0u8; 3762];
    data[0..4].copy_from_slice(&(if validator.v3 { 2u32 } else { 3 }).to_le_bytes());
    data[4..36].copy_from_slice(&[validator.node; 32]);
    if !validator.v3 {
        match validator.key {
            Key::None => {}
            Key::Seed(seed) => {
                data[144] = 1;
                data[145..193].copy_from_slice(&compressed(seed));
            }
            Key::Invalid => data[144] = 1, // 48 zero bytes: not a valid compressed point
        }
    }
    data
}

struct Writer {
    dir: std::path::PathBuf,
    files: Vec<Value>,
}
impl Writer {
    fn response(&mut self, name: &str, method: &str, params: Value, result: Value) {
        let id = self.files.len() + 1;
        let request = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        let body = serde_json::to_vec(&json!({"jsonrpc": "2.0", "result": result, "id": id}))
            .expect("serialize");
        let file = format!("{:03}-{name}.json", self.files.len());
        fs::write(self.dir.join(&file), &body).expect("write capture file");
        self.files.push(json!({
            "file": file,
            "method": method,
            "request": request.to_string(),
            "http_status": 200,
            "sha256": hex::encode(Sha256::digest(&body)),
            "bytes": body.len(),
            "received_at": "2026-10-16T00:00:00.000Z",
        }));
    }
    fn finish(self, mode: &str, first_slot: u64, last_slot: u64) {
        let manifest = json!({
            "schema_version": 1,
            "format": "monocle-rpc-capture",
            "mode": mode,
            "endpoint": "synthetic:monocle-test-cluster",
            "epoch": first_slot / SLOTS_PER_EPOCH,
            "first_slot": first_slot,
            "last_slot": last_slot,
            "files": self.files,
        });
        let text = serde_json::to_string_pretty(&manifest).expect("serialize") + "\n";
        fs::write(self.dir.join("manifest.json"), text).expect("write manifest");
    }
}
fn writer(dir: &Path) -> Writer {
    fs::create_dir_all(dir).expect("create capture directory");
    Writer {
        dir: dir.to_path_buf(),
        files: Vec::new(),
    }
}
fn epoch_info(slot: u64) -> Value {
    json!({"absoluteSlot": slot, "blockHeight": slot - 1000, "epoch": slot / SLOTS_PER_EPOCH,
        "slotIndex": slot % SLOTS_PER_EPOCH, "slotsInEpoch": SLOTS_PER_EPOCH, "transactionCount": 1})
}

fn main() {
    let out = std::env::args_os()
        .nth(1)
        .expect("usage: synthetic-capture <out-dir>");
    let out = Path::new(&out);
    let validators = validators();
    let finalized = json!({"commitment": "finalized"});
    let start = CAPTURE_EPOCH * SLOTS_PER_EPOCH + 1_200;

    let mut w = writer(&out.join("validators"));
    w.response(
        "epoch-info",
        "getEpochInfo",
        json!([finalized]),
        epoch_info(start),
    );
    w.response(
        "epoch-schedule",
        "getEpochSchedule",
        json!([]),
        json!({
        "firstNormalEpoch": 0, "firstNormalSlot": 0, "leaderScheduleSlotOffset": SLOTS_PER_EPOCH,
        "slotsPerEpoch": SLOTS_PER_EPOCH, "warmup": false}),
    );
    w.response(
        "genesis-hash",
        "getGenesisHash",
        json!([]),
        json!(b58(&[0x11; 32])),
    );
    let nodes: Vec<Value> = [SHRED_VERSION, SHRED_VERSION, SHRED_VERSION, 999]
        .iter()
        .enumerate()
        .map(|(i, v)| json!({"pubkey": b58(&[0x60 + i as u8; 32]), "shredVersion": v}))
        .collect();
    w.response("cluster-nodes", "getClusterNodes", json!([]), json!(nodes));
    w.response(
        "rent-vote-v4",
        "getMinimumBalanceForRentExemption",
        json!([3762, finalized]),
        json!(RENT_MINIMUM),
    );
    w.response(
        "alpenglow-feature",
        "getAccountInfo",
        json!(["A1pengvuM6JEcyNuTnMqepBKhwHE3N6PmUrdATGawhJS", {"commitment": "finalized", "encoding": "base64"}]),
        json!({"context": {"apiVersion": "4.3.0", "slot": start + 1}, "value": null}),
    );
    let listing = |delinquent: bool| -> Vec<Value> {
        validators
            .iter()
            .enumerate()
            .filter(|(_, v)| v.delinquent == delinquent)
            .map(|(i, v)| {
                json!({"votePubkey": b58(&vote_pubkey(i)), "nodePubkey": b58(&[v.node; 32]),
                "activatedStake": v.stake, "commission": 0, "epochCredits": [],
                "epochVoteAccount": v.stake > 0, "lastVote": start, "rootSlot": start - 32})
            })
            .collect()
    };
    w.response(
        "vote-accounts",
        "getVoteAccounts",
        json!([finalized]),
        json!({"current": listing(false), "delinquent": listing(true)}),
    );
    // Request order matches the listing: current, then delinquent.
    let order: Vec<usize> = (0..validators.len())
        .filter(|&i| !validators[i].delinquent)
        .chain((0..validators.len()).filter(|&i| validators[i].delinquent))
        .collect();
    for (batch, chunk) in order.chunks(5).enumerate() {
        let keys: Vec<String> = chunk.iter().map(|&i| b58(&vote_pubkey(i))).collect();
        let values: Vec<Value> = chunk
            .iter()
            .map(|&i| {
                let v = &validators[i];
                json!({"data": [base64::Engine::encode(&base64::engine::general_purpose::STANDARD, vote_state(v)), "base64"],
                    "executable": false, "lamports": v.lamports, "owner": VOTE_PROGRAM,
                    "rentEpoch": u64::MAX, "space": 3762})
            })
            .collect();
        w.response(
            &format!("vote-state-{batch:03}"),
            "getMultipleAccounts",
            json!([keys, {"commitment": "finalized", "encoding": "base64"}]),
            json!({"context": {"apiVersion": "4.3.0", "slot": start + 2}, "value": values}),
        );
    }
    w.response(
        "epoch-info-end",
        "getEpochInfo",
        json!([finalized]),
        epoch_info(start + 3),
    );
    w.finish("validators", start, start + 3);

    // Rank the five qualifying validators as Agave would, then sign with ranks
    // covering 800 of 960 stake (83.3%), including the later of the tied pair.
    let mut ranked: Vec<(u64, [u8; 48], u8)> =
        [(400, 21), (250, 22), (150, 23), (150, 24), (10, 33)]
            .iter()
            .map(|&(stake, seed)| (stake, compressed(seed), seed))
            .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let block = [0x47u8; 32];
    let mut payload = vec![6u8];
    payload.extend_from_slice(&GENESIS_SLOT.to_le_bytes());
    payload.extend_from_slice(&block);
    payload.extend_from_slice(&SHRED_VERSION.to_le_bytes());
    let signers = [0usize, 1, 3];
    let signatures: Vec<_> = signers
        .iter()
        .map(|&rank| keypair(ranked[rank].2).sign(&payload))
        .collect();
    let signature =
        Signature::from(SignatureProjective::aggregate(signatures.iter()).expect("aggregate")).0;
    let mut bits: BitVec<u8, Lsb0> = BitVec::repeat(false, ranked.len());
    for rank in signers {
        bits.set(rank, true);
    }
    let bitmap = encode_base2(&bits).expect("bitmap");

    let cert_start = GENESIS_SLOT + 64;
    let mut w = writer(&out.join("genesis-cert"));
    w.response(
        "epoch-info",
        "getEpochInfo",
        json!([finalized]),
        epoch_info(cert_start),
    );
    w.response(
        "genesis-cert",
        "getAgGenesisCert",
        json!([]),
        json!({
        "block": {"slot": GENESIS_SLOT, "blockId": block.to_vec()},
        "signature": {"signature": signature.to_vec(), "bitmap": bitmap}}),
    );
    w.response(
        "epoch-info-end",
        "getEpochInfo",
        json!([finalized]),
        epoch_info(cert_start + 1),
    );
    w.finish("genesis-cert", cert_start, cert_start + 1);
}

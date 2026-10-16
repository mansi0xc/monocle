use base64::Engine;
use monocle_core::{TrustBasis, TrustStore, finality, parse_checkpoint};
use monocle_genesis_checkpoint::*;
use serde_json::{Value, json};
use std::{fs, path::Path};

const SYNTHETIC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/genesis-capture/synthetic"
);

fn built() -> (BuiltCheckpoint, String) {
    let dir = Path::new(SYNTHETIC);
    let checkpoint =
        build_checkpoint(&read_capture(&dir.join("validators"), "validators").unwrap()).unwrap();
    let bundle = build_bundle(
        &read_capture(&dir.join("genesis-cert"), "genesis-cert").unwrap(),
        &checkpoint,
    )
    .unwrap();
    (checkpoint, bundle)
}

fn verify(checkpoint: &str, bundle: &str, basis: TrustBasis) -> Value {
    let mut trust = TrustStore::new();
    let digest = trust
        .pin(parse_checkpoint(checkpoint.as_bytes()).unwrap(), basis)
        .unwrap();
    let request = json!({"schema_version": 1, "run_id": "genesis-test",
        "profile_id": monocle_core::certificate::PROFILE_ID, "bundle_json": bundle, "selected_checkpoint": digest});
    serde_json::to_value(finality::verify_evidence(
        &serde_json::to_vec(&request).unwrap(),
        &trust,
    ))
    .unwrap()
}
fn codes(report: &Value) -> String {
    report.to_string()
}

#[test]
fn committed_outputs_are_exactly_what_the_builder_produces() {
    let (checkpoint, bundle) = built();
    let dir = Path::new(SYNTHETIC).join("built");
    assert_eq!(
        checkpoint.json,
        fs::read_to_string(dir.join("checkpoint.json")).unwrap()
    );
    assert_eq!(bundle, fs::read_to_string(dir.join("bundle.json")).unwrap());
    let summary = &checkpoint.summary;
    assert_eq!(summary["ranked"], 5);
    assert_eq!(summary["total_stake"], "960");
    assert_eq!(summary["checkpoint_epoch"], 100);
    assert_eq!(summary["shred_version"], 1234);
    assert_eq!(
        summary["excluded"],
        json!({"no_bls_key": 1, "zero_stake": 1, "below_minimum_balance": 1, "below_admission_cutoff": 0,
            "invalid_bls_key": 1, "duplicate_bls_key": 2, "duplicate_node": 2})
    );
}

#[test]
fn the_rebuilt_genesis_certificate_verifies_only_under_registered_key_trust() {
    let (checkpoint, bundle) = built();
    let registered = verify(
        &checkpoint.json,
        &bundle,
        TrustBasis::RegisteredKeys {
            reason: "Rebuilt from the synthetic capture".into(),
        },
    );
    assert_eq!(registered["conclusion"], "VERIFIED");
    assert_eq!(registered["evidence_scope"], "genesis_migration");
    assert!(codes(&registered).contains("GENESIS_ANCHOR_ESTABLISHED"));
    assert!(codes(&registered).contains("REGISTERED_KEYS_ACCEPTED"));
    // The rebuilt checkpoint has no proofs of possession, so ordinary trust cannot qualify keys.
    let operator = verify(
        &checkpoint.json,
        &bundle,
        TrustBasis::OperatorPinned {
            reason: "Reviewed".into(),
        },
    );
    assert_eq!(operator["conclusion"], "INCONCLUSIVE");
    assert!(codes(&operator).contains("MISSING_POP"));
    // A captured checkpoint can never be selected as a synthetic fixture.
    let parsed = parse_checkpoint(checkpoint.json.as_bytes()).unwrap();
    assert!(
        TrustStore::new()
            .pin(parsed, TrustBasis::SyntheticFixture)
            .is_err()
    );
}

#[test]
fn a_wrong_rank_order_or_signer_set_cannot_verify() {
    let (checkpoint, bundle) = built();
    // Swap the two equal-stake validators: the bitmap then selects the wrong key.
    let mut cp: Value = serde_json::from_str(&checkpoint.json).unwrap();
    let validators = cp["validators"].as_array_mut().unwrap();
    let (two, three) = (
        validators[2]["bls_pubkey_hex"].clone(),
        validators[3]["bls_pubkey_hex"].clone(),
    );
    validators[2]["bls_pubkey_hex"] = three;
    validators[3]["bls_pubkey_hex"] = two;
    let swapped = serde_json::to_string(&cp).unwrap();
    let mut b: Value = serde_json::from_str(&bundle).unwrap();
    b["checkpoint_reference"] =
        serde_json::to_value(parse_checkpoint(swapped.as_bytes()).unwrap().digest()).unwrap();
    let report = verify(
        &swapped,
        &b.to_string(),
        TrustBasis::RegisteredKeys {
            reason: "test".into(),
        },
    );
    assert_eq!(report["conclusion"], "REJECTED");
    assert!(codes(&report).contains("INVALID_SIGNATURE"));
}

#[test]
fn a_modified_capture_file_is_refused() {
    let tmp = std::env::temp_dir().join(format!("monocle-capture-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let source = Path::new(SYNTHETIC).join("genesis-cert");
    for entry in fs::read_dir(&source).unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), tmp.join(entry.file_name())).unwrap();
    }
    let file = tmp.join("001-genesis-cert.json");
    let text = fs::read_to_string(&file)
        .unwrap()
        .replacen("\"slot\":", "\"slot\": ", 1);
    fs::write(&file, text).unwrap();
    let error = read_capture(&tmp, "genesis-cert").err().unwrap();
    assert!(error.contains("SHA-256 differs"), "{error}");
    assert!(
        read_capture(&source, "validators").is_err(),
        "mode must match"
    );
    fs::remove_dir_all(&tmp).unwrap();
}

fn account(stake: u64, seed: u8, node: u8) -> VoteAccount {
    let key = solana_bls_signatures::Keypair::derive(&[seed; 32]).unwrap();
    let affine = *key.public;
    VoteAccount {
        vote: [seed; 32],
        node: [node; 32],
        stake,
        lamports: u64::MAX,
        bls: Some(solana_bls_signatures::PubkeyCompressed::from(&affine).0),
    }
}

#[test]
fn admission_cutoff_drops_every_account_tied_with_the_first_truncated_stake() {
    // Four accounts, room for two: the third-largest stake (20) is the floor, so
    // both accounts holding 20 go, even though one of them would fit by count.
    let accounts = [
        account(30, 1, 1),
        account(20, 2, 2),
        account(20, 3, 3),
        account(10, 4, 4),
    ];
    let map = rank_map(&accounts, 0, 2).unwrap();
    assert_eq!(map.cutoff_stake, Some(20));
    assert_eq!(map.ranked.len(), 1);
    assert_eq!(map.excluded.below_admission_cutoff, 3);
    assert_eq!(map.total_stake, 30);
    // Without the cap all four rank, by stake then compressed key.
    let map = rank_map(&accounts, 0, MAX_ALPENGLOW_VOTE_ACCOUNTS).unwrap();
    let stakes: Vec<u64> = map.ranked.iter().map(|r| r.stake).collect();
    assert_eq!(stakes, [30, 20, 20, 10]);
    assert!(map.ranked[1].bls_compressed < map.ranked[2].bls_compressed);
}

#[test]
fn vote_state_reader_handles_versions_and_rejects_malformed_layouts() {
    let mut v4 = vec![0u8; 3762];
    v4[0] = 3;
    v4[4..36].copy_from_slice(&[9; 32]);
    assert_eq!(parse_vote_state(&v4).unwrap(), ([9; 32], None));
    v4[144] = 2;
    assert!(parse_vote_state(&v4).is_err(), "invalid option tag");
    let mut v3 = v4.clone();
    v3[0] = 2;
    assert_eq!(parse_vote_state(&v3).unwrap().1, None, "V3 has no BLS key");
    let mut uninitialized = v4.clone();
    uninitialized[0] = 0;
    assert!(parse_vote_state(&uninitialized).is_err());
    assert!(parse_vote_state(&v4[..100]).is_err());
}

#[test]
fn real_mainnet_vote_account_matches_its_listing_and_holds_a_valid_key() {
    let vector: Value = serde_json::from_str(include_str!(
        "../../../fixtures/genesis-capture/mainnet-vote-account.json"
    ))
    .unwrap();
    let data = base64::engine::general_purpose::STANDARD
        .decode(vector["data_base64"].as_str().unwrap())
        .unwrap();
    let (node, bls) = parse_vote_state(&data).unwrap();
    // Independent check of the offsets: getVoteAccounts reported this node identity.
    assert_eq!(bs58::encode(node).into_string(), vector["node_pubkey"]);
    let key = bls.expect("V4 account with a registered BLS key");
    assert_eq!(hex::encode(key), vector["bls_pubkey_compressed_hex"]);
    // Bytes read from a wrong offset would essentially never decompress to a subgroup point.
    assert!(solana_bls_signatures::pubkey::PubkeyAffine::try_from(&key).is_ok());
}

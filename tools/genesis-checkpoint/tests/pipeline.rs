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

const MAINNET: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/genesis-capture/mainnet"
);

#[test]
fn the_recorded_mainnet_genesis_certificate_verifies_only_under_registered_key_trust() {
    let dir = Path::new(MAINNET);
    let checkpoint =
        build_checkpoint(&read_capture(&dir.join("validators"), "validators").unwrap()).unwrap();
    let bundle = build_bundle(
        &read_capture(&dir.join("genesis-cert"), "genesis-cert").unwrap(),
        &checkpoint,
    )
    .unwrap();
    // check-genesis.sh compares the committed outputs byte for byte; these pin the values.
    let summary = &checkpoint.summary;
    assert_eq!(
        summary["checkpoint_digest"],
        "sha256:a83a1b6f8e64efc0ffbb41c8b5404c9bcf06cf4d1d1b3900e468b4ecf8520194"
    );
    assert_eq!(summary["checkpoint_epoch"], 1064);
    assert_eq!(summary["ranked"], 673);
    assert_eq!(summary["total_stake"], "438565601666629812");
    assert_eq!(summary["shred_version"], 50093);
    assert_eq!(summary["shred_version_agreement"], "3912/3912");
    assert_eq!(summary["slots_after_boundary"], 3797);
    assert_eq!(
        summary["excluded"],
        json!({"no_bls_key": 4, "zero_stake": 2, "below_minimum_balance": 0, "below_admission_cutoff": 0,
            "invalid_bls_key": 0, "duplicate_bls_key": 0, "duplicate_node": 0})
    );

    let registered = TrustBasis::RegisteredKeys {
        reason: "Rebuilt from the recorded epoch-1063 mainnet capture".into(),
    };
    let report = verify(&checkpoint.json, &bundle, registered.clone());
    assert_eq!(report["conclusion"], "VERIFIED");
    assert_eq!(report["evidence_scope"], "genesis_migration");
    assert!(codes(&report).contains("GENESIS_ANCHOR_ESTABLISHED"));
    assert!(codes(&report).contains("REGISTERED_KEYS_ACCEPTED"));
    // 446 of 673 ranks signed; the encoder trims the bitmap after the last signer.
    assert!(
        codes(&report).contains(
            "Signers hold 360285044710320374 of 438565601666629812 stake, meeting the exact 82% threshold of 359623793366636446"
        ),
        "{report}"
    );

    let operator = verify(
        &checkpoint.json,
        &bundle,
        TrustBasis::OperatorPinned {
            reason: "Reviewed".into(),
        },
    );
    assert_eq!(operator["conclusion"], "INCONCLUSIVE");
    assert!(codes(&operator).contains("MISSING_POP"));

    // Dropping the highest-staked signer leaves a well-formed signature over the wrong set.
    let mut tampered: Value = serde_json::from_str(&bundle).unwrap();
    let raw = tampered["certificates"][0]["raw_hex"].as_str().unwrap();
    let first_ranks = 2 * 245;
    assert_eq!(
        &raw[first_ranks..first_ranks + 2],
        "ff",
        "ranks 0-7 all signed"
    );
    let raw = format!("{}fe{}", &raw[..first_ranks], &raw[first_ranks + 2..]);
    tampered["certificates"][0]["raw_hex"] = json!(raw);
    let rejected = verify(&checkpoint.json, &tampered.to_string(), registered);
    assert_eq!(rejected["conclusion"], "REJECTED");
    assert!(codes(&rejected).contains("INVALID_SIGNATURE"), "{rejected}");
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

#[test]
fn only_a_feature_program_account_with_an_activation_slot_is_active() {
    let account = |owner: &str, data: &[u8]| {
        let data = base64::engine::general_purpose::STANDARD.encode(data);
        json!({"context": {"apiVersion": "4.3.2", "slot": 1}, "value": {"data": [data, "base64"],
            "executable": false, "lamports": 1_000_000, "owner": owner,
            "rentEpoch": u64::MAX, "space": 9}})
    };
    let activated = [1, 0xc0, 0x5a, 0x65, 0x1b, 0, 0, 0, 0];
    assert!(!feature_active(&json!({"context": {"slot": 1}, "value": null})).unwrap());
    // Mainnet, 2026-10-20: the feature address was sent lamports before activation.
    let funded = account("11111111111111111111111111111111", &[]);
    assert!(!feature_active(&funded).unwrap());
    assert!(!feature_active(&account("11111111111111111111111111111111", &activated)).unwrap());
    assert!(
        !feature_active(&account(FEATURE_PROGRAM, &[0])).unwrap(),
        "pending"
    );
    assert!(feature_active(&account(FEATURE_PROGRAM, &activated)).unwrap());
    for malformed in [&[][..], &[1], &activated[..8], &[2]] {
        assert!(feature_active(&account(FEATURE_PROGRAM, malformed)).is_err());
    }
}

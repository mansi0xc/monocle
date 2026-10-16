//! Rebuild, offline, the validator rank map Agave uses to verify Alpenglow
//! certificates for one epoch, from recorded RPC responses; and turn a recorded
//! `getAgGenesisCert` response into a Monocle proof bundle.
//!
//! Rules follow Agave v4.3.0, which mainnet runs; they are identical at Monocle's
//! pinned revision 98535e50:
//! - `runtime/src/bank.rs` `update_epoch_stakes`: at the first bank of epoch N the
//!   VAT-filtered vote accounts become the stakes for epoch N + 1.
//! - `vote/src/vote_account.rs` `clone_and_filter_for_vat` (SIMD-0357 admission).
//! - `runtime/src/epoch_stakes.rs` `BLSPubkeyToRankMap::new` (ranks and denominator).
//!
//! A capture only approximates the snapshot: it is recorded during epoch N, after
//! the boundary. Nothing here authenticates the endpoint; the caller still decides
//! whether to trust the result.
use base64::Engine;
use monocle_core::{capture::inspect_genesis_rpc, certificate, parse_bundle, parse_checkpoint};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use solana_bls_signatures::{Pubkey as BlsPubkey, pubkey::PubkeyAffine};
use std::{collections::HashMap, fs, path::Path};

pub type Error = String;

/// `runtime/src/bank.rs` `MAX_ALPENGLOW_VOTE_ACCOUNTS`.
pub const MAX_ALPENGLOW_VOTE_ACCOUNTS: usize = 2000;
pub const VOTE_PROGRAM: &str = "Vote111111111111111111111111111111111111111";
/// VoteStateV4: u32 tag, four pubkeys, two u16 commissions, u64 pending rewards.
const V4_BLS_OPTION: usize = 4 + 4 * 32 + 2 + 2 + 8;

/// A recorded capture whose files all match the digests in its manifest.
pub struct Capture {
    pub manifest: Value,
    pub manifest_digest: String,
    files: Vec<(Value, Vec<u8>)>,
}

pub fn read_capture(dir: &Path, mode: &str) -> Result<Capture, Error> {
    let manifest_bytes =
        fs::read(dir.join("manifest.json")).map_err(|e| format!("{}: {e}", dir.display()))?;
    let manifest: Value =
        serde_json::from_slice(&manifest_bytes).map_err(|e| format!("manifest.json: {e}"))?;
    if manifest["format"] != "monocle-rpc-capture"
        || manifest["schema_version"] != 1
        || manifest["mode"] != mode
    {
        return Err(format!(
            "{}: expected a monocle-rpc-capture {mode} manifest",
            dir.display()
        ));
    }
    let mut files = Vec::new();
    for entry in manifest["files"]
        .as_array()
        .ok_or("manifest.json lists no files")?
    {
        let name = entry["file"].as_str().ok_or("manifest file name missing")?;
        if name.is_empty() || name.contains(['/', '\\']) || name.starts_with('.') {
            return Err(format!("manifest names an unsafe file: {name}"));
        }
        let bytes = fs::read(dir.join(name)).map_err(|e| format!("{name}: {e}"))?;
        if Some(hex::encode(Sha256::digest(&bytes)).as_str()) != entry["sha256"].as_str() {
            return Err(format!(
                "{name}: SHA-256 differs from the manifest; the capture was modified"
            ));
        }
        files.push((entry.clone(), bytes));
    }
    Ok(Capture {
        manifest_digest: hex::encode(Sha256::digest(&manifest_bytes)),
        manifest,
        files,
    })
}

impl Capture {
    fn responses(&self, method: &str) -> Vec<&(Value, Vec<u8>)> {
        self.files
            .iter()
            .filter(|(entry, _)| entry["method"] == method)
            .collect()
    }
    /// The first recorded response for `method`.
    pub fn raw(&self, method: &str) -> Option<&[u8]> {
        self.responses(method)
            .into_iter()
            .next()
            .map(|(_, b)| b.as_slice())
    }
    fn result(&self, method: &str) -> Result<Value, Error> {
        result(
            self.raw(method)
                .ok_or(format!("capture has no {method} response"))?,
            method,
        )
    }
    fn endpoint(&self) -> String {
        let endpoint = self.manifest["endpoint"].as_str().unwrap_or("unknown");
        endpoint.chars().take(160).collect()
    }
}

fn result(bytes: &[u8], method: &str) -> Result<Value, Error> {
    let value: Value = serde_json::from_slice(bytes).map_err(|e| format!("{method}: {e}"))?;
    if value.get("error").is_some() {
        return Err(format!("{method}: RPC error {}", value["error"]));
    }
    value
        .get("result")
        .cloned()
        .ok_or(format!("{method}: response has no result"))
}
fn u64_at(value: &Value, what: &str) -> Result<u64, Error> {
    value
        .as_u64()
        .ok_or(format!("{what}: expected an unsigned 64-bit integer"))
}
fn base58_32(value: &Value, what: &str) -> Result<[u8; 32], Error> {
    let text = value.as_str().ok_or(format!("{what}: expected base58"))?;
    let bytes = bs58::decode(text)
        .into_vec()
        .map_err(|e| format!("{what}: {e}"))?;
    bytes
        .try_into()
        .map_err(|_| format!("{what}: expected 32 bytes"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoteAccount {
    pub vote: [u8; 32],
    pub node: [u8; 32],
    pub stake: u64,
    pub lamports: u64,
    pub bls: Option<[u8; 48]>,
}

/// Read the node pubkey and optional compressed BLS key at their fixed offsets.
/// Both precede every variable-length field (solana-vote-interface 6.x layout).
pub fn parse_vote_state(data: &[u8]) -> Result<([u8; 32], Option<[u8; 48]>), Error> {
    let tag = data
        .get(0..4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or("vote state is truncated")?;
    let node: [u8; 32] = data
        .get(4..36)
        .and_then(|b| b.try_into().ok())
        .ok_or("vote state is truncated")?;
    match tag {
        // V1_14_11 and V3 predate BLS keys.
        1 | 2 => Ok((node, None)),
        3 => match data.get(V4_BLS_OPTION) {
            Some(0) => Ok((node, None)),
            Some(1) => data
                .get(V4_BLS_OPTION + 1..V4_BLS_OPTION + 49)
                .and_then(|b| b.try_into().ok())
                .map(|key| (node, Some(key)))
                .ok_or("vote state V4 BLS key is truncated".into()),
            _ => Err("vote state V4 has an invalid BLS option tag".into()),
        },
        _ => Err(format!("unsupported vote state version {tag}")),
    }
}

/// Pair `getVoteAccounts` stakes with the account data fetched for each listed key.
pub fn vote_accounts(capture: &Capture) -> Result<Vec<VoteAccount>, Error> {
    let listed = capture.result("getVoteAccounts")?;
    let mut stakes = HashMap::new();
    for group in ["current", "delinquent"] {
        for v in listed[group]
            .as_array()
            .ok_or(format!("getVoteAccounts.{group} missing"))?
        {
            let vote = base58_32(&v["votePubkey"], "votePubkey")?;
            let node = base58_32(&v["nodePubkey"], "nodePubkey")?;
            let stake = u64_at(&v["activatedStake"], "activatedStake")?;
            if stakes.insert(vote, (stake, node)).is_some() {
                return Err("getVoteAccounts lists a vote account twice".into());
            }
        }
    }
    let mut accounts = Vec::new();
    for (entry, bytes) in capture.responses("getMultipleAccounts") {
        let request: Value = serde_json::from_str(entry["request"].as_str().unwrap_or(""))
            .map_err(|e| format!("manifest request: {e}"))?;
        let keys = request["params"][0]
            .as_array()
            .ok_or("getMultipleAccounts request lists no keys")?;
        let values = result(bytes, "getMultipleAccounts")?["value"]
            .as_array()
            .cloned()
            .ok_or("getMultipleAccounts returned no values")?;
        if keys.len() != values.len() {
            return Err("getMultipleAccounts returned a different number of accounts".into());
        }
        for (key, account) in keys.iter().zip(&values) {
            let vote = base58_32(key, "requested vote account")?;
            let Some(&(stake, listed_node)) = stakes.get(&vote) else {
                return Err("fetched an account getVoteAccounts did not list".into());
            };
            if account["owner"] != VOTE_PROGRAM {
                return Err(format!("{key}: not owned by the vote program"));
            }
            if account["data"][1] != "base64" {
                return Err(format!("{key}: expected base64 account data"));
            }
            let data = base64::engine::general_purpose::STANDARD
                .decode(account["data"][0].as_str().unwrap_or(""))
                .map_err(|e| format!("{key}: {e}"))?;
            let (node, bls) = parse_vote_state(&data).map_err(|e| format!("{key}: {e}"))?;
            if node != listed_node {
                return Err(format!(
                    "{key}: node pubkey in account data differs from getVoteAccounts"
                ));
            }
            let lamports = u64_at(&account["lamports"], "lamports")?;
            accounts.push(VoteAccount {
                vote,
                node,
                stake,
                lamports,
                bls,
            });
        }
    }
    if accounts.len() != stakes.len() {
        return Err(format!(
            "fetched {} of {} listed vote accounts",
            accounts.len(),
            stakes.len()
        ));
    }
    Ok(accounts)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Exclusions {
    pub no_bls_key: usize,
    pub zero_stake: usize,
    pub below_minimum_balance: usize,
    pub below_admission_cutoff: usize,
    pub invalid_bls_key: usize,
    pub duplicate_bls_key: usize,
    pub duplicate_node: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ranked {
    pub vote: [u8; 32],
    pub node: [u8; 32],
    pub stake: u64,
    pub bls_compressed: [u8; 48],
    pub bls_uncompressed: [u8; 96],
}

#[derive(Clone, Debug)]
pub struct RankMap {
    pub ranked: Vec<Ranked>,
    pub total_stake: u64,
    pub admitted: usize,
    pub cutoff_stake: Option<u64>,
    pub excluded: Exclusions,
}

/// SIMD-0357 admission, then Agave's rank map: drop undecodable keys and every
/// account sharing a BLS key or node identity, sort by stake (descending) then
/// compressed key bytes, and sum the remaining stake as the denominator.
pub fn rank_map(
    accounts: &[VoteAccount],
    minimum_balance: u64,
    max_accounts: usize,
) -> Result<RankMap, Error> {
    let mut excluded = Exclusions::default();
    let mut admitted = Vec::new();
    for account in accounts {
        match account.bls {
            None => excluded.no_bls_key += 1,
            Some(_) if account.stake == 0 => excluded.zero_stake += 1,
            Some(_) if account.lamports < minimum_balance => excluded.below_minimum_balance += 1,
            Some(key) => admitted.push((account, key)),
        }
    }
    let mut cutoff_stake = None;
    if admitted.len() > max_accounts {
        // Remove every account at or below the first truncated stake, ties included.
        let mut stakes: Vec<u64> = admitted.iter().map(|(a, _)| a.stake).collect();
        stakes.sort_unstable_by(|a, b| b.cmp(a));
        let floor = stakes[max_accounts];
        let before = admitted.len();
        admitted.retain(|(a, _)| a.stake > floor);
        excluded.below_admission_cutoff = before - admitted.len();
        cutoff_stake = Some(floor);
    }
    let admitted_count = admitted.len();
    let mut bls_counts: HashMap<[u8; 48], usize> = HashMap::new();
    let mut node_counts: HashMap<[u8; 32], usize> = HashMap::new();
    let mut candidates = Vec::new();
    for (account, key) in admitted {
        // Checked decompression: on curve, prime-order subgroup, not identity.
        let Ok(affine) = PubkeyAffine::try_from(&key) else {
            excluded.invalid_bls_key += 1;
            continue;
        };
        *bls_counts.entry(key).or_default() += 1;
        *node_counts.entry(account.node).or_default() += 1;
        candidates.push((account, key, affine));
    }
    let mut ranked = Vec::new();
    for (account, key, affine) in candidates {
        if bls_counts[&key] != 1 {
            excluded.duplicate_bls_key += 1;
        } else if node_counts[&account.node] != 1 {
            excluded.duplicate_node += 1;
        } else {
            ranked.push(Ranked {
                vote: account.vote,
                node: account.node,
                stake: account.stake,
                bls_compressed: key,
                bls_uncompressed: BlsPubkey::from(&affine).0,
            });
        }
    }
    ranked.sort_by(|a, b| {
        b.stake
            .cmp(&a.stake)
            .then(a.bls_compressed.cmp(&b.bls_compressed))
    });
    let total_stake = ranked
        .iter()
        .try_fold(0u64, |sum, r| sum.checked_add(r.stake))
        .ok_or("ranked stake overflows u64")?;
    if total_stake == 0 {
        return Err("no validator qualifies for the rank map".into());
    }
    Ok(RankMap {
        ranked,
        total_stake,
        admitted: admitted_count,
        cutoff_stake,
        excluded,
    })
}

/// Slot-to-epoch arithmetic for a schedule without warmup.
#[derive(Clone, Copy, Debug)]
pub struct Schedule {
    pub slots_per_epoch: u64,
    pub first_normal_epoch: u64,
    pub first_normal_slot: u64,
}
impl Schedule {
    pub fn epoch_of(&self, slot: u64) -> Result<u64, Error> {
        if slot < self.first_normal_slot {
            return Err("slots during epoch warmup are unsupported".into());
        }
        Ok(self.first_normal_epoch + (slot - self.first_normal_slot) / self.slots_per_epoch)
    }
    pub fn first_slot(&self, epoch: u64) -> u64 {
        self.first_normal_slot + (epoch - self.first_normal_epoch) * self.slots_per_epoch
    }
}

/// The signing context and schedule a checkpoint was built for.
#[derive(Clone, Debug)]
pub struct Context {
    pub genesis_hash: [u8; 32],
    /// The epoch whose certificates this rank map verifies: capture epoch + 1.
    pub epoch: u64,
    pub shred_version: u16,
    pub schedule: Schedule,
}

pub struct BuiltCheckpoint {
    pub json: String,
    pub digest: String,
    pub context: Context,
    pub summary: Value,
}

pub fn build_checkpoint(capture: &Capture) -> Result<BuiltCheckpoint, Error> {
    let schedule_value = capture.result("getEpochSchedule")?;
    if schedule_value["warmup"] != false {
        return Err("epoch schedules with warmup are unsupported".into());
    }
    let schedule = Schedule {
        slots_per_epoch: u64_at(&schedule_value["slotsPerEpoch"], "slotsPerEpoch")?,
        first_normal_epoch: u64_at(&schedule_value["firstNormalEpoch"], "firstNormalEpoch")?,
        first_normal_slot: u64_at(&schedule_value["firstNormalSlot"], "firstNormalSlot")?,
    };
    if schedule.slots_per_epoch == 0
        || schedule_value["leaderScheduleSlotOffset"] != schedule_value["slotsPerEpoch"]
    {
        return Err("expected epoch stakes to be snapshotted exactly one epoch ahead".into());
    }
    let info = capture.result("getEpochInfo")?;
    let capture_epoch = u64_at(&info["epoch"], "epoch")?;
    let first_slot = u64_at(&info["absoluteSlot"], "absoluteSlot")?;
    if schedule.epoch_of(first_slot)? != capture_epoch {
        return Err("getEpochInfo disagrees with getEpochSchedule".into());
    }
    let ends: Vec<_> = capture
        .responses("getEpochInfo")
        .into_iter()
        .map(|(_, b)| result(b, "getEpochInfo"))
        .collect::<Result<_, _>>()?;
    if ends.iter().any(|end| end["epoch"] != info["epoch"]) {
        return Err("the epoch changed during the capture; record it again".into());
    }
    let feature = capture.result("getAccountInfo")?;
    if !feature["value"].is_null() {
        let data = base64::engine::general_purpose::STANDARD
            .decode(feature["value"]["data"][0].as_str().unwrap_or(""))
            .map_err(|e| format!("alpenglow feature account: {e}"))?;
        // bincode `Feature { activated_at: Option<u64> }`: tag 1 means activated.
        if data.first() != Some(&0) {
            return Err("the alpenglow feature is already active; its VAT burn changes the admission minimum, which this builder does not reproduce".into());
        }
    }
    let minimum_balance = u64_at(
        &capture.result("getMinimumBalanceForRentExemption")?,
        "rent minimum",
    )?;
    let genesis_hash = base58_32(&capture.result("getGenesisHash")?, "genesis hash")?;
    let nodes = capture.result("getClusterNodes")?;
    let nodes = nodes.as_array().ok_or("getClusterNodes returned no list")?;
    let mut versions: HashMap<u64, usize> = HashMap::new();
    for node in nodes {
        if let Some(version) = node["shredVersion"].as_u64() {
            *versions.entry(version).or_default() += 1;
        }
    }
    let (&shred_version, &agreeing) = versions
        .iter()
        .max_by_key(|(version, count)| (**count, std::cmp::Reverse(**version)))
        .ok_or("no cluster node reports a shred version")?;
    if agreeing * 2 <= nodes.len() {
        return Err("no shred version is reported by a majority of cluster nodes".into());
    }
    let shred_version =
        u16::try_from(shred_version).map_err(|_| "shred version exceeds u16".to_string())?;

    let accounts = vote_accounts(capture)?;
    let map = rank_map(&accounts, minimum_balance, MAX_ALPENGLOW_VOTE_ACCOUNTS)?;
    let context = Context {
        genesis_hash,
        epoch: capture_epoch + 1,
        shred_version,
        schedule,
    };
    let boundary = schedule.first_slot(capture_epoch);
    let description = format!(
        "Validator rank map for epoch {} rebuilt offline from RPC responses recorded at {} during epoch {} (slots {}-{}, {} slots after the snapshot boundary; manifest sha256:{}). Applies Agave v4.3.0 VAT admission and rank-map rules. Not authenticated: the endpoint and capture timing are assumptions. Keys carry no proofs of possession; selecting registered-key trust accepts them as registered on chain.",
        context.epoch,
        capture.endpoint(),
        capture_epoch,
        first_slot,
        capture.manifest["last_slot"],
        first_slot.saturating_sub(boundary),
        capture.manifest_digest,
    );
    let checkpoint = json!({
        "schema_version": 1,
        "profile_id": certificate::PROFILE_ID,
        "context": {
            "genesis_hash_hex": hex::encode(genesis_hash),
            "epoch": context.epoch.to_string(),
            "shred_version": shred_version,
        },
        "total_stake": map.total_stake.to_string(),
        "validators": map.ranked.iter().enumerate().map(|(rank, r)| json!({
            "rank": rank,
            "identity_hex": hex::encode(r.node),
            "bls_pubkey_hex": hex::encode(r.bls_uncompressed),
            "stake": r.stake.to_string(),
            "proof_of_possession_hex": null,
        })).collect::<Vec<_>>(),
        "provenance": { "kind": "capture", "description": description },
    });
    let json = serde_json::to_string_pretty(&checkpoint).map_err(|e| e.to_string())? + "\n";
    let parsed = parse_checkpoint(json.as_bytes())
        .map_err(|d| format!("built checkpoint is invalid: {} {}", d.path, d.message))?;
    let digest = serde_json::to_value(parsed.digest()).map_err(|e| e.to_string())?;
    let e = &map.excluded;
    let summary = json!({
        "capture_epoch": capture_epoch,
        "checkpoint_epoch": context.epoch,
        "first_slot": first_slot,
        "slots_after_boundary": first_slot.saturating_sub(boundary),
        "endpoint": capture.endpoint(),
        "manifest_sha256": capture.manifest_digest,
        "genesis_hash": capture.result("getGenesisHash")?,
        "shred_version": shred_version,
        "shred_version_agreement": format!("{agreeing}/{}", nodes.len()),
        "minimum_balance": minimum_balance,
        "vote_accounts": accounts.len(),
        "admitted": map.admitted,
        "admission_cutoff_stake": map.cutoff_stake,
        "ranked": map.ranked.len(),
        "total_stake": map.total_stake.to_string(),
        "excluded": {
            "no_bls_key": e.no_bls_key,
            "zero_stake": e.zero_stake,
            "below_minimum_balance": e.below_minimum_balance,
            "below_admission_cutoff": e.below_admission_cutoff,
            "invalid_bls_key": e.invalid_bls_key,
            "duplicate_bls_key": e.duplicate_bls_key,
            "duplicate_node": e.duplicate_node,
        },
        "checkpoint_digest": digest,
    });
    Ok(BuiltCheckpoint {
        digest: digest.as_str().unwrap_or_default().to_owned(),
        json,
        context,
        summary,
    })
}

/// Re-encode the response fields as the pinned complete wire message. The
/// response has no framing or shred version; the checkpoint context supplies it,
/// and a wrong value can only make the signature fail to verify.
pub fn genesis_wire(
    slot: u64,
    block_id: &[u8],
    signature: &[u8],
    bitmap: &[u8],
    shred_version: u16,
) -> Vec<u8> {
    let mut wire = vec![1, 12];
    wire.extend_from_slice(&slot.to_le_bytes());
    wire.extend_from_slice(block_id);
    wire.extend_from_slice(signature);
    wire.extend_from_slice(&(bitmap.len() as u64).to_le_bytes());
    wire.extend_from_slice(bitmap);
    wire.extend_from_slice(&shred_version.to_le_bytes());
    wire
}

pub fn build_bundle(capture: &Capture, checkpoint: &BuiltCheckpoint) -> Result<String, Error> {
    let bytes = capture
        .raw("getAgGenesisCert")
        .ok_or("capture has no getAgGenesisCert response")?;
    let report = inspect_genesis_rpc(bytes)
        .map_err(|d| format!("getAgGenesisCert: {} {}", d.path, d.message))?;
    let cert = report
        .certificate
        .ok_or("the recorded response holds no genesis certificate")?;
    let slot = cert.slot.get();
    let context = &checkpoint.context;
    let epoch = context.schedule.epoch_of(slot)?;
    if epoch != context.epoch {
        return Err(format!(
            "certificate slot {slot} is in epoch {epoch}, but the checkpoint covers epoch {}",
            context.epoch
        ));
    }
    let decode = |text: &str| hex::decode(text).map_err(|e| e.to_string());
    let wire = genesis_wire(
        slot,
        &decode(&cert.block_id_hex)?,
        &decode(&cert.signature_hex)?,
        &decode(&cert.bitmap_hex)?,
        context.shred_version,
    );
    let description = format!(
        "Alpenglow genesis certificate from getAgGenesisCert recorded at {} (manifest sha256:{}). The wire message is re-encoded from the response fields with the checkpoint's shred version {}; the response itself carries neither.",
        capture.endpoint(),
        capture.manifest_digest,
        context.shred_version,
    );
    let bundle = json!({
        "schema_version": 1,
        "profile_id": certificate::PROFILE_ID,
        "context": {
            "genesis_hash_hex": hex::encode(context.genesis_hash),
            "epoch": context.epoch.to_string(),
            "shred_version": context.shred_version,
        },
        "claim": {
            "kind": "genesis_migration",
            "slot": slot.to_string(),
            "block_id_hex": cert.block_id_hex,
            "certificate_ids": ["genesis"],
        },
        "certificates": [{
            "schema_version": 1,
            "id": "genesis",
            "kind": "genesis",
            "format": certificate::WIRE_FORMAT,
            "raw_hex": hex::encode(&wire),
        }],
        "checkpoint_reference": checkpoint.digest,
        "provenance": { "kind": "capture", "description": description },
        "parent_digest": null,
        "mutations": [],
    });
    let json = serde_json::to_string_pretty(&bundle).map_err(|e| e.to_string())? + "\n";
    parse_bundle(json.as_bytes())
        .map_err(|d| format!("built bundle is invalid: {} {}", d.path, d.message))?;
    Ok(json)
}

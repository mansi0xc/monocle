# Verifying the mainnet Alpenglow genesis certificate

C19 adds what Monocle needs to verify a real cluster's genesis certificate:

- a capture tool that records raw RPC responses;
- an offline builder that rebuilds the validator rank map Agave uses;
- a converter from a recorded `getAgGenesisCert` response to a proof bundle;
- an explicit **registered-key trust** basis.

The pipeline is tested end to end on a synthetic cluster in exactly the RPC format. C21 then ran it on mainnet: the Alpenglow genesis certificate for slot 459,652,999 **verifies under registered-key trust**, natively and in the browser. The recorded capture is committed, and CI rebuilds the checkpoint and bundle from its bytes. See [Result, 2026-10-21](#result-2026-10-21) for the figures and what they do and do not establish.

## Timing: capture during epoch 1063

Agave verifies a certificate in epoch N against `epoch_stakes(N)`. That snapshot is taken at the **first bank of epoch N − 1**, from VAT-filtered vote accounts. No RPC returns it afterwards. So the mainnet genesis certificate (epoch 1064) needs the validator set recorded **during epoch 1063**, as early as possible after its boundary.

| Event | Slot | Estimate (217 ms/slot, measured 2026-10-16) |
| --- | --- | --- |
| Epoch 1063 starts; the snapshot for 1064 is taken | 459,216,000 | ≈ 2026-10-20 11:42 UTC |
| Epoch 1064 starts | 459,648,000 | ≈ 2026-10-21 13:46 UTC |
| Alpenglow takes over (5,000 slots in) | 459,653,000 | ≈ 2026-10-21 14:04 UTC |

Check the epoch first. The capture tool records it, and the builder refuses a capture whose epoch changed mid-run.

```sh
curl -s https://api.mainnet-beta.solana.com -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getEpochInfo"}'
```

## Steps

1. **During epoch 1063**, record the validator set. Use a fresh directory; the tool never overwrites one.

   ```sh
   npm run capture:rpc -- validators https://api.mainnet-beta.solana.com captures/mainnet-1063-validators
   ```

2. **Once migrated** (`getAgGenesisCert` stops returning `null`), record the certificate:

   ```sh
   npm run capture:rpc -- genesis-cert https://api.mainnet-beta.solana.com captures/mainnet-genesis-cert
   ```

3. **Offline**, build the checkpoint and bundle:

   ```sh
   npm run build:genesis -- captures/mainnet-1063-validators out/mainnet-genesis captures/mainnet-genesis-cert
   ```

   The builder writes `checkpoint.json`, `bundle.json` and `summary.json`. The summary lists the vote accounts read, every exclusion and its reason, the ranked count, the denominator, the shred version and how many nodes agreed on it, and how many slots after the boundary the capture began.
4. In the inspector, import `bundle.json` and then `checkpoint.json`. Choose **Operator-pinned, keys registered on chain**, record why you trust the capture, and verify. Choosing **Operator-pinned checkpoint** instead gives `MISSING_POP`, because vote accounts store keys but not proofs.

Keep the capture directories. Their manifests and digests are what make a result reproducible, and every description in the checkpoint and bundle cites the manifest digest.

## What the builder reproduces

These rules come from Agave v4.3.0, the release mainnet ran when C19 was written. They are identical at Monocle's pinned `master` revision `98535e50`, which diverged from v4.3.0. Mainnet migrated on v4.3.2; its 23 commits since v4.3.0 add thread pools and performance work but change none of these rules, the signed payload, or certificate verification:

1. **Admission (SIMD-0357, `clone_and_filter_for_vat`).**
   - Keep vote accounts that have a BLS key, nonzero stake, and lamports at least equal to the minimum.
   - The minimum is the rent-exempt balance for a 3,762-byte V4 vote account. Once the `alpenglow` feature is active, the VAT burn is added; that depends on slot parameters, so the builder refuses post-activation captures. As in Agave, "active" means the feature program owns the account and it records an activation slot; on 2026-10-20 the mainnet feature address held lamports but was still a System-owned account, which is not a feature.
   - If more than 2,000 accounts remain, drop every account whose stake is at or below the 2,001st-largest stake, ties included.
2. **Rank map (`BLSPubkeyToRankMap::new`).**
   - Drop keys that fail checked decompression.
   - Drop **every** account that shares a BLS key or node identity with another.
   - Sort by stake descending, then compressed key bytes ascending.
   - The denominator is the sum of the ranked stakes, not the epoch's total stake.
3. **Context.**
   - The genesis hash comes from `getGenesisHash`.
   - The shred version is the value reported by a strict majority of `getClusterNodes`.
   - The checkpoint epoch is the capture epoch plus one, and the certificate's slot must fall in it.
4. **Wire message.** The certificate is re-encoded as the complete version 1, tag 12 message from the response fields, with the checkpoint's shred version. The response carries neither framing nor shred version. A wrong value can only make the signature fail.

Vote accounts are read at fixed offsets from their raw bytes:

| Field | Location |
| --- | --- |
| Version tag | `u32` at byte 0 (V4 is 3) |
| Node identity | bytes 4..36 |
| BLS key option | tag byte at 144 |
| Compressed BLS key | bytes 145..193 |

Both fields precede all variable-length data. A mainnet test vector pins the offsets: its node identity must equal `getVoteAccounts`' `nodePubkey`, and its key must decompress.

## What stays assumed

- **The endpoint is not authenticated.** A dishonest RPC could serve a consistent fake validator set and certificate. The report says so, and trust remains your explicit choice.
- **Capture timing is an approximation.** Activated stake does not change within an epoch, but a vote account's BLS key or balance could change between the boundary and the capture. Any mismatch moves ranks or keys and makes the signature fail (`INVALID_SIGNATURE`). A wrong denominator near the threshold could also change the quorum result. That is why the summary records the exclusions and the capture lateness.
- **Registered-key trust skips proof re-verification.** Agave makes the same assumption (`PopVerified::new_unchecked`): the vote program verified each proof at registration (SIMD-0387). Proofs are not stored in vote accounts. Monocle still checks every key's encoding, curve and subgroup membership, and rejects the identity point.
- **Genesis scope.** A verified result establishes the genesis/migration anchor under these assumptions. It does not establish ordinary block finality or that the migration completed.

## Dry run, 2026-10-16

A validator capture of mainnet epoch 1059 at `api.mainnet-beta.solana.com` (slots 457,699,812–457,699,837, 15 responses, 5.1 MB) built a rank map for epoch 1060:

- 680 vote accounts were read: 679 V4 and 1 V3.
- 674 were ranked. 5 had no BLS key and 1 had zero stake. None had an invalid or duplicate key, and none was below the minimum or the cutoff.
- The denominator was 437,770,887,568,417,071 lamports, above 2⁵³, so it stays an exact u64.
- All 3,861 cluster nodes reported shred version 50093.
- The node identity in every account's data matched `getVoteAccounts`.

No certificate exists for that epoch, so nothing was verified. The capture is not committed; only one account is kept, as the test vector `fixtures/genesis-capture/mainnet-vote-account.json`.

## Result, 2026-10-21

Mainnet activated the `alpenglow` feature at slot 459,648,000, the first slot of epoch 1064, and its genesis certificate is for slot 459,652,999, the last TowerBFT slot. Verified under **Operator-pinned, keys registered on chain**, the certificate is **VERIFIED** with scope `genesis_migration` (`GENESIS_ANCHOR_ESTABLISHED`, `REGISTERED_KEYS_ACCEPTED`).

| | |
| --- | --- |
| Validator capture | Epoch 1063, slots 459,219,797–459,219,814, 2026-10-20 11:51 UTC: 3,797 slots after the boundary. 15 responses; manifest SHA-256 `ced6cf27…d60ab66`. |
| Certificate capture | Epoch 1064, slots 459,670,307–459,670,311, 2026-10-21 14:55 UTC. 3 responses; manifest SHA-256 `59fcbbb6…5093e3b`. |
| Rank map for epoch 1064 | 679 vote accounts read; 673 ranked. 4 had no BLS key and 2 had zero stake; none was invalid, duplicated, under the minimum or cut off. |
| Context | Shred version 50093, reported by all 3,912 cluster nodes; genesis hash `5eykt4Us…`. |
| Checkpoint digest | `sha256:a83a1b6f8e64efc0ffbb41c8b5404c9bcf06cf4d1d1b3900e468b4ecf8520194` |
| Signers | 446 of 673 ranks |
| Stake | 360,285,044,710,320,374 of 438,565,601,666,629,812 lamports (82.15%); the exact 82% threshold is 359,623,793,366,636,446. |

Three control runs behave as designed:

- **Ordinary operator trust** is `INCONCLUSIVE` with `MISSING_POP`: vote accounts store no proofs.
- **Clearing the highest-staked signer's bit** is `REJECTED` with `INVALID_SIGNATURE`: the signature is well formed but over a different set.
- **Flipping a signature byte** is `REJECTED` with `MALFORMED_SIGNATURE`.

**The bitmap is shorter than the rank map.** It declares 672 positions, not 673. Agave trims it after the highest signing rank (`aggregate_accumulator.rs`: `ranks.last_one() + 1`), and rank 672 did not sign. Ranks 0–671 must match Agave's exactly, or the aggregate would not verify. Whether Agave also ranked the 673rd account (2,129,065 lamports) cannot be seen from the certificate. Either way the result stands: removing it lowers the threshold, and the signers still hold 82.1507%.

**Two surprises during the run.**

- The feature address was sent lamports before activation and briefly existed as a System-owned account. The builder had treated any existing account as active and refused the capture. It now follows Agave: only an account the feature program owns, with an activation slot, is active.
- `getAgGenesisCert` responses from devnet and testnet (already migrated) were inspected before mainnet's appeared. Their shape matched the synthetic fixture.

**What this establishes.** Given an honest `api.mainnet-beta.solana.com` at both capture times and keys registered through the vote program, validators holding 82.15% of the rebuilt epoch-1064 stake signed the migration anchor at slot 459,652,999. It does not authenticate the endpoint, prove ordinary block finality, or show that the migration completed. Those remain the assumptions listed above.

**Reproducing it.** `fixtures/genesis-capture/mainnet/` holds both captures exactly as recorded, with their manifests, and the `built/` outputs. `scripts/check-genesis.sh` rebuilds the checkpoint and bundle from the recorded bytes and requires them to match byte for byte. A builder test pins the figures above and checks the verified, operator-trust and cleared-bit verdicts (the flipped byte was a one-off run), and a browser test verifies the same files through WASM. The validator capture includes `getClusterNodes`, which lists the public gossip, TPU and RPC addresses of 3,912 nodes; it is kept in full because the manifest digests cover every file.

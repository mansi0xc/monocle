# Verifying the mainnet Alpenglow genesis certificate

C19 adds what Monocle needs to verify a real cluster's genesis certificate:

- a capture tool that records raw RPC responses;
- an offline builder that rebuilds the validator rank map Agave uses;
- a converter from a recorded `getAgGenesisCert` response to a proof bundle;
- an explicit **registered-key trust** basis.

The pipeline is tested end to end on a synthetic cluster in exactly the RPC format. It has also been dry-run on mainnet epoch 1059 data. No real genesis certificate has been verified yet: mainnet's appears only after migration.

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

These rules come from Agave v4.3.0, the release mainnet runs. They are identical at Monocle's pinned `master` revision `98535e50`, which diverged from v4.3.0:

1. **Admission (SIMD-0357, `clone_and_filter_for_vat`).**
   - Keep vote accounts that have a BLS key, nonzero stake, and lamports at least equal to the minimum.
   - The minimum is the rent-exempt balance for a 3,762-byte V4 vote account. Once the `alpenglow` feature is active, the VAT burn is added; that depends on slot parameters, so the builder refuses post-activation captures.
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

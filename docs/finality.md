# C4 direct finality and migration evidence

Implemented: 2026-10-11. Policy: `agave-98535e50-direct-finality-v1`.

`monocle_core::finality::verify_evidence(request_bytes, &trust_store)` verifies the original input through C3, then evaluates the declared claim. It accepts the existing `agave-98535e50-base2-v1` certificate profile and the same bounded request/bundle/checkpoint contracts. The separate policy ID identifies the new composition rules without silently changing the certificate-only API.

No caller-provided report, success flag, policy file, or supplied list of “verified” certificates can bypass verification. The only public evaluator entry takes original request bytes and the explicit local trust store. The composition helpers are private. C3's `verify_certificates` remains available for individual checks and still defers block/migration conclusions.

## Supported claims

| Requested claim | Required evidence | Established scope and path |
| --- | --- | --- |
| `certificate_inspection` | Every selected certificate passes C3 | `individual_certificates`; no finality path |
| `block_finality` — fast | A valid FastFinalize certificate for the claimed slot/block, at least 80% | `block_finality`, `fast` |
| `block_finality` — slow | Valid Notar and Finalize certificates for the same claimed slot/context, each at least 60%; Notar binds the block ID | `block_finality`, `slow` |
| `genesis_migration` | A valid Genesis certificate for the claimed anchor slot/block, at least 82% | `genesis_migration`; no ordinary finality path |

Both block and migration claims must include an explicit block ID. A missing target stays inconclusive; the evaluator does not guess it. A finalize vote signs a slot and shred version only. On the slow path, the notarization supplies the block identity and the finalize certificate supplies its own slot-bound aggregate. Different signer sets are permitted; each aggregate must independently meet its threshold. **Stake is never added across certificates.**

All certificates in a bundle share one selected checkpoint. C3 has already checked matching profile, declared chain context, wire shred version, claimed slot, and every block ID that the certificate signs. Conflicting slots/blocks fail before composition. This prevents a valid certificate about a different target from satisfying the claim.

## Complete-bundle policy

Every supplied certificate is selected exactly once by the existing contract, and every selected certificate must pass before composition. A bad or unsupported extra certificate prevents success even if another certificate supplies a fast path. This is a conservative Monocle input policy; a user can prepare a separate bundle containing the evidence they actually want to evaluate.

Ordinary block-finality claims may contain the supported Notar, Finalize, and FastFinalize kinds. Selecting Genesis in that scope is an explicit incompatibility. A genesis-migration claim accepts only Genesis certificates. Thus mixed ordinary/migration evidence cannot be silently relabeled.

Within a compatible, fully checked bundle, the evaluator prefers a supported fast certificate over the slow pair. When there are redundant certificates of the same kind, it records the lexicographically smallest ID as the witness. Slow witnesses are listed in Notar-then-Finalize order. Input order is preserved in the full per-certificate reports, but order cannot change the selected path. Duplicate evidence does not create new votes or satisfy a missing certificate kind.

The policy follows explicit certificate kinds; it does not upgrade a Notar envelope into a FastFinalize envelope. Base3, fallback/multi-message variants, skip/reward kinds, indirect ancestor finality, and authenticated epoch transitions remain unsupported.

## Reports and diagnostics

The new `EvidenceVerificationReport` contains the policy ID; original request, bundle, profile, and checkpoint identities; a typed requested `claim`; every original C3 certificate report; global checks; and assumptions. C3's per-certificate reports are preserved unchanged by composition.

Three fields describe **established evidence**, not the requested goal:

- `evidence_scope`: `unestablished`, `individual_certificates`, `block_finality`, or `genesis_migration`.
- `finality_path`: `fast` or `slow` only for a verified ordinary block-finality claim; otherwise null.
- `supporting_certificate_ids`: deterministic witness IDs only when verification succeeds; otherwise empty.

The requested slot/block stays in `claim`; this field is populated after successful input/profile/context preparation and is null when those prerequisites stop the run. It must not be presented as a verified target when `evidence_scope` is `unestablished`. A report's JSON is informational output, not an authenticated transferable proof token.

| Situation | Overall outcome | Finality-stage diagnostic |
| --- | --- | --- |
| Input, profile, or checkpoint preparation fails | Preserve rejection/inconclusive outcome | `PREREQUISITE_MISSING`, not checked; original diagnostic retained |
| A selected certificate fails | `REJECTED` | `CERTIFICATE_CHECKS_FAILED`, not checked; individual failure retained |
| A selected certificate is unsupported or lacks qualification | `INCONCLUSIVE` | `CERTIFICATE_CHECKS_INCOMPLETE`, not checked |
| Inspection succeeds | `VERIFIED`, individual scope | `FINALITY_NOT_REQUESTED`, not checked |
| Missing block ID | `INCONCLUSIVE` | `MISSING_BLOCK_ID`, not checked |
| Valid Notar without Finalize or fast evidence | `INCONCLUSIVE` | `MISSING_FINALIZATION`, not checked |
| Valid Finalize without Notar or fast evidence | `INCONCLUSIVE` | `MISSING_NOTARIZATION`, not checked |
| Valid certificates selected for an incompatible scope | `REJECTED` | `INCOMPATIBLE_EVIDENCE`, failed |
| Supported fast or slow claim succeeds | `VERIFIED`, block scope | `FAST_FINALITY_ESTABLISHED` or `SLOW_FINALITY_ESTABLISHED`, passed |
| Supported genesis claim succeeds | `VERIFIED`, migration scope | `GENESIS_ANCHOR_ESTABLISHED`, passed |

A false signature is never explained merely as a missing counterpart. Conversely, a missing counterpart does not change a valid certificate's signature result. `REJECTED` describes the supplied evidence/claim; it is not proof that the chain's block can never finalize.

## Meaning and limits of success

Ordinary success is conditional on the pinned protocol's consensus safety assumptions and the externally trusted checkpoint. C4 does not authenticate that checkpoint, determine the actual validator set for a slot, or validate transaction execution, account state, or ancestry. Epoch/genesis labels are compared context, not additional signed vote fields.

Genesis success establishes the certificate's **migration anchor** under that context. It does not prove feature activation, TowerBFT transition conditions, or that a node/the cluster completed migration. Pinned migration code also considers local phase, block ancestry, and activation state; those are outside this evidence bundle. A genesis certificate is therefore never displayed as ordinary block finality.

The tested fixtures are synthetic. No real testnet certificate or the earlier genesis RPC response has been verified. Standard public-key PoP qualification, checkpoint authenticity, resource caps, and stricter Base2 padding restrictions remain as documented in [C3](certificate-verification.md). Product worker/UI, arbitrary-input browser import, tamper/replay, and real capture integration remain later checkpoints.

## Reproduce

```sh
npm ci --ignore-scripts --no-audit --no-fund
bash scripts/check-finality.sh
cargo run --locked --quiet -p monocle-finality-compat
```

The full gate includes previous checkpoints, ten new native integration tests, deterministic generation of 35 C4 cases, actual Rust-to-TypeScript report checks, a fourth WASM import gate, and full native/report equality in actual Chrome/Chromium. These are development harnesses, not the product inspector. No full Agave verifier is executed; compatibility remains source-reviewed narrow policy plus real upstream leaf cryptography.

## Primary-source basis

- [Pinned validated finalization type](https://github.com/anza-xyz/agave/blob/98535e50e9bb38574e6b0c31ff284cd864d3ca88/runtime/src/validated_block_finalization.rs): fast certificates versus independently verified Notar/Finalize pairs for the same slot; block identity from notarization.
- [Pinned consensus pool](https://github.com/anza-xyz/agave/blob/98535e50e9bb38574e6b0c31ff284cd864d3ca88/votor/src/consensus_pool.rs): pairs either arrival order by slot, handles fast finalization directly, and routes Genesis to migration status.
- [Pinned migration state machine](https://github.com/anza-xyz/agave/blob/98535e50e9bb38574e6b0c31ff284cd864d3ca88/votor-messages/src/migration.rs): certificate evidence is distinct from completion of the migration transition.

Exact inspected-source hashes and Monocle-specific restrictions are in the [policy record](../experiments/finality-compat/policy.json). The underlying formats, thresholds, and cryptographic sources remain pinned in the [C3 profile](../experiments/certificate-compat/protocol-profile.json).

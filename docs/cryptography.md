# Cryptographic model and implementation obligations

Status: C1 proves fixed native/browser BLS execution; C2 supplies input and checkpoint boundaries; C3 implements individual certificate verification with checked points, actual selected-key PoPs, and exact stake thresholds. See [certificate-verification.md](certificate-verification.md) for the supported profile and compatibility evidence. C4 adds direct finality/migration composition, described in [finality.md](finality.md). No real-network finality is verified.

## Three questions that must remain separate

1. **Signature validity:** do these qualified public keys verify these exact signed bytes?
2. **Quorum:** do the actual signers carry sufficient stake under the selected context?
3. **Evidence sufficiency:** does this certificate type, or combination of types, establish the claimed kind of finality?

A valid signature is not an answer to all three. Checkpoint authenticity is an additional assumption beneath each question.

## BLS aggregation

BLS uses a pairing-friendly curve; Alpenglow's design uses BLS12-381. Conceptually, validators sign protocol-defined messages, their signatures are combined, and verification relates the aggregate to the selected public keys and signed messages. A curve name alone is not a complete interoperability specification.

The implementation must match the upstream ciphersuite, group placement, hash-to-curve/domain rules, point serialization, and message serialization. Do not select an arbitrary BLS library and assume its defaults match. Use established primitives; Monocle's work is the adapter, evidence evaluation, diagnostics, and reproducibility workflow.

The native and WASM paths must reject malformed points, prohibited identity points, invalid subgroup membership, and otherwise invalid public keys/signatures according to the selected scheme. Requirements and library guarantees must be written down when the dependency is chosen, rather than inferred from a successful happy-path test.

Protocol context: [SIMD-0326](https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/0326-alpenglow.md).

## Proof of possession and rogue-key protection

An aggregate verifier must not accept arbitrary attacker-chosen public keys under assumptions that require proof of key possession. A malicious key registration can otherwise exploit relationships between public keys. Proof-of-possession validation and an authentic validator registry address different problems: possession of a key does not establish its rightful stake.

C3 requires actual standard public-key-bound PoPs for each selected signer. It does not accept a trusted-source Boolean as a substitute; alternative qualification policies require a separate deliberate profile. Imported metadata saying a key was validated is not itself evidence. The UI and report must disclose which route was used. A conversion into an upstream qualified-key type must never bypass the required trust decision.

Relevant implementation surface: [Agave certificate verifier](https://github.com/anza-xyz/agave/blob/master/bls-cert-verify/src/cert_verify.rs).

## Exact signed bytes

Signed messages are not a JSON rendering of the visible fields. A profile must reconstruct the precise upstream representation, including vote kind and context. Notarization and finalization have different payload shapes; adding a displayed block ID to a slot-only signed vote would be a false representation of what the signature covers.

Agave's wire implementation incorporates `shred_version` into vote payload construction. It must come from the intended context, not a guessed default. Cluster labels supplied outside the signed payload need separate checks and must not be described as cryptographically bound unless they actually are.

Changes to a signed field, payload domain, encoding, or corresponding signer aggregate should not verify against the old signature. Preserve original bytes so a user can inspect these differences.

Primary source: [vote payload construction](https://github.com/anza-xyz/agave/blob/master/votor-messages/src/wire.rs).

## Signer selection and stake

A bitmap is meaningful only relative to its encoding rules and canonical rank map. Do not treat its byte length as the validator count, sort an independently fetched account list, or silently drop an unknown rank. Repeated/overlapping selections must not inflate signed stake.

Use the protocol's stake denominator. Recomputing a convenient denominator from just the keys available in a bundle can create a false quorum. A trusted context must identify the denominator and the rank/stake entries used for the relevant epoch.

If a required threshold is the fraction `n/d`, compare exact integers:

```text
signed_stake * d >= total_stake * n
```

Use checked accumulation and sufficiently wide checked intermediate multiplication, with range constraints appropriate to the pinned profile. Percentage displays may be rounded for readability, but a rounded display never decides acceptance. Test immediately below, exactly at, and immediately above each threshold.

Rank-map source: [Agave epoch stakes](https://github.com/anza-xyz/agave/blob/master/runtime/src/epoch_stakes.rs). Threshold selection: [certificate types](https://github.com/anza-xyz/agave/blob/master/votor-messages/src/certificate.rs).

## Evidence profiles

| Evidence | Required interpretation for the proposed supported paths |
| --- | --- |
| Fast finalization | Valid notarization-vote aggregate for the block with at least 80% stake |
| Slow finalization | Required notarization and finalization evidence, each meeting 60%; compatible context and matching slot |
| Genesis/migration | Dedicated genesis-vote profile, currently 82% in the reviewed migration code; report separately |
| Notarization alone | May be individually valid but is insufficient for ordinary block finality |
| Skip, reward, unknown kinds | Do not promote to block-finality evidence; only interpret supported semantics |

These values are profile inputs tied to a selected source revision, not timeless global constants. Slow-path composition must respect the fact that a finalize vote binds a slot, whereas notarization binds the specific block. Additional fallback/indirect-finality paths are unsupported until deliberately implemented and tested.

Sources: [SIMD-0326](https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/0326-alpenglow.md), [genesis migration threshold](https://github.com/anza-xyz/agave/blob/master/votor-messages/src/migration.rs).

## Checkpoint trust

The MVP starts from a separately trusted epoch context. It does not prove that context from a trust root and does not automatically authenticate the next epoch's validator set. A correct computation over an attacker-chosen set would still produce the wrong real-world assurance.

A checkpoint digest can detect a changed file and identify the selected context. It does not show who created the file or why its contents should be trusted. Keep local trust selection separate from imported bundle content, and report the exact checkpoint identity with every result.

## Tamper experiments

A payload mutation tests whether the signature binds the altered data. A signer mutation may fail decoding or change the public-key aggregate and fail signature verification. Neither should be presented as a clean quorum-only experiment.

For a quorum-only example, generate a distinct signature from a deliberately selected below-threshold set of fixture validators. Those keys are deterministic public test material, never operational validator keys. The expected result is signature success and quorum failure. Fixture generation must record what was signed and which keys participated.

## Result semantics

- `VERIFIED`: the declared supported evidence scope is established under the selected trusted context and recorded assumptions.
- `REJECTED`: a supplied supported claim fails a necessary check. This does not prove the block is non-final.
- `INCONCLUSIVE`: missing context/evidence or an unsupported profile prevents the requested conclusion.

Each individual check is `PASS`, `FAIL`, or `NOT_CHECKED`. An execution error or cancellation must be represented clearly and must never inherit a previous successful report.

Certificates in this scope do not verify transaction execution results, account state, or arbitrary ancestry. Those claims require additional authenticated evidence and are outside the initial project.

## What remains to establish in code

C1 pins the BLS dependency and upstream reference, checks one Notar payload layout, verifies uncompressed fixture points and public-key PoPs, and demonstrates browser execution. See [compatibility.md](compatibility.md). C3 adds Base2 decoding, actual standard PoP checks, malformed/identity/subgroup tests, and exact stake thresholds. Authentic historical checkpoint acquisition, application-specific registration qualification, additional malformed-point/fuzz coverage, broader composition paths, and full upstream differential execution remain open.

## C2 input and trust implementation

The core preserves original bytes, rejects duplicate JSON keys and lossy numeric transport, and hashes exact inputs with SHA-256. Checkpoint ranks remain in their declared order, duplicate identities/keys are rejected, and listed stake uses u128 accumulation against the original denominator. These are input invariants, not quorum decisions.

Only an explicit local `TrustStore.pin` decision makes a parsed checkpoint selectable; bundle labels, digest references, and imported Boolean flags cannot do so. Parsing and preflight alone leave PoP bytes unqualified; the separate C3 certificate API performs those checks. See [contracts.md](contracts.md) for precise limits and report semantics.

## C3 implemented scope

The four-kind Base2 profile reconstructs source vote payloads separately from containing wire messages, verifies BLS aggregates using qualified selected keys, and retains the original trusted denominator. Fast-finalization uses a Notar vote payload; finalization signs a slot only. `selected_stake` is a decoded fact and must be read alongside the signature check. This certificate-only API leaves composition unperformed; C4's separate evidence API performs the supported direct paths.

Standard PoPs bind the compressed public key and no additional application payload. They do not authenticate checkpoint membership or prove vote-account registration. Nonzero unused bitmap padding is deliberately rejected, even though upstream truncates it. These restrictions and independently encoded fixture tests are documented in [certificate-verification.md](certificate-verification.md).

## C4 composition and scope

The new evidence API verifies the original bytes through C3 before combining any results. A fast certificate establishes the direct 80% path; the slow path requires separately valid 60% Notar and Finalize evidence for the same slot/context. The notarization binds the block ID. Signer sets may differ, and their stakes are never added across certificates.

Genesis success is migration-anchor evidence only, not proof of network activation or ordinary block finality. Reports leave established scope empty of any finality claim (`unestablished`) on rejection or incomplete evidence. Every selected certificate must pass, including extras. See [finality.md](finality.md) for the deliberate bundle policy, diagnostics, and consensus/checkpoint assumptions.

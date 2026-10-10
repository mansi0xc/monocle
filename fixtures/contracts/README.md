# C2 contract fixtures

Both files are synthetic parser inputs. `bundle.json` contains the four opaque bytes `01 02 03 ff`; **these are not a valid Agave certificate**. The source format and kind are unverified labels used to exercise preflight routing. A compatible checkpoint still produces `INCONCLUSIVE`, because certificate verification is absent.

`checkpoint.json` reuses public deterministic keys and stored PoPs from C1 with invented identities, epoch, genesis hash, and stakes. C2 checks lengths and structural invariants only; C1's BLS experiment separately verifies its own PoP/signature fixture. The first stake exceeds JavaScript's maximum safe integer, and the denominator and bundle slot reach `u64::MAX` to test exact transport.

The bundle's checkpoint reference is SHA-256 of the exact checkpoint file bytes, including the trailing newline. Formatting changes require updating that reference. The cross-target harness pins this checkpoint explicitly as `SyntheticFixture`; loading either JSON file does not establish trust by itself.

These samples define Monocle's container format, not a new consensus wire format. See [contracts.md](../../docs/contracts.md) for limits and semantics.

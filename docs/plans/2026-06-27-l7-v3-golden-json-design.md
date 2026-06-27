# L7.v3 Golden JSON Conformance — Design

**Status:** proposed (v0.10.0 milestone)
**Author:** auto-generated
**Date:** 2026-06-27
**Predecessor:** v0.9.0 L7 oracle (`conformance/oracle/main.go` + 3 conformance tests in `wasm-neovm/tests/conformance.rs`)

## Goal

The v0.9.0 L7 oracle verifies a contract's bytecode shape and final
return stack against `neo-go` v0.105.1's embedded VM. Three tests pass
(`l7_nep17_macro_sample_oracle`, `l7_nep11_macro_sample_oracle`,
`l7_existing_samples_oracle`). The oracle does **not** yet capture
golden expected outputs — each test asserts "oracle returns HALT
with state X" but doesn't pin a specific return value or gas number.

L7.v3 fixes that: commit per-contract golden JSON files that
record the oracle's expected `state`, `gas_consumed`, and
`return_stack` for one well-chosen method per reference contract.
A new test (`l7_v3_golden_json_conformance`) runs the oracle
and diffs the result against the committed JSON.

When the Rust devpack changes (e.g. a translator fix changes
`balanceOf`'s bytecode shape), the diff surfaces the regression
in CI. The developer can either fix the regression or, if
intentional, regenerate the golden JSON with
`UPDATE_L7_GOLDEN=1 cargo test ...`.

## Reference contract set (7)

| # | Contract | Path | Chosen method | Rationale |
|---|----------|------|---------------|-----------|
| 1 | `nep17-token`     | `contracts/nep17-token/`        | `totalSupply` | Read-only, no args, stable, hits NEP-17 entry path. |
| 2 | `nep11-nft`       | `contracts/nep11-nft/`          | `totalSupply` | Read-only, no args, stable, hits NEP-11 entry path. |
| 3 | `nep17-macro-sample` | `contracts/nep17-macro-sample/` | `totalSupply` | Tests the `nep17!` macro from v0.8.0 L5. |
| 4 | `nep11-macro-sample` | `contracts/nep11-macro-sample/` | `totalSupply` | Tests the `nep11!` macro from v0.8.0 L5. |
| 5 | `hello-world`     | `contracts/hello-world/`        | `hello`       | No syscalls; the lowest-fi contract. |
| 6 | `escrow`          | `contracts/escrow/`             | `symbol`      | Read-only string return, validates UTF-8 + manifest. |
| 7 | `timelock-vault`  | `contracts/timelock-vault/`     | `totalSupply` | Read-only, exercises a different storage layout. |

All 7 are read-only (no state mutation) and take no arguments. This
keeps the golden JSON small and the CI diff readable. The escrow
contract's `deposit` method FAULTs in the current oracle (storage
not initialised) — that's an L6 concern, not L7.v3.

## Golden JSON file format

Per-contract file in `wasm-neovm/tests/golden/<contract>.json`:

```json
{
  "method": "totalSupply",
  "arguments": [],
  "expected": {
    "state": "HALT",
    "gas_consumed": 1042,
    "return_stack": ["0"]
  }
}
```

Top-level fields:
- `method` — the contract method invoked.
- `arguments` — the arg list passed to the oracle (currently always
  `[]`).
- `expected` — the oracle's `InvocationResult` (from
  `conformance/oracle/main.go`) with the `state`, `gas_consumed`,
  and `return_stack` fields. `events` and `storage_diff` are
  omitted (oracle doesn't capture them — see *Limitations* below).

## Test logic

`l7_v3_golden_json_conformance` (in `wasm-neovm/tests/conformance.rs`):
1. Iterate over `wasm-neovm/tests/golden/*.json`.
2. For each file: `build_and_emit(contract_dir)`, `run_oracle(contract_dir, method, args)`, parse oracle output, assert `state` / `gas_consumed` / `return_stack` match `expected`.
3. On mismatch, print a unified diff and fail with `UPDATE_L7_GOLDEN=1 cargo test ...` to regenerate.
4. If `UPDATE_L7_GOLDEN=1` is set: overwrite the golden file with the
   current oracle output and print "regenerated".

The test runs after the existing 3 L7 tests in the same
`#[test]` group, so it shares the per-contract `CARGO_TARGET_DIR`
isolation (no wasm-file race).

## Limitations

- **No event capture.** The current oracle uses
  `vm.New()` + `LoadNEFMethod` which doesn't run
  `System.Runtime.Notify` (that needs the full InteropContext).
  The L7.v3 golden JSON is therefore *return-stack-only*. Capturing
  `System.Runtime.Notify` events requires switching to
  `core.NewBlockchain` + `GetTestVM` (the path used by neo-go's
  own test suite); that's a larger milestone (L7.v4 or later).
- **No storage diff.** Same root cause as above.
- **Gas numbers are sensitive to neo-go version.** The
  `conformance/go.mod` pins `neo-go v0.105.1`. A neo-go upgrade
  may change `gas_consumed` by a few gas without changing
  semantics. Mitigation: when the pin is bumped, regenerate the
  golden JSON as part of the bump commit.

## Definition of done

- 7 golden JSON files committed in
  `wasm-neovm/tests/golden/`.
- `l7_v3_golden_json_conformance` test passes on
  `master` with the golden files.
- `UPDATE_L7_GOLDEN=1` env var regenerates the golden files.
- CI (`.github/workflows/conformance.yml`) runs the test.
- CHANGELOG entry for v0.10.0 noting the golden JSON addition.
- One atomic commit: `feat(conformance): L7.v3 golden JSON`.

## Open questions

- Do we want to also capture `gas_consumed` ranges (e.g.
  `>= 1000` rather than `== 1042`)? Pro: more robust to
  neo-go version changes. Con: less precise. Default to
  exact match for now; can loosen later if maintenance
  is a problem.
- Should the L7.v3 test live in the same `conformance.rs`
  file as L7, or in a new `golden_json.rs`? Default to
  same file (it's the same test harness).

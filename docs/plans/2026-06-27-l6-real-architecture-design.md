# L6 Real Executor — Architecture Clarification

**Status:** proposed (v0.12.0 milestone)
**Author:** auto-generated
**Date:** 2026-06-27
**Predecessor:** v0.11.0 L6 minimal cross-call executor

## Goal

The original L6 design proposed "a real, automated test that
compiles a contract to NEF, loads it in the C# NeoVM" plus
"a real wasm32 NeoVM executor that actually executes the
contract-to-contract call on the wasm32 target". The latter
was assumed to require a real cross-call executor in our code.

After auditing the architecture, the wasm32 cross-call
mechanism is **already correct**: the translator emits a
`SYSCALL 0x525B7D62` opcode into the deployed `.nef`, and the
host's NeoVM dispatches `System.Contract.Call` at runtime. The
Rust wrapper's `#[cfg(target_arch = "wasm32")]` path is
**dead code in production** (the wasm module never calls the
Rust wrapper; the host runs the .nef directly).

The v0.11.0 L6 minimal stub (`Err(NeoError::Wasm32CrossCallUnavailable)`)
is therefore the correct production behaviour: it returns
a structured error if the wrapper is ever called on wasm32,
which only happens in host-mode tests.

The actual gap: **no sample contract currently exercises a
cross-contract call**, and the L7 oracle never sees one. This
milestone adds a sample contract that does a cross-contract
call, plus tests that prove the production path (translator
emits SYSCALL → host dispatches) works end-to-end.

## What this milestone does

### 1. New sample contract: `cross-call-wrapper`

A new contract in `contracts/cross-call-wrapper/` that:

- Holds a hardcoded list of (token_hash, method) pairs in
  storage (a "portfolio" of NEP-17 tokens to query).
- Exposes a `balance_of(token_index: i64, account: i64) -> i64`
  method that calls the token's `balanceOf` via
  `NeoVMSyscall::contract_call` and returns the result.
- Exposes a `total_supply_of(token_index: i64) -> i64` method
  that calls the token's `totalSupply` via
  `NeoVMSyscall::contract_call`.

This is a real, on-chain-style contract: in production it
would be deployed and the host's NeoVM would dispatch the
SYSCALLs. The L7 oracle runs it in neo-go's VM (which is
production-equivalent) and verifies the cross-call works.

### 2. New L7 conformance test: cross-call end-to-end

`l7_cross_call_end_to_end` in `wasm-neovm/tests/conformance.rs`:

- Builds `nep17-token` and `cross-call-wrapper` to wasm32.
- Translates both to NEF + manifest.
- In a *first* oracle invocation: calls `totalSupply` on
  `nep17-token` directly; captures the return value.
- In a *second* oracle invocation: calls
  `total_supply_of(0)` on `cross-call-wrapper`; this *should*
  chain through `System.Contract.Call` to `nep17-token`'s
  `totalSupply` and return the same value.
- Asserts the two return values match.

The first invocation proves the target contract works. The
second proves the cross-call dispatches. Same return value =
the cross-call succeeded.

This test is the L6 real-executor proof: it exercises the
production code path (translator emits SYSCALL → host
dispatches) end-to-end through the conformance oracle.

### 3. New golden JSON: cross-call wrapper

A new golden file `wasm-neovm/tests/golden/cross-call-wrapper.json`
capturing the oracle's expected output for the cross-call
invocation. Regenerated via `UPDATE_L7_GOLDEN=1` along with
the other 7 golden files.

### 4. L7.v3 conformance: 8 goldens (was 7)

The L7.v3 test now iterates 8 golden files (the 7 from
v0.10.0 + cross-call-wrapper).

## What this milestone does NOT do

- **No host-mode cross-call executor.** The host-mode
  `neovm_syscall("System.Contract.Call", ...)` still falls
  through to `default_value_for` and returns Null. This is
  a host-mode test fallback; the production path doesn't
  use it. The L6 minimal Result-returning stub covers the
  Rust wrapper; the production path is proven by the L7
  oracle.
- **No removal of the `neo_contract_call` extern.** The
  extern is declared in the Rust source but never called by
  the translator. Leaving it for now (it documents the
  intent and would be used if we switched to architecture B
  in the future).
- **No architecture B switch.** The translator continues to
  emit SYSCALL opcodes. Switching to host-imports would be a
  major redesign; out of scope.

## Affected sites

- `contracts/cross-call-wrapper/Cargo.toml` (new)
- `contracts/cross-call-wrapper/src/lib.rs` (new)
- `wasm-neovm/tests/conformance.rs`: new
  `l7_cross_call_end_to_end` test.
- `wasm-neovm/tests/golden/cross-call-wrapper.json` (new).
- `CHANGELOG.md`: v0.12.0 entry.
- Version bumped 0.11.0 → 0.12.0 across workspace + 22
  contracts (the new cross-call-wrapper plus the existing 21).

## Definition of done

- New `cross-call-wrapper` contract builds to
  `wasm32-unknown-unknown` and translates to a well-formed NEF.
- New `l7_cross_call_end_to_end` test passes: the cross-call
  returns the same value as the direct call.
- New golden JSON committed.
- L7.v3 conformance test iterates 8 goldens (was 7) and
  passes.
- `cargo test --workspace` + `cargo clippy --workspace
  --all-targets --all-features` both green.
- CHANGELOG entry for v0.12.0.
- Merged to `master`, pushed to `origin`.

## Open questions

- Should the `cross-call-wrapper` use a real chain-state
  hash for the target contract, or a placeholder? Default
  to placeholder (consistent with the L8 lookup helper:
  the deploy-time tool can fill in the real hash).
- Should the wrapper also test `System.Runtime.LoadScript`
  and `System.Contract.CallNative`? Default to no for
  v0.12.0 (one cross-call is enough to prove the path;
  more would inflate the milestone).

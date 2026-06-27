# L6 Cross-Call Executor (Minimal) — Design

**Status:** proposed (v0.11.0 milestone)
**Author:** auto-generated
**Date:** 2026-06-27
**Predecessor:** v0.10.0 L7.v3 golden JSON conformance

## Goal

The v0.8.0 B4 fix made the wasm32 cross-call panic-loud rather than
silently returning `NeoValue::Null`. The intent was that any contract
chaining through `NeoVMSyscall::contract_call` / `load_script` /
`contract_call_native` would fail loudly instead of corrupting
state with a fake success.

But "panic" is itself a failure mode — it unwinds the call stack,
crashes the test runner, and provides no structured error for the
contract author to handle. The C# devpack's `Contract.Call<T>`
returns a typed `Result`; a contract author can `match` on it
and decide what to do.

L6 (minimal) replaces the panic-loud wasm32 path with a Result
that returns a new `ContractCallError::Wasm32CrossCallUnavailable`
variant. The host path is unchanged. The `DefaultContractCaller`
API (added in v0.9.0 L9) becomes usable on wasm32 — the typed
`call_typed<T>` returns a `Result<ContractCallError>` that the
contract author can handle, rather than crashing the VM.

**Non-goal:** actually executing the cross-call on wasm32. That
requires a real NeoVM in wasm32 form (months of work). The
minimal L6 surfaces the limitation cleanly via the type system.

## Mechanism

### 1. New error variant

In `rust-devpack/neo-runtime/src/contract_caller.rs`:

```rust
pub enum ContractCallError {
    NoReturn,
    TypeMismatch(String),
    Panicked(String),
    Wasm32CrossCallUnavailable { syscall: &'static str },
    Other(NeoError),
}
```

The `syscall` field records which syscall the user was trying to
call (e.g. `"System.Contract.Call"`, `"System.Runtime.LoadScript"`,
`"System.Contract.CallNative"`).

### 2. Replace panic with Result

In `rust-devpack/neo-syscalls/src/wrapper.rs`, the three
`#[cfg(target_arch = "wasm32")]` panic sites are rewritten to
return `Err(ContractCallError::Wasm32CrossCallUnavailable { syscall })`
instead of `panic!`. The host path is unchanged.

The three sites:
- `NeoVMSyscall::load_script` (line 1103) — `syscall: "System.Runtime.LoadScript"`
- `NeoVMSyscall::contract_call` (line 1150) — `syscall: "System.Contract.Call"`
- `NeoVMSyscall::contract_call_native` (line 1170) — `syscall: "System.Contract.CallNative"`

The two iterator panic sites (`iterator_next`, `iterator_value`)
are **unchanged** — they are translator-bug detectors (Q4), not
cross-call failures, and must remain panic.

### 3. Bridge from NeoError to ContractCallError

The syscall `contract_call` returns `NeoResult<NeoValue>`
(= `Result<NeoValue, NeoError>`), not `Result<NeoValue, ContractCallError>`.
The panic-to-Result conversion happens at the
`DefaultContractCaller::call_raw` boundary, not in the syscall
itself. So:

- `NeoVMSyscall::contract_call` on wasm32 returns
  `Err(NeoError::new("wasm32 cross-call unavailable: System.Contract.Call"))`.
- `DefaultContractCaller::call_raw` translates that `NeoError`
  into `Err(ContractCallError::Wasm32CrossCallUnavailable { ... })`
  when the underlying syscall is the cross-call stub.

Implementation: the syscall returns
`Err(NeoError::new("wasm32 cross-call unavailable: <name>"))`
with a recognised prefix; `DefaultContractCaller` checks for
the prefix and converts. Simpler: the syscall returns a
`NeoError::Wasm32CrossCallUnavailable` variant (new), and
`DefaultContractCaller::from_neo_error` converts. This
avoids string-prefix matching.

### 4. New NeoError variant

In `rust-devpack/neo-types/src/error.rs` (or wherever
`NeoError` lives), add:

```rust
pub enum NeoError {
    // ... existing variants ...
    Wasm32CrossCallUnavailable { syscall: &'static str },
}
```

The `Display` impl produces
`"wasm32 cross-call unavailable: <syscall>"`. The
`message()` method returns the same string.

### 5. Test

`rust-devpack/tests/l6_cross_call.rs` (new file):

- `host_path_contract_call_returns_value` — runs on host
  (`#[cfg(not(target_arch = "wasm32"))]`), calls a known-good
  method on a known contract, asserts `Ok(value)`.
- `wasm32_path_contract_call_returns_wasm32_unavailable` — runs
  on wasm32 (`#[cfg(target_arch = "wasm32")]`), calls
  `DefaultContractCaller::call_raw`, asserts
  `Err(ContractCallError::Wasm32CrossCallUnavailable { syscall: "System.Contract.Call" })`.
- `contract_typed_call_propagates_wasm32_unavailable` — calls
  `call_typed::<NeoValue>`, asserts the same error variant.

The wasm32 tests are skipped on host (the test compiles but
returns early). The host tests are skipped on wasm32. CI runs
both targets.

### 6. CHANGELOG + version bump

v0.11.0. CHANGELOG entry noting:
- New `ContractCallError::Wasm32CrossCallUnavailable` variant.
- New `NeoError::Wasm32CrossCallUnavailable` variant.
- Three wasm32 syscall sites no longer panic (return Result).
- Two iterator panic sites unchanged (Q4 translator-bug detectors).
- No behaviour change on host.
- Cross-call still does not execute on wasm32 (real executor
  deferred to a future milestone).

## Affected sites (file:line)

- `rust-devpack/neo-types/src/error.rs`: add `Wasm32CrossCallUnavailable` variant to `NeoError`.
- `rust-devpack/neo-syscalls/src/wrapper.rs`:
  - `load_script` (line 1103): panic → `Err(NeoError::Wasm32CrossCallUnavailable { ... })`.
  - `contract_call` (line 1150): panic → `Err(NeoError::Wasm32CrossCallUnavailable { ... })`.
  - `contract_call_native` (line 1170): panic → `Err(NeoError::Wasm32CrossCallUnavailable { ... })`.
- `rust-devpack/neo-runtime/src/contract_caller.rs`:
  - Add `Wasm32CrossCallUnavailable { syscall: &'static str }` to `ContractCallError`.
  - Add `From<NeoError> for ContractCallError` arm that converts
    `NeoError::Wasm32CrossCallUnavailable` → `ContractCallError::Wasm32CrossCallUnavailable`.
- `rust-devpack/tests/l6_cross_call.rs`: new test file (3 tests).

## Definition of done

- Three panic sites in `neo-syscalls/src/wrapper.rs` replaced
  with `Err(NeoError::Wasm32CrossCallUnavailable { ... })`.
- New `NeoError` and `ContractCallError` variants with `Display` and
  `From` impls.
- 3 tests in `l6_cross_call.rs` (host + wasm32 coverage).
- `cargo test --workspace` + `cargo test --workspace --target wasm32-unknown-unknown` both green.
- `cargo clippy --workspace --all-targets --all-features` clean.
- CHANGELOG entry for v0.11.0.
- Version bumped 0.10.0 → 0.11.0 across workspace + 21 contracts.
- Merged to `master`, pushed to `origin`.

## Open questions

- Should the L6 work also add a `l6_cross_call` test in the
  wasm32 conformance suite? Default to yes — it documents
  the expected behaviour on wasm32 and catches regressions.
- Should the host path's `NeoVMSyscall::contract_call` also
  gain a typed-error variant, or is the existing
  `Result<NeoValue, NeoError>` sufficient? Default to
  sufficient (no host-path change).

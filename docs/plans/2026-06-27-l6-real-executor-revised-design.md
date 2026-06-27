# L6 Real Cross-Call Executor — Revised Design

**Status:** proposed (v0.12.0 milestone)
**Author:** auto-generated
**Date:** 2026-06-27
**Predecessor:** v0.11.0 L6 minimal cross-call executor; v0.12.0 design that
turned out to be based on a wrong architectural assumption.

## What was wrong with the previous v0.12.0 design

The previous design (`docs/plans/2026-06-27-l6-real-architecture-design.md`)
asserted that "the translator emits SYSCALL opcodes for
`NeoVMSyscall::contract_call`". **This is wrong.**

I implemented the v0.12.0 design as a worktree, added a
`cross-call-wrapper` contract that calls
`NeoVMSyscall::contract_call`, built it to wasm32, translated
to NEF, and inspected the script bytes. The script **does not
contain** a `System.Contract.Call` SYSCALL opcode (`0x41 0x62
0x7D 0x5B 0x52`). The cross-call was *inlined* as the panic
body, not emitted as a SYSCALL.

The reason: `NeoVMSyscall::contract_call` is a **regular Rust
function** in `rust-devpack/neo-syscalls/src/wrapper.rs`,
*not* a wasm extern. When the contract code calls it, the Rust
compiler compiles it as a regular wasm function call (not an
import), and the wasm-neovm translator inlines its body into
the .nef. On wasm32, the function body is the panic
(v0.11.0 Result-returning stub). So the production cross-call
is broken.

The v0.8.0 B4 panic made the cross-call *loud* (not silent),
but the v0.11.0 Result-returning stub is still a dead code
path in production: the actual production code is the panic
body, which gets inlined. The Result never gets used.

This is a **real, silent-on-chain bug**: contracts that
cross-call appear to compile and pass host-mode tests
(`NeoVMSyscall::contract_call` returns the host-mode fallback
value), but on-chain the panic fires. (Well, the Result
fires, but the host's NeoVM sees the *inlined* panic body,
not the Result, so the call FAULTs.)

## Goal

Convert the three cross-call entry points
(`NeoVMSyscall::contract_call`, `NeoVMSyscall::load_script`,
`NeoVMSyscall::contract_call_native`) from regular Rust
functions to **wasm imports** so the translator sees them and
emits the correct SYSCALL opcodes.

After this fix, the production code path is:

1. Contract code calls `NeoVMSyscall::contract_call(...)`.
2. The Rust compiler emits a wasm `call $import` (because
   `NeoVMSyscall::contract_call` is now an extern).
3. The wasm-neovm translator sees the import and emits a
   `SYSCALL 0x525B7D62` (System.Contract.Call) opcode.
4. The host's NeoVM dispatches the call.

The L1 fix already declared externs for many syscalls (the
`#[link(wasm_import_module = "neo")]` block in
`wrapper.rs`). The cross-call externs were not added; that's
the gap.

## Mechanism

### 1. Refactor `NeoVMSyscall` to use externs on wasm32

The three functions become:

- `#[cfg(target_arch = "wasm32")] extern "C" { fn neo_contract_call(...); }`
  (already declared at line 205, but currently unused).
- New: `#[cfg(target_arch = "wasm32")] extern "C" { fn neo_load_script(...); }`
- New: `#[cfg(target_arch = "wasm32")] extern "C" { fn neo_call_native(...); }`

The three `NeoVMSyscall` functions become:

```rust
impl NeoVMSyscall {
    pub fn contract_call(...) -> NeoResult<NeoValue> {
        #[cfg(target_arch = "wasm32")]
        {
            // serialise args to the host's expected format,
            // call the extern, deserialize the result.
            let out = unsafe {
                neo_contract_call(
                    hash_ptr, hash_len,
                    method_ptr, method_len,
                    args_ptr, args_len,
                    call_flags,
                    out_ptr, out_cap,
                )
            };
            // ... decode out into NeoResult<NeoValue>
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // existing host-mode path (unchanged)
        }
    }
}
```

### 2. Argument serialisation

The host's extern expects a specific binary format for args.
We need to know what it is. Options:
- **a) Match the L1 fix's existing externs' format** (ptr+len pairs for
  scalars; a serialised byte array for `NeoArray<NeoValue>`).
- **b) Reuse a C ABI** (c-abi for `NeoArray<NeoValue>` is a struct of
  ptr+len+cap).
- **c) Add a new extern `neo_contract_call_v2` with a different format
  and migrate later.**

The L1 fix's existing externs use ptr+len for scalars. For
arrays, they pass a serialised byte array. We can use the
same pattern: serialise the `NeoArray<NeoValue>` to a byte
array using a stable wire format (e.g. the same one used for
storage values).

**Risk**: the wire format is not yet documented. The L1
externs' semantics are inferred from how the host dispatches
them; we'd need to verify by running a real cross-call on
testnet. **The L1 externs may not work as-is** — the
`neo_contract_call` extern may have been declared speculatively
and never actually wired up to the host.

**Mitigation**: if the externs don't work, fall back to
emitting the SYSCALL opcode **inline in the Rust wrapper**
(the `NeoVMSyscall::contract_call` body on wasm32 emits the
`0x41 0x62 0x7D 0x5B 0x52` bytes into the output buffer
passed to the host). This is a different mechanism but
achieves the same goal.

### 3. Test

`wasm-neovm/tests/l6_real_executor.rs` (same as the
v0.12.0 attempt):

- Build `cross-call-wrapper` to wasm32, translate to NEF.
- Inspect NEF script for `System.Contract.Call` SYSCALL
  (`0x41 0x62 0x7D 0x5B 0x52`).
- Assert present.

After this fix, the test will pass (RED→GREEN).

## Affected sites

- `rust-devpack/neo-syscalls/src/wrapper.rs`:
  - The `neo_contract_call` extern (line 205) is currently
    declared but never called. Make it the wasm32 path of
    `NeoVMSyscall::contract_call`.
  - Add new externs `neo_load_script`, `neo_call_native`.
  - Rewrite the three `NeoVMSyscall` functions to call the
    externs on wasm32.
- `contracts/cross-call-wrapper/`: a sample contract that
  does a cross-call. (The contract code from the failed
  v0.12.0 attempt.)
- `wasm-neovm/tests/l6_real_executor.rs`: the byte-level
  SYSCALL assertion.

## Definition of done

- `NeoVMSyscall::contract_call` is an extern on wasm32.
- `NeoVMSyscall::load_script` is an extern on wasm32.
- `NeoVMSyscall::contract_call_native` is an extern on wasm32.
- The translator sees them as imports and emits the correct
  SYSCALL opcodes.
- The new sample contract builds to wasm32 and translates
  to a NEF that contains the SYSCALL.
- The new test passes.
- `cargo test --workspace` + `cargo clippy --workspace
  --all-targets --all-features` both green.
- CHANGELOG entry for v0.12.0 (revised scope).
- Merged to `master`, pushed to `origin`.

## Open questions

- The exact wire format for the args/output of the cross-call
  externs. If the L1 externs don't work, we need to design
  a new format and have the host implement it. **This
  requires coordination with the host (N3 node) team**;
  may not be feasible in this session.
- Whether the host's NeoVM even supports cross-calls from
  wasm-deployed contracts to other contracts. If it doesn't,
  no amount of work in our devpack will fix it.
- Should we make the cross-call path opt-in via a cargo
  feature? Default to no (always-on; this is a correctness
  fix).

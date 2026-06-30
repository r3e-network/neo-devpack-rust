<!-- Copyright (c) 2025-2026 R3E Network. SPDX-License-Identifier: MIT -->
# Testing & Debugging Rust contracts on Neo N3

The Rust-on-Neo toolchain gives the same kinds of testing and debugging you get
on Solana (`solana-program-test`) or Move (the Move VM harness), spanning three
complementary layers plus first-class disassembly/tracing. Everything below
works for **any** source the toolchain accepts (Rust, and the Move/Solana
cross-chain front-ends) because the layers operate on the translated NEF /
NeoVM, not on the source language.

## The three testing layers

| Layer | Crate / tool | Runs | Use for | Needs |
|---|---|---|---|---|
| 1. Unit / logic | [`neo-test`](../rust-devpack/neo-test) | your method **natively** against a mock runtime/storage | fast logic checks, storage/runtime mocking, assertions | nothing |
| 2. Bytecode e2e | [`neo-vm-test`](../neo-vm-test) | the **translated NEF** on a real NeoVM (neo-go), with storage + runtime syscalls serviced | proving the compiled bytecode behaves — pure compute **and** stateful storage/witness/notify (catches translator/lowering bugs) | Go (builds the oracle once) |
| 3. Full-chain e2e | [`integration-tests`](../integration-tests) + Neo Express | a deployed contract on a **live chain** | deploy/upgrade, multi-contract calls, native-contract interop | Neo Express (`neoxp`) |

### Layer 1 — unit tests with mocks (`neo-test`)

```rust
use neo_test::*;

#[test]
fn transfers_require_witness() {
    let mut env = TestEnvironment::new();
    env.set_storage(b"balance:alice", &100i64.to_le_bytes());
    env.add_witness(b"alice");

    // Invoke your method natively; it reads/writes the mock env.
    let ok = env.call_method("transfer", &[], || MyToken::transfer_impl(&env));

    assert!(ok);
    env.assert_storage().assert_contains(b"balance:bob");
    env.assert_runtime().assert_witness(b"alice");
}
```

Fast and dependency-free, but it runs your **Rust source**, not the emitted
NeoVM bytecode.

### Layer 2 — real-VM bytecode tests (`neo-vm-test`)

This is the Solana-`program-test` equivalent: compile the contract, translate it
to a NEF, and execute methods on a real NeoVM, asserting the **actual on-chain
return value**.

```rust
use neo_vm_test::Contract;

#[test]
fn arithmetic_on_real_vm() {
    let c = Contract::compile("contracts/feature-arithmetic").unwrap();
    c.invoke("calc", &[20.into(), 6.into()]).assert_returns_i64(81);
    c.invoke("bitcount", &[255.into()]).assert_returns_i64(64);
    c.invoke("bigmath", &[100.into(), 7.into()]).assert_returns_i64(40);
}
```

Run with `make vm-test` (or `cargo test --manifest-path neo-vm-test/Cargo.toml`).
On first use it builds `wasm-neovm` and the conformance oracle (`go build`).
Point `$NEO_VM_ORACLE` / `$NEO_WASM_NEOVM` at prebuilt binaries to skip that.

**Stateful** tests seed storage / signers / runtime env with the builder and
read back the storage diff and emitted events:

```rust
let c = Contract::compile("contracts/feature-storage-raw").unwrap();
c.call("getKeyed")
    .arg(7)
    .storage(&[7], &999i64.to_le_bytes())  // seed storage the contract reads
    .signer([0u8; 20])                     // check_witness(this) == true
    .time(1_700_000_000_000)               // System.Runtime.GetTime
    .run()
    .assert_returns_i64(999);
```

The oracle services `System.Storage.*` (Get/Put/Delete/contexts),
`System.Runtime.*` (Log/Notify/CheckWitness/GetTime/GetTrigger/GetNetwork/…) and
`System.Contract.GetCallFlags`, mirroring neo-go's interop ABI, so storage
round-trips, witness checks and notifications all work. `VmOutcome` carries the
return stack, `storage_diff`, and `events`, with assertions: `assert_returns_i64`,
`assert_returns_bool`, `assert_halt`, `assert_fault`, `assert_storage(key, val)`,
`assert_event(name)`. Deploy/upgrade, multi-contract calls and native-contract
interop are out of scope for the in-process VM — use Layer 3 for those.

### Layer 3 — full-chain tests (Neo Express)

For storage/runtime/deploy and multi-contract flows, deploy to a local
[Neo Express](neoexpress-integration.md) chain. See
`integration-tests/tests/neo_express.rs` and `make smoke-neoxp`.

## Debugging

`neo-vm-test` exposes the oracle's debugger from Rust:

```rust
let c = Contract::compile("contracts/feature-arithmetic").unwrap();

// Disassemble the NeoVM script: "IP: OPCODE operand"
println!("{}", c.disassemble().unwrap());

// Single-step trace a method: each instruction + the post-step eval stack.
println!("{}", c.trace("calc", &[20.into(), 6.into()]).unwrap());
```

The same is available directly from the oracle binary for any NEF:

```
oracle -disasm contract.nef -from 0 -count 80
oracle -trace  -in request.json -from 0 -to 1000000   # request = {nef_path,manifest_path,method,arguments,...}
```

A trace ends with `--- final state=<halted?> steps=N ---`; the per-line
`estack=[...]` (top first) makes a wrong return value or a FAULT obvious — this
is exactly how the translator's `eqz`/`div_u`/popcount bugs were pinpointed.

For deeper, automated correctness checking there is a **differential fuzzer**
([`conformance/fuzz`](../conformance/fuzz/README.md)): it runs the same op two
ways (native Rust vs real NeoVM) over thousands of inputs and flags any
divergence. Run it with `python3 conformance/fuzz/diff_fuzz.py --root "$PWD"`.

## At a glance

```
write Rust contract ──► cargo build --target wasm32-unknown-unknown
        │                         │
        │ Layer 1                 ▼  wasm-neovm  ──► NEF + manifest
        ▼                         │        │
  neo-test (mocks)                │        ├─► neo-vm-test (Layer 2: real VM)
                                  │        ├─► oracle -disasm / -trace (debug)
                                  │        └─► Neo Express deploy (Layer 3)
```

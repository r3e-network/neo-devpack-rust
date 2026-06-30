<!-- Copyright (c) 2025-2026 R3E Network. SPDX-License-Identifier: MIT -->
# neo-vm-test

Run **compiled** Neo N3 contracts on a **real NeoVM** from ordinary Rust tests —
the Rust-on-Neo analogue of Solana's `solana-program-test`.

```rust
use neo_vm_test::Contract;

#[test]
fn it_runs_on_chain() {
    let c = Contract::compile("contracts/feature-arithmetic").unwrap();
    c.invoke("calc", &[20.into(), 6.into()]).assert_returns_i64(81);

    // debugging, too:
    println!("{}", c.disassemble().unwrap());
    println!("{}", c.trace("calc", &[20.into(), 6.into()]).unwrap());
}
```

`Contract::compile` builds the crate to wasm32 and translates it with
`wasm-neovm`; `invoke` executes the method on the neo-go VM via the conformance
oracle and returns a `VmOutcome` with assertion helpers (`assert_returns_i64`,
`assert_returns_bool`, `assert_halt`, `assert_fault`). `disassemble`/`trace`
expose the oracle's debugger.

- Run: `make vm-test` (from the repo root), or `cargo test` here.
- Binaries are resolved from `$NEO_VM_ORACLE` / `$NEO_WASM_NEOVM`, else built on
  first use (`go build` for the oracle, `cargo build` for the translator).
- The default oracle is a bare VM (pure compute / arithmetic / control-flow /
  bit & BigInteger). For storage/runtime/deploy use Neo Express
  (`integration-tests`). See [the testing & debugging guide](../docs/testing-and-debugging-guide.md).

This is a dev-only crate (`publish = false`).

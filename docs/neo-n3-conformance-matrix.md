# Neo N3 Conformance Matrix (neo-devpack-rust)

Date: 2026-02-14

This matrix is based on fresh local verification runs:

- `cargo test --manifest-path wasm-neovm/Cargo.toml --all-features`
- `cargo test --manifest-path move-neovm/Cargo.toml --all-features`
- `cargo test --manifest-path rust-devpack/Cargo.toml --all-features`
- `cargo test --manifest-path solana-compat/Cargo.toml --all-features`
- `cargo test --manifest-path integration-tests/Cargo.toml --all-features`
- `make test-contracts`
- `make test-cross-chain`
- `make smoke-neoxp`

## Verdict

- Neo N3 Rust contract flow is strongly validated for the implemented subset.
- Full feature parity with all Wasm/NeoVM surfaces is not claimed.

## Scope Boundaries (Explicitly Unsupported)

- Multiple memories are rejected (`docs/wasm-pipeline.md:214`).
- Reference types beyond `funcref` are rejected (`docs/wasm-pipeline.md:215`).
- Function signatures are constrained to `i32/i64` params and at most one return (`docs/wasm-pipeline.md:227`).
- Floating-point and SIMD operations are rejected (`docs/wasm-pipeline.md:228`).

## Contract Syntax & Feature Surface

| Area | Status | Evidence |
|---|---|---|
| `#[neo_contract]`, `#[neo_method]`, manifest macros | Supported + tested | `rust-devpack/tests/neo_contract_exports_tests.rs`, `rust-devpack/tests/manifest_overlay.rs` |
| Core runtime/storage/context/crypto/json helpers | Supported + tested | `rust-devpack/tests/comprehensive_test_suite.rs`, `rust-devpack/tests/neo_runtime_tests.rs` |
| NEP-17 and NEP-11 contract patterns | Supported + tested | `wasm-neovm/tests/integration_tests.rs`, `scripts/neoxp_smoke.sh:203` |
| Cross-chain Solana/Move adapters (as implemented) | Supported + tested | `wasm-neovm/tests/cross_chain_tests.rs`, `wasm-neovm/tests/solana_move_integration.rs`, `scripts/neoxp_smoke.sh:278` |

## System Syscalls (Canonical Table)

Reference canonical consistency check: `wasm-neovm/tests/syscall_consistency.rs:78`

| Syscall | Supported | Registry/Hash Verified | Direct Syscall-Specific Test | Example Direct Evidence |
|---|---|---|---|---|
| `System.Contract.Call` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Contract.CallNative` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Contract.CreateMultisigAccount` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Contract.CreateStandardAccount` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Contract.GetCallFlags` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Contract.NativeOnPersist` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Contract.NativePostPersist` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Crypto.CheckMultisig` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Crypto.CheckSig` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Iterator.Next` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Iterator.Value` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.BurnGas` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.CheckWitness` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.CurrentSigners` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.GasLeft` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.GetAddressVersion` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.GetCallingScriptHash` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.GetEntryScriptHash` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.GetExecutingScriptHash` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.GetInvocationCounter` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.GetNetwork` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.GetNotifications` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.GetRandom` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.GetScriptContainer` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.GetTime` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.GetTrigger` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.LoadScript` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.Log` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.Notify` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Runtime.Platform` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Storage.AsReadOnly` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Storage.Delete` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Storage.Find` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Storage.Get` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Storage.GetContext` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Storage.GetReadOnlyContext` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `System.Storage.Put` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |

## Extended Native Methods (`Neo.Crypto.*`)

Reference hash/lookup validation: `wasm-neovm/src/syscalls.rs:92`

| Native Method Descriptor | Supported | Hash/Lookup Verified | Direct Descriptor-Specific Test | Example Direct Evidence |
|---|---|---|---|---|
| `Neo.Crypto.SHA256` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `Neo.Crypto.RIPEMD160` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `Neo.Crypto.Murmur32` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `Neo.Crypto.Keccak256` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `Neo.Crypto.Hash160` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `Neo.Crypto.Hash256` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |
| `Neo.Crypto.VerifyWithECDsa` | Yes | Yes | Yes | `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs` |

## Runtime Neo Express Validation

`make smoke-neoxp` deploys the bundled contract suite and invokes the runtime-safe smoke subset, including:

- HelloWorld (`scripts/neoxp_smoke.sh:200`)
- NEP-17 (`scripts/neoxp_smoke.sh:203`)
- NEP-11 (`scripts/neoxp_smoke.sh:208`)
- Deploy validation for stateful Governance/Oracle/Marketplace flows
- Cross-chain Solana + Move flows (`scripts/neoxp_smoke.sh:278`)

## Interpretation

- "Supported" means recognized by translator/registry and validated by current test suite.
- Direct descriptor-specific coverage is validated by `wasm-neovm/tests/neo_n3_direct_syscall_coverage.rs`, which translates each canonical descriptor and verifies the emitted SYSCALL hash matches the expected entry.

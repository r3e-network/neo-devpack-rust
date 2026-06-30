<!-- Copyright (c) 2025-2026 R3E Network. SPDX-License-Identifier: MIT -->
# Feature-coverage sample contracts

Eleven sample contracts under `contracts/feature-*` that exercise the **entire**
toolchain surface — every value type, every storage facade, the runtime/syscall
surface, crypto, events, manifest macros, cross-contract calls, the NEP standard
traits, and every WASM op category the translator supports. Unlike the
domain-demo contracts (token, escrow, …), these are organised by *feature* so
each language/devpack/translator capability has an explicit, validated example.

Every sample is validated end-to-end: `cargo build --target wasm32-unknown-unknown`
→ `wasm-neovm` translation → **VALID** NEF (NEF3 magic, 64-byte compiler, method
tokens, double-SHA256 checksum) + manifest (ABI/permissions/events schema). The
pure-compute samples are additionally executed on a **real NeoVM** (the neo-go
conformance oracle) and their results diff-checked against the Rust semantics
(23 cases across arithmetic/control-flow/memory — all match).

| Contract | Crate | Covers |
|---|---|---|
| `feature-arithmetic` | `arithmetic-neo` | all integer add/sub/mul/div/rem (signed+unsigned), and/or/xor/not, shl/shr/rotl/rotr, **clz/ctz/popcnt**, comparisons, wrap/extend/sign-extend, full `NeoInteger` ops incl. `try_div`/`try_rem`/`fits_in_neovm`/conversions |
| `feature-control-flow` | `control-flow-neo` | block/loop/if/else, br/br_if, **br_table**, direct calls, **call_indirect** (fn pointers), recursion, early return, select/drop/locals |
| `feature-memory` | `memory-neo` | heap `Vec` (dlmalloc) + memory.grow, memory.fill/copy, `static`/`static mut` globals, stack arrays/slices |
| `feature-storage-raw` | `storage-raw-neo` | full heap-free `RawStorage` + `RawKeyBuilder` + `RawStorageGet` (all 3 get_into outcomes), typed put/get, integer-keyed entries |
| `feature-storage-typed` | `storage-typed-neo` | `NeoStorage` byte-string facade, `NeoStorageContext` (read-only), low-level `NeoVMSyscall::storage_*` |
| `feature-runtime` | `runtime-neo` | scalar runtime/syscall surface: get_time, check_witness(*), get_trigger/network/address_version/random/invocation_counter/gas_left/call_flags, script-hash i64 forms, burn_gas, log, `NeoRuntimeContext`. All three `NeoResult<_>` export kinds |
| `feature-crypto` | `crypto-neo` | `NeoCrypto::sha256/ripemd160/keccak256/keccak512/murmur32` (pure-Rust hashers, correct digests) |
| `feature-types` | `types-neo` | full value-type surface: `NeoByteString`/`NeoString`/`NeoBoolean`/`NeoArray`/`NeoMap`/`NeoStruct`/`NeoValue` (+ all `From`/accessors), `Hash160`/`Hash256`, `NeoIterator`, `serialise_*`, size constants, `BigInt` |
| `feature-events-manifest` | `events-manifest-neo` | `#[neo_event]` + `.emit()`, `notify_event`, `neo_manifest_overlay!`/`neo_supported_standards!`/`neo_permission!`/`neo_trusts!`/`neo_safe_methods!`, `#[neo_method(safe)]`, `#[neo_entry]` |
| `feature-contract-calls` | `contract-calls-neo` | `contract_call`, `call_typed`, `DefaultContractCaller::call_raw`/`call_typed`, `contract_call_native`, all 11 native-contract hash constants/helpers, `NeoError`/`NeoResult` |
| `feature-standards` | `standards-neo` | NEP trait impls (`Nep17Token`, `Nep11Token`, `Nep24Royalty`, `Nep27Receiver`, `Nep26Receiver`, `Nep29Deploy`, `Nep30Verify`, `Nep31Destroy`, `Nep22Update`) + `compute_bps_royalty`/`NEP_BPS_DENOMINATOR` |

The `nep17!`/`nep11!` declarative macros are covered by the existing
`nep17-macro-sample`/`nep11-macro-sample` contracts.

## Findings surfaced + fixed while building these samples

**Fixed (real correctness bugs):**
- **Type-strict `EQUAL` broke every chained comparison** (found by the
  differential fuzzer — see [`conformance/fuzz`](../conformance/fuzz/README.md)).
  `i32/i64.eqz` lowered to `PUSH0; EQUAL` and `eq`/`ne` to `EQUAL`/`NOTEQUAL`, but
  NeoVM's `EQUAL` is type-strict so `Boolean(false) == Integer(0)` is false. LLVM
  chains comparisons (`if x==0` → `(x==0)==0`), feeding a comparison's Boolean
  result into the next `eqz`/`eq`, which always misfired — so guarded branches
  silently fell through. This corrupted `div_u`/`rem_u` and `BigInteger`
  negative `>>`, and affected any contract with a chained comparison / `if x==0`
  / `match` / boolean logic (valid NEF, wrong on chain). Fixed: `eqz`→`NOT`,
  `eq`→`NUMEQUAL`, `ne`→`NUMNOTEQUAL`, and the `br_table` selector compare→
  `NUMEQUAL`. Verified by 30,888/30,888 op cases + 90/90 memory cases on the real VM.
- **clz/ctz/popcnt produced wrong results on chain.** The SWAR popcount trick
  `(x * 0x0101…01) >> shift` relied on 64-bit multiply wraparound; NeoVM
  `BigInteger` is arbitrary-precision, so the product polluted the result. Fixed
  by masking the product to the operand width before the shift
  (`wasm-neovm/.../bits/helpers.rs`). This also unblocked all `NeoInteger`/BigInt
  math, because num-bigint uses these intrinsics internally (the `bigmath` sample
  FAULTed before the fix, now returns the correct value).
- **5 scalar runtime/protocol syscalls were unreachable** (extern link-names did
  not match the canonical alias). Added aliases for `protocol_get_network`,
  `protocol_get_address_version`, `protocol_get_trigger`, `runtime_get_gas_left`,
  `runtime_get_call_flags` (`wasm-neovm/src/neo_syscalls.rs`).
- **`serialise_*`, `BigInt`, `MAX_NOTIFICATION_SIZE`/`MAX_STACK_SIZE` were not
  re-exported** from `neo-devpack` (contracts couldn't name them). Added to the
  crate root + prelude.

**Known wasm32 limitations (documented in each sample, intentionally not shipped
as broken-but-valid NEFs):**
- Byte-string/array-returning syscalls (script-hash byte form, `platform`,
  `get_notifications`, `current_signers`, `get_script_container`,
  `create_*_account`, `load_script`) use an `out_ptr,out_cap` ABI the translator
  doesn't marshal — only scalar/i64 forms are bridged.
- Crypto **signature** syscalls (`check_sig`/`check_multisig`/`verify_with_ecdsa`)
  are not marshalled (the pure-Rust `NeoCrypto` hashers are used instead).
- `#[neo_event]::emit()` emits **name-only** on wasm32 (state-carrying
  `runtime_notify_with_state` is not bridged) — events carry no field data
  on-chain today.
- `contract_call`/`call_typed`/native calls emit the SYSCALL but return `Null`
  on wasm32 (args/flags not marshalled) — call path only.
- `#[neo_safe]` is currently unusable (anonymous-const overlay illegal in `impl`,
  marks a non-exported name standalone); use `#[neo_method(safe)]` or
  `neo_safe_methods!`.

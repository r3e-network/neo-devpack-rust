# Wasm → NeoVM Capability Matrix

This document tracks the current translation coverage. It should be kept up to date whenever new Wasm language features or NeoVM helpers are implemented.

| Area | Status | Notes |
|------|--------|-------|
| Numeric types | ✅ `i32`, `i64`<br>❌ `f32`, `f64`, SIMD (`v128`) | Floats and SIMD instructions bail out in `translate_function`. |
| Reference types | ✅ `funcref` (only)<br>❌ GC refs | Only `funcref` is allowed; `externref`, `anyref`, and GC proposal types are rejected. |
| Return values | ✅ Single result<br>❌ Multi-value | Multi-value returns bail out early. |
| Memories | ✅ Single memory (32-bit index)<br>❌ `memory64`, shared memories, multiple memories | Translator rejects modules declaring additional or shared memories. |
| Tables | ✅ `funcref` tables (one or more)<br>❌ `table64`, typed tables, declared segments | Multiple `funcref` tables are supported; `table64`, shared tables, and non-`funcref` element types are rejected. |
| Bulk-memory ops | ✅ `memory.copy`, `memory.fill`, `memory.init`, `data.drop` with helpers<br>❌ Passive element ops beyond current helpers | Some helper paths still bail if initialisation helpers are unavailable. |
| Atomics | ❌ | No mapping exists for atomic instructions. |
| SIMD | ❌ | SIMD opcodes are rejected. |
| Exceptions (`exception-handling`) | ❌ | No mapping; translator treats unknown operators as unsupported. |
| Panic semantics | ⚠️ Uses `ABORT` | Rust panics that survive to Wasm become `ABORT`; richer diagnostics TBD. |
| Gas accounting | ❌ | Gas model integration not yet designed; relies on NeoVM defaults. |
| Host interop | ✅ Syscalls generated via pre-hashed table<br>✅ Opcode imports<br>✅ Friendly `neo::` aliases resolved at translation time<br>✅ Common `env::` shims (`memcpy`/`memmove`/`memset`) | Translator recognises canonical `syscall::*` descriptors, DevPack-style `neo::*` imports, and bridges common C runtime shims emitted as `env::` imports, lowering everything to the appropriate NeoVM helpers with overlap-safe semantics. |
| Manifest emission | ✅ ABI (methods, safe flag)<br>✅ Custom overlays merged<br>✅ Auto method-token inference<br>⚠️ Complex type annotations limited to integers | `wasm-neovm` infers method tokens for literal `SYSCALL` patterns, including `System.Contract.Call` (with hash/method/flags) and zero-hash placeholders for other interops. |
| NEF metadata | ✅ `nefSource`, `nefMethodTokens` | See `metadata/` helpers. |
| Testing | ✅ Comprehensive translator suites (~120 focused tests) | Arithmetic, control-flow, memory, table, syscall, and optimisation suites exercise the lowering logic end-to-end. |

## Supported Smart-Contract Subset

The translator intentionally targets deterministically executable Wasm suitable for on-chain smart
contracts. The currently supported surface is:

- **Numeric operations**: `i32`/`i64` integer arithmetic, bit-twiddling, comparisons. No floating-point, SIMD or saturating integer instructions.
- **Control flow**: structured `block`/`loop`/`if`/`else`/`br`/`br_if`, function calls, `call_indirect` via `funcref` tables, and `return`. Tail calls, exception handling, and indirect references to non-function types are rejected.
- **Memory model**: one 32-bit linear memory (`memory 0`) with deterministic helpers for loads, stores, grow/size, passive/active data segments, and bulk-memory opcodes that operate on that memory. Shared memories, `memory64`, multiple memories, and threads are not accepted. Exported start functions are wrapped in an init stub that runs exactly once, preventing recursive initialisation when start bodies touch memory.
- **Tables**: one or more `funcref` tables with element segments populated through the runtime helpers. Table64, typed tables, and GC/reference-proposal tables are out of scope.
- **Globals & locals**: mutable and immutable integer globals, locals, and parameters. The start function must have signature `() -> ()`.
- **Imports**: `syscall::`, `opcode::`, and whitelisted `neo::` imports that map to NeoVM syscalls; other host imports are rejected. `env::mem*` shims are allowed and lowered to bounded helpers.
- **Determinism**: no floating-point, random host interaction, or instructions with implementation-defined behaviour. Panics translate to `ABORT`, preserving consensus safety.

> **Compiler guidance**: when compiling Rust or C/C++ contracts use `wasm32-unknown-unknown` (or equivalent) and disable features that emit unsupported instructions (e.g. `RUSTFLAGS="-C target-feature=-simd128"`). Enabling bulk-memory is fine; atomics, threads, and exceptions must remain disabled. For C/C++, prefer the repository's `scripts/build_c_contract.sh`, which passes `-nostdlib`/`-fno-builtin` to avoid `env::` imports such as `memcpy`.

This matrix complements the capability table below and serves as the contract for what the translator will accept.

## Verification & Guardrails

To keep the supported surface reliable, run the following checks for every change:

- `cargo test` (default target) – exercises translator unit tests plus the table/memory integration suites.
- `cargo test neo_opcodes_match_reference -- --nocapture` – fails if the generated opcode table drifts from `neo/src/Neo.VM/OpCode.cs`.
- `cargo test translate_table_* translate_memory_* --test table_tests -- --ignored` (or a CI pattern) – optional focused runs for table/memory helpers when touching runtime code.
- Consider compiling a real contract (e.g. with the provided `scripts/build_contract.sh`) to ensure the produced NEF passes opcode validation before deployment.
- Reject Wasm binaries that contain unsupported instructions (floats, SIMD, atomics, multiple memories) during CI by scanning custom sections or leveraging the translator’s early `bail!` paths.
- When Neo upstream changes opcodes/syscalls, regenerate the metadata using `cargo build --build-plan` and re-run the opcode/syscall consistency tests before merging.

## Open Items

1. **Floating-point lowering** – Decide on semantics for `f32`/`f64` and map key arithmetic/logical operations to NeoVM equivalents (likely via runtime helpers).
2. **SIMD & Atomics** – Either support or provide tooling to pre-sanitise Wasm modules (e.g., clang flags to avoid emission).
3. **Multi-value returns** – Extend function call plumbing to push/pop multiple stack values and encode them into NeoVM tuples or arrays.
4. **Memory64 / Multiple memories** – Evaluate feasibility; NeoVM exposes a single linear memory today.
5. **Gas model** – Establish a translation-time estimator or post-pass instrumentation.
6. **ABI enrichment** – Support composite Rust types (structs/enums) when generating manifest parameter metadata.

Contributors should update this table as new features land.

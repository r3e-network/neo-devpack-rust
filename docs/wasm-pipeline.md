# WebAssembly Contract Pipeline

This document outlines the steps required to make the neo-llvm toolchain operate on a Rust → Wasm → NeoVM conversion pipeline. The intent is to reuse the existing Rust developer experience while performing bytecode generation from WebAssembly modules instead of relying on the unfinished NeoVM LLVM backend.

For the normative translator specification (acceptance rules, operator mappings, manifest/NEF semantics) see [`spec/wasm-neovm-spec.tex`](../spec/wasm-neovm-spec.tex) or build the PDF via `make -C spec`.

## 1. Overview

```
Rust Contract  ──cargo build──▶ Wasm module ──wasm-neovm──▶ NeoVM script ──NEF writer──▶ NEF + manifest
```

1. **Rust compilation**: Contracts are built for `wasm32-unknown-unknown` using stable Rust. This emits a `.wasm` file that embeds the contract logic as Wasm bytecode.
2. **Wasm → NeoVM translation**: The `wasm-neovm` translator consumes the Wasm module, lowers supported instructions to NeoVM opcodes, and produces a raw NeoVM script (byte vector).
3. **NEF + manifest emission**: The translator packages the script together with metadata to yield a `.nef` artifact and a JSON manifest.

The translator can initially focus on the minimal instruction subset required by simple contracts and expand over time.

## 2. Rust build changes

- Add a dedicated target dir such as `contracts/<name>/Cargo.toml` with `crate-type = ["cdylib"]` so `cargo build --target wasm32-unknown-unknown --release` produces `<name>.wasm`.
- The repository provides `scripts/build_contract.sh` which builds the Wasm artefact and runs the translator.
- Plain C contracts can use `scripts/build_c_contract.sh`, a thin wrapper around clang that mirrors the same Wasm → NeoVM flow.
- A future improvement is to expose a `cargo neovm` subcommand that shells out to both the Wasm build and the translator.

Start sections are wrapped in an init-aware stub: the stub checks the runtime `INIT_FLAG`, runs the memory/table/global initialiser once, and only then executes the exported start body. This prevents start functions that touch memory from recursively entering the initialiser and makes the stub offset the one recorded in the manifest for exported `start` methods (that is, `abi.methods[].offset` points at the stub, not the raw body).

Example manifest slice and matching stub (simplified):

```json
{
  "abi": {
    "methods": [
      { "name": "start", "offset": 12, "parameters": [], "returntype": "Void", "safe": false }
    ]
  }
}
```

```
000c LDSFLD0
000d JMPIF_L +0c    ; skip init helper if already run
0012 CALL_L  +1234  ; runtime init helper
0017 LDSFLD(startSlot)
0018 JMPIF_L +05    ; skip start body if already executed
001d CALL_L  +5678  ; start body (import or defined)
0022 PUSH1
0023 STSFLD(startSlot)
0024 RET
```

Only the stub offset (here `0x0c`) is written to the manifest; the start body retains its own offset for internal calls.

### 2.1 Enforcing the supported Wasm surface

The workspace ships a `.cargo/config.toml` entry for `wasm32-unknown-unknown` that sets

```
-C panic=abort -C target-feature=-simd128,-atomics,-reference-types,-multivalue,-tail-call
```

Any attempt to compile a contract that uses SIMD, atomics, reference types beyond `funcref`, tail calls, or multi-value returns fails during `cargo build` before the translator is invoked. `scripts/build_contract.sh` inherits those defaults and adds `-C opt-level=z` / `-C strip=symbols`; the environment variable `NEO_WASM_RUSTFLAGS` can override the entire flag string when required (for example, to re-enable bulk-memory experiments).

Note: the default script opts for a stable-safe feature mask (`-C target-feature=-simd128`) to avoid rustc warnings about unstable feature flags. If you need stricter masking (e.g., disabling atomics/reference-types/multivalue/tail-call) set `NEO_WASM_RUSTFLAGS` explicitly; rustc may emit warnings for those unstable feature names on stable toolchains.

The C helper (`scripts/build_c_contract.sh`) mirrors the restriction via `-mattr=-simd128` by default so Clang emits an error instead of producing SIMD bytecode. Projects that compile Wasm outside these scripts should mirror the same mask; if you need to disable additional features, extend `DEFAULT_CFLAGS` or pass extra `-mattr` flags after the first `--`.

## 3. Translator architecture (`crates/wasm-neovm`)

`wasm-neovm` lives in the repository as a Rust crate that depends on `wasmparser`, `serde_json`, `anyhow`, and `clap`. Build-time helpers scan the upstream `neo` repository to generate exhaustive opcode/syscall tables so every NeoVM instruction and interop hash is available to the translator. The current layout is:

```
wasm-neovm/
├─ src/
│  ├─ lib.rs
│  ├─ manifest.rs        // Manifest rendering helpers
│  ├─ metadata.rs        // Method-token extraction utilities
│  ├─ nef.rs             // NEF writer utilities
│  ├─ neo_syscalls.rs    // Friendly `neo::` → syscall descriptor mapping
│  ├─ opcodes.rs         // Generated opcode metadata
│  ├─ syscalls.rs        // Generated syscall metadata
│  └─ translator/
│      └─ mod.rs         // Wasm parsing and NeoVM lowering
└─ tests/
   ├─ basic.rs
   ├─ memory_tests.rs
   ├─ table_tests.rs
   ├─ syscall_tests.rs
   └─ … (additional focused suites)
```

### 3.1 Parsing & validation

1. Walk Wasm sections with `ModuleReader`.
2. Record the type signatures, function bodies, exports, and any custom sections.
3. Enforce constraints that keep the first version tractable: a single linear memory, integer-only arithmetic (floating point and SIMD still pending), and reference types restricted to `funcref` handles (which back `call_indirect`).
4. Require exported functions to be `#[no_mangle] pub extern "C" fn ...` to avoid name mangling.

### 3.2 Instruction lowering

Maintain a small interpreter over the Wasm operand stack. For every operator, emit one or more NeoVM opcode bytes while tracking literal constants so redundant pushes can be removed later on:

| Wasm operator                         | NeoVM emission                                                                           |
|---------------------------------------|-------------------------------------------------------------------------------------------|
| `i32.const n`, `i64.const n`          | `emit_push(n)` using the smallest `PUSHINT*` opcode that can materialise the literal      |
| `local.get x`                         | `emit_ld(slot(x))` (`LDARG*` for params, `LDLOC*` for locals) or reuse known literal pushes |
| `local.set x`                         | `emit_st(slot(x))`, updating the cached literal value if the operand is constant          |
| `local.tee x`                         | Store the value and immediately reload it (`ST*` + `LD*`), preserving literal knowledge   |
| `i32`/`i64` `add` / `sub` / `mul`     | `ADD` / `SUB` / `MUL`                                                                     |
| `i32`/`i64` `and` / `or` / `xor`      | `AND` / `OR` / `XOR`                                                                      |
| `i32`/`i64` `shl` / `shr_s` / `shr_u` | `SHL` / `SHR` with shift counts masked to the width and operand masking for logical shifts |
| `i32`/`i64` `rotl` / `rotr`           | Literal pairs fold to immediates; dynamic operands emit `PICK`/`SWAP` sequences to reuse values |
| `block` / `loop` / `if`               | Emits NeoVM `JMP*_L` placeholders patched at `end`; loops jump to the recorded header |
| `br` / `br_if`                        | Unconditional/conditional branches translate to `JMP_L` / `JMPIF_L`; current support assumes single-value or void blocks |
| `i32`/`i64` `div_s` / `div_u` / `rem_s` / `rem_u` | `DIV` / `MOD`, masking both operands for unsigned variants to match Wasm semantics |
| `i32`/`i64` comparisons (`eq`, `ne`, `eqz`, signed/unsigned `lt`/`le`/`gt`/`ge`) | `EQUAL` / `NOTEQUAL` / `LT` / `LE` / `GT` / `GE`; `eqz` compares against `PUSH0` and unsigned variants mask operands before comparison |
| `select` / typed `select` (single `i32`/`i64` result) | `JMPIFNOT_L` + `DROP` for the true arm with `JMP_L`/`NIP` to discard the non-selected value |
| `br_table`                           | Duplicates the selector to emit `JMPIF_L` comparisons for each case, drops the selector before branching to the chosen depth, and falls back to the default target |
| Integer conversions (`i32.wrap_i64`, `i64.extend_i32_{s,u}`, `i32.extend{8,16}_s`, `i64.extend{8,16,32}_s`) | Masks operands with `AND` and applies `SHL`/`SHR` pairs when required to reproduce Wasm sign-extension semantics |
| `i32`/`i64` `clz` / `ctz` / `popcnt` | Compile-time literals fold to immediates; dynamic operands call compact helpers (constructed from `INITSLOT`, `SETITEM`, and arithmetic opcodes) that exactly reproduce Wasm bit-counting semantics |
| `global.get` / `global.set`          | Module globals are stored in static slots initialised from constant expressions; immutable globals fold to literals, mutable globals use `STSFLD*`/`LDSFLD*` |
| `memory.size` / `memory.grow` / `memory.load*` / `memory.store*` / `memory.fill` / `memory.copy` / `memory.init` / `data.drop` | Linear memory lives in a static buffer slot; helpers handle bounds checks, (re)allocation via `NEWBUFFER` + `MEMCPY`, byte slicing with `SUBSTR`, staged writes through `SETITEM`, bulk operations (`MEMCPY` for copy/init) with memmove-style overlap handling, and lazily-applied drop flags so repeated `memory.init`/`data.drop` calls follow Wasm semantics |
| `call_indirect`                      | Funcref tables dispatch via runtime lookups that honour dynamic table mutations and trap when entries are null or out of range |
| `table.get` / `table.set` / `table.size` / `table.grow` / `table.fill` / `table.copy` / `table.init` / `elem.drop` | Tables live in static slots backed by NeoVM arrays; helpers validate indices, enforce declared maxima, copy passive segments, and mark dropped elements |
| `drop`                                | Elide the literal push when possible, otherwise emit `DROP`                               |
| `unreachable`                         | Emit `ABORT` to terminate execution                                                       |
| `return`                              | Ensure the stack holds the result (if any) and append `RET`                               |

Structured control flow (`block`, `loop`, `if`, `br`, `br_if`, `br_table`) already uses a label/fix-up mechanism that patches `JMP*_L` immediates once their targets are known, maintaining Wasm stack-height invariants for void and single-value blocks. Unsigned operators are implemented by masking operands to the appropriate bit-width before invoking `DIV`, `MOD`, or `SHR`, ensuring that NeoVM semantics line up with Wasm's zero-extension rules. Memory instructions dispatch to runtime helpers that guard against out-of-bounds access and synchronise the backing buffer with the current page count.

Block and `if`/`loop` result types are limited to a single integer; the translator validates stack layouts at branch targets and block ends so mismatched arities surface as translation errors rather than silent miscompilation.

`if` expressions with a result must include an `else` arm. `loop` labels expect the stack height at loop entry when branching back to the header (continue), while exiting a loop honours the declared result arity.

Passive data segments are materialised into static slots alongside their drop flags; `memory.init` copies slices into the linear-memory buffer via helper-managed `MEMCPY` calls, while `data.drop` marks a segment as unusable for subsequent inits. Active segments are applied during the first memory initialisation, ensuring that traditional `(data (i32.const ...))` declarations populate the backing buffer before user code executes. Global initialisers run alongside the memory bootstrap so `global.get`/`global.set` operate on pre-populated slots.

### 3.3 Locals and stack slots

- Reserve consecutive NeoVM local slots for Wasm locals beyond the parameter list (supporting `i32` and `i64`).
- Map parameter indices `0..N` directly to `LDARGn`/`STARGn` when `n ≤ 6`; otherwise fall back to generic `LDARG`/`STARG` with immediates.
- Track the last known literal for each slot so `local.get` can reuse earlier pushes without emitting new bytecode.
- Provide helpers `emit_ld(slot: u32)` and `emit_st(slot: u32)` that choose the optimal opcode encoding.

### 3.4 Syscall and opcode bridge

Many contracts invoke runtime services through `extern` Wasm imports (e.g., `env::storage_read`). Build-time code scans the upstream `neo` repository and generates tables so every syscall descriptor and opcode name is available to the translator. During translation:

- Imports from the `syscall` module become `SYSCALL` opcodes carrying the pre-hashed identifier bytes.
- Imports from the `neo` module reuse the friendlier DevPack names (`neo::storage_get`, `neo::notify`, …), which are resolved through the generated lookup table before lowering to the same `SYSCALL` sequence. This keeps existing Rust contracts working while adopting the new pipeline.
- Imports from the `opcode` module map directly to NeoVM opcodes, with literal parameters folded into the immediate operand. Helper imports `opcode::RAW` and `opcode::RAW4` append arbitrary bytes so variable-length sequences (or unrecognised instructions) can still be emitted.
- Unsupported import modules surface as explicit diagnostics so missing bridges are easy to spot.
- Imports from the `env` module are confined to memory shims (`memcpy`, `memmove`, `memset`, and builtin spellings). Each shim expects three `i32` parameters `(dest, src/value, len)` and, when the Wasm signature includes a single `i32` result, returns the destination pointer. Calls are lowered to bounded runtime helpers that honour the NeoVM linear-memory model.

### 3.5 Error handling

All translation failures return a structured error variant. Include the offending function index, instruction offset, and a short explanation to aid debugging.

## 4. NEF + manifest assembly

The translator owns a small NEF writer (`nef.rs`) that:

1. Streams the generated script bytes (and optional metadata) into the NEF container via `write_nef_with_metadata`.
2. Collects ABI information (export names, parameter types, return types) while translating and renders it with `manifest::build_manifest`. Manifest overlays may not mutate translated signatures/offsets; mismatches abort translation.
3. Scans the emitted script for literal `SYSCALL` patterns. `System.Contract.Call` invocations still produce contract/method tokens when the hash/method/argument array are constant, and every other syscall contributes a zero-hash `MethodToken` so the NEF metadata lists each interop touched by the contract. Method-token names are capped at 32 bytes; oversized entries are rejected/ignored.
4. Writes `<contract>.nef` and `<contract>.manifest.json` next to the Wasm artefact, ready for packaging.

Method-token names are capped at 32 bytes to match the NEF format; longer syscall names are ignored during inference, and overlays/embedded sections are validated at parse time.

### Opcode & syscall imports

The translator recognises two reserved import modules to expose the complete NeoVM surface:

- `syscall` – import functions named after interop descriptors (for example `System.Runtime.GetTime`). Calls become `SYSCALL` opcodes with the correct hashed identifier.
- `opcode` – import functions named after NeoVM opcodes (case-insensitive) for instructions that do not have a natural Wasm equivalent. Literal parameters (`i32.const`, etc.) are folded into fixed-size immediates. For more elaborate payloads you can use the helper imports `opcode::RAW` (append a single byte) and `opcode::RAW4` (append four bytes) to emit arbitrary script data.

These facilities complement automatic lowering of core Wasm instructions (`i32.const`, `i32.add`, etc.) and make it possible to exercise every opcode, syscall, and native contract entry point from Rust.

## 5. Testing strategy

- **Unit tests** over the translator covering each supported operator mapping.
- **Integration tests** that compile miniature Rust crates to Wasm, run the translator, and assert the emitted opcodes/manifest match expectations.
- **Golden tests** comparing generated NEF scripts against known-good outputs for canonical contracts (e.g., storage key-value, NEP-17 skeleton).
- **Validation** by running the final NEF through `neo-cli` or the NeoVM reference interpreter to ensure acceptance.

## 6. Roadmap for instruction coverage

1. Arithmetic & locals (in progress → achieved for `i32`/`i64` adds, subs, muls, comparisons, literal tracking).
2. Control flow – translate `block/loop/if`, `br`, `br_if`, and user function calls.
3. Memory & buffers – current helpers cover `memory.size`, `memory.grow`, the `load*`/`store*` family, and bulk operations (`memory.fill`/`memory.copy`) with overlap-safe copies; remaining work tracks data segments and multi-memory modules. Init failures propagate rather than being swallowed.
4. Heap abstractions – translate `Vec`, `String` usage to Neo array operations.
5. Events & manifests – `neo_event` macros emit canonical `abi.events` entries (Boolean/Integer/ByteArray/etc.) via custom sections; the translator merges and deduplicates them alongside any manual overlays. Storage usage is inferred directly from `System.Storage.*` syscalls so `features.storage` flips on automatically, and exporting payment handlers (`onPayment`, `onNEP17Payment`, `onNEP11Payment`) toggles `features.payable`.
6. Debug metadata – attach sequence points, contract hash hints, gas metering.

## 7. Developer workflow summary

1. `cargo build -p <contract>` targeting `wasm32-unknown-unknown --release` (the repo ships `scripts/build_contract.sh <path> [name] [translator args...]` to wrap this + translation in one step; `scripts/build_c_contract.sh` handles the clang-based C sample).
2. `wasm-neovm translate --input <name>.wasm --nef build/<name>.nef --manifest build/<name>.manifest.json [--compare-manifest path/to/reference.json]` (automatically invoked by the helper scripts).
3. Optionally run `neo-cli`, Neo Express, or emulator tests (`integration-tests/` contains a ready-made Neo Express harness).
4. Package artifacts for deployment.

Automate steps 1–3 behind a single command (`cargo neovm build`) or rely on the provided shell scripts during local development. The optional `--compare-manifest` flag fails the translation if the generated manifest diverges from a known-good JSON file, providing a quick validation pass in CI.

## 8. Open questions

- **Gas accounting**: derive gas costs by summing opcode tables or replaying in the NeoVM simulator.
- **Type system**: decide on encoding for complex types (`struct`, `enum`) crossing the ABI boundary.
- **Panic semantics**: map Rust panics (which become `unreachable` in Wasm) to `ABORT` or `ASSERTMSG`.
- **Host interop**: specify how storage and contract management APIs surface to Rust (likely via `extern "C"` functions provided by the devpack runtime).

## 9. Unsupported Wasm Features

The NeoVM runtime enforces a single linear memory and does not expose a general-purpose heap or garbage collector. As a result:

- **Multiple memories** – Wasm modules may declare at most one memory. The translator rejects additional memories with an explicit error, because NeoVM helpers only manage the single linear buffer allocated during startup.
- **Reference types beyond `funcref`** – NeoVM represents function references as integer sentinels, but it has no concept of `externref`, `anyref`, or GC-managed objects. These reference types remain unsupported; any module importing or producing them will receive a descriptive translation error.

These limitations keep the generated bytecode aligned with current NeoVM capabilities. Future NeoVM enhancements that add multi-memory support or host-managed reference handles would require revisiting this design.

## 10. Current Compatibility Matrix

| Area | Status / Notes |
|------|----------------|
| Linear memory | Exactly one 32-bit, non-shared `memory` section is allowed. Data/bulk ops are supported, but modules without a defined memory cannot use passive segments. |
| Tables & references | Tables must be `funcref`, `table64` and shared tables are rejected. `call_indirect`, `table.get/set/copy/fill/grow`, and passive element initialisers are translated via runtime helpers that trap on null dispatches. |
| Imports | Only function imports are accepted. Modules such as `env`, `syscall`, `neo`, and `opcode` have dedicated lowering paths; global or memory imports are currently unsupported. |
| Globals | Global variables must be `i32`/`i64`. Initialisers are validated at translation time and stored in static slots. |
| Function signatures | Exported ABI methods as well as `call_indirect` signatures may only use `i32`/`i64` parameters and at most a single return value. Typed `select` is also restricted to single `i32`/`i64` results. |
| Numeric features | Integer arithmetic, comparisons, shifts, bit counts, and conversions are covered. Floating-point instructions and SIMD remain on the roadmap and will currently raise descriptive translation errors. |

---

This design keeps the immediate scope manageable, delivers a functioning pipeline for simple contracts, and leaves space for incremental additions until the translator reaches feature parity with the NeoVM runtime.

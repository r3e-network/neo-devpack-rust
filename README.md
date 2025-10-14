# Neo Wasm → NeoVM Pipeline

This repository hosts the Rust tooling required to compile Neo N3 smart contracts to WebAssembly and convert the resulting modules into NeoVM NEF artefacts. The legacy in-tree LLVM NeoVM backend has been retired in favour of a simpler, Wasm-first workflow.

The workflow is:

```
Rust contract (neo-devpack) ──cargo build --target wasm32-unknown-unknown──▶ Wasm module ──wasm-neovm──▶ NEF + manifest
```

## What's Included

- **`wasm-neovm`** – a Rust CLI/library that translates a Wasm module into NeoVM bytecode and emits the accompanying NEF+manifest pair.
- **`rust-devpack`** – the existing Rust developer tooling (types, macros, runtime stubs) for authoring Neo contracts.
- **`scripts/build_contract.sh`** – helper script that builds a contract to Wasm and invokes the translator in a single step.
- **Documentation** – updated notes on the new pipeline in [`docs/wasm-pipeline.md`](docs/wasm-pipeline.md) and the NEF container format in [`docs/nef-format-specification.md`](docs/nef-format-specification.md).

## Quick Start

1. Install Rust and the Wasm target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
2. Build your contract crate (for example `contracts/hello`):
   ```bash
   scripts/build_contract.sh contracts/hello Hello
   ```
   The script compiles the crate for `wasm32-unknown-unknown` (release mode) and then runs the translator to produce `Hello.nef` and `Hello.manifest.json` next to the `.wasm` artefact.
   Append additional translator flags after the optional contract name, for example `--safe-method main` to mark the exported entry point as safe.

3. Alternatively, run the translator manually:
   ```bash
   cargo build --manifest-path contracts/hello/Cargo.toml \
     --release --target wasm32-unknown-unknown

   cargo run --manifest-path wasm-neovm/Cargo.toml -- \
     --input contracts/hello/target/wasm32-unknown-unknown/release/Hello.wasm \
     --nef build/Hello.nef \
     --manifest build/Hello.manifest.json \
     --name Hello \
     --safe-method main \
     --manifest-overlay contracts/hello/manifest-extra.json
   ```

   Use one or more `--safe-method <name>` flags to mark exported methods as safe in the generated manifest. Supply `--manifest-overlay <file>` to merge additional JSON metadata when needed.

Rust contracts can now embed manifest metadata directly via DevPack macros:

```rust
use neo_devpack::prelude::*;

#[neo_event]
pub struct TransferEvent {
    pub from: NeoByteString,
    pub to: NeoByteString,
    pub amount: NeoInteger,
}

neo_permission!("0xff", ["balanceOf"]);
neo_supported_standards!(["NEP-17"]);
neo_trusts!(["*"]);
```

Each `#[neo_event]` declaration automatically registers the event schema, and `neo_permission!` records required permissions. The translator merges these custom sections with any additional overlay file or CLI flags, so manifests stay in sync without manual JSON edits.

### Emitting Opcodes and Syscalls

The translator understands a small set of reserved Wasm import modules:

- `syscall`: import functions named after Neo interop descriptors (for example, `System.Runtime.GetTime`). During translation each call becomes a NeoVM `SYSCALL` with the appropriate 4-byte hash.
- `neo`: use the DevPack-friendly aliases (`storage_get`, `notify`, `call_contract`, …). The translator resolves these names to their canonical descriptors before emitting the `SYSCALL` instruction, so existing Rust contracts keep compiling unchanged.
- `opcode`: import functions whose names match NeoVM opcodes (for example, `SWAP`). Calls to these opcodes emit the corresponding bytecode. For immediates you can either supply literal parameters (e.g., `PUSHINT32` takes one `i32.const` argument) or fall back to the utility imports `RAW` (append a single byte) and `RAW4` (append four bytes) to inject arbitrary data.

Example (in WAT form):

```wat
(module
  (import "syscall" "System.Runtime.GetTime" (func $get_time (result i64)))
  (import "opcode" "DEPTH" (func $depth))
  (func (export "entry") (result i64)
    call $depth
    call $get_time))
```

The accompanying Rust contract can declare the imports with `#[link(wasm_import_module = "syscall")]` / `#[link(wasm_import_module = "neo")]` / `#[link(wasm_import_module = "opcode")]` attributes. To emit raw bytes, bind to `opcode::RAW`/`opcode::RAW4` and pass literal constants.

## Translator Capabilities

`wasm-neovm` already lowers a useful subset of Wasm into NeoVM bytecode:

- Immediate folding for `i32.const` and `i64.const`, choosing the smallest available `PUSHINT*` opcode and propagating literal values through locals.
- Integer arithmetic and comparisons – `add`, `sub`, `mul`, `eq`, `ne`, `eqz`, `lt`, `le`, `gt`, and `ge` – shared between 32-bit and 64-bit Wasm operands.
- Bitwise logic, shifts, and rotations – `and`, `or`, `xor`, `shl`, `shr_s`/`shr_u`, and `rotl`/`rotr` (with shift masking and dynamic support).
- Bit counting – `clz`, `ctz`, and `popcnt` fold literals to immediates and fall back to small NeoVM helpers for dynamic operands.
- Globals – `global.get`/`global.set` for `i32`/`i64` globals, initialised from constant expressions and stored in module-scoped static slots.
- Indirect calls – `call_indirect` over funcref tables populated via `elem` segments, lowering to bounds-checked dispatch that traps on uninitialised entries.
- Reference types – `ref.null`, `ref.func`, `ref.is_null`, `ref.eq`, and `ref.as_non_null`, with funcref values represented as sentinel-aware integers.
- Table operations – full support for `table.get/set/size/grow/fill/copy` across multiple tables, passive segment initialisation via `table.init`, inline table initialisers, and `elem.drop`, all routed through shared runtime helpers with bounds checks.
- Structured control flow – `block`, `loop`, `if`/`else`, `br`, `br_if`, and `br_table`, using patched `JMP*_L` sequences while maintaining Wasm stack height guarantees (single-value or void blocks today).
- Conditional selection – `select` (and typed select with a single `i32`/`i64` result) lowered via `JMPIFNOT_L`, `DROP`, and `NIP` patterns.
- Integer conversions – `i32.wrap_i64`, `i64.extend_i32_{s,u}`, and sign-extension helpers (`i32.extend{8,16}_s`, `i64.extend{8,16,32}_s`).
- Signed and unsigned division/remainder (`div_s`, `div_u`, `rem_s`, `rem_u`) lowered to `DIV`/`MOD`, masking operands to preserve Wasm semantics.
- Full support for `local.get`, `local.set`, and `local.tee`, mapping function arguments to `LDARG*` and stack locals to `LDLOC*`/`STLOC*` opcodes.
- Linear memory – single-memory modules can use the full `load*`/`store*` family, `memory.size`, `memory.grow`, bulk operations (`memory.fill`, `memory.copy`), and data-segment primitives (`memory.init`, `data.drop`). Passive segment bytes are cached in static slots, active segments are copied in during the first initialisation, and helpers enforce bounds checks before every access.
- Exported signatures may use `i32` or `i64` parameters; literal tracking carries through both kinds of locals.
- `drop` elimination (removing dead literals) and `unreachable` lowering to the NeoVM `ABORT` opcode.
- Import bridges for every NeoVM opcode (`opcode::<NAME>`) and syscall (`syscall::<Descriptor>`), including helpers for emitting raw immediates (`opcode::RAW`/`RAW4`).

-Unsupported instructions (floating-point, reference types beyond funcref, and multi-memory) surface descriptive errors. [`docs/wasm-pipeline.md`](docs/wasm-pipeline.md) tracks the roadmap toward full coverage.

## Development

- Run translator tests:
  ```bash
  cargo test --manifest-path wasm-neovm/Cargo.toml
  ```
- Work on the devpack:
  ```bash
  cargo test --manifest-path rust-devpack/Cargo.toml
  ```
- Format & lint (uses stable Rust tooling):
  ```bash
  cargo fmt --all
  cargo clippy --all-targets --all-features
  ```

## Directory Layout

```
.
├── docs/                 # Updated documentation for the Wasm pipeline
├── rust-devpack/         # Rust SDK for Neo contracts
├── scripts/              # Helper scripts (build + translate)
└── wasm-neovm/           # Wasm → NeoVM translator crate
```

## Next Steps

- Extend table support (table grow/set operations, reference types) and richer element segment initialisers.
- Add floating-point/SIMD instruction lowering once integer semantics are fully settled.
- Enrich manifest generation with devpack metadata (events, permissions, standards) and exercise NEF output in the NeoVM reference runner.

Contributions towards these milestones are welcome.

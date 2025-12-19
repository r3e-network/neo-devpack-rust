# Neo Wasm → NeoVM Pipeline

[![CI](https://github.com/r3e-network/neo-llvm/workflows/CI/badge.svg)](https://github.com/r3e-network/neo-llvm/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/r3e-network/neo-llvm/branch/master/graph/badge.svg)](https://codecov.io/gh/r3e-network/neo-llvm)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

This repository hosts the Rust tooling required to compile Neo N3 smart contracts to WebAssembly and convert the resulting modules into NeoVM NEF artefacts. The legacy in-tree LLVM NeoVM backend has been retired in favour of a simpler, Wasm-first workflow.

The workflow is:

```
Rust contract (neo-devpack) ──cargo build --target wasm32-unknown-unknown──▶ Wasm module ──wasm-neovm──▶ NEF + manifest
```

## Cross-Chain Compilation (Solana ready; Move experimental)

The toolchain supports cross-chain smart contract compilation from Solana and Move ecosystems:

### Solana Programs

Compile Solana-compatible Rust contracts using the `neo-solana-compat` compatibility layer:

```bash
# Step 1: Replace solana_program with neo-solana-compat in Cargo.toml
[dependencies]
neo-solana-compat = { path = "../solana-compat" }

# Step 2: Build to WASM
cargo build --manifest-path contracts/solana-hello/Cargo.toml \
  --target wasm32-unknown-unknown --release

# Step 3: Translate to NEF
cargo run --manifest-path wasm-neovm/Cargo.toml -- \
  --input contracts/solana-hello/target/wasm32-unknown-unknown/release/solana_hello.wasm \
  --nef build/solana_hello.nef \
  --manifest build/solana_hello.manifest.json \
  --name solana-hello
```

**Solana API Compatibility:**

| Solana API     | Neo-Solana-Compat | Notes                            |
| -------------- | ----------------- | -------------------------------- |
| `Pubkey`       | ✅ Supported      | 32-byte key, converts to UInt160 |
| `AccountInfo`  | ✅ Supported      | Maps to storage operations       |
| `ProgramError` | ✅ Supported      | Full enum support                |
| `entrypoint!`  | ✅ Supported      | WASM export generation           |
| `invoke()`     | ✅ Supported      | Maps to System.Contract.Call     |

**Syscall Mapping:**

- `sol_log` → `System.Runtime.Log`
- `sol_sha256` → `Neo.Crypto.SHA256`
- `sol_get_clock_sysvar` → `System.Runtime.GetTime`
- `sol_invoke` → `System.Contract.Call`
- Account read/write → `System.Storage.Get/Put`

### Move Contracts (Experimental)

> Note: Move bytecode → WASM lowering is now implemented with control flow,
> storage-backed resource operations, and ability checks. The coverage is still
> experimental and does not yet model every Move feature.

Compile Move-style contracts (Aptos/Sui patterns) to NeoVM:

```bash
# Step 1: Build Move-style Rust contract
cargo build --manifest-path contracts/move-coin/Cargo.toml \
  --target wasm32-unknown-unknown --release

# Step 2: Translate to NEF
cargo run --manifest-path wasm-neovm/Cargo.toml -- \
  --input contracts/move-coin/target/wasm32-unknown-unknown/release/move_coin.wasm \
  --nef build/MoveCoin.nef \
  --manifest build/MoveCoin.manifest.json \
  --name MoveCoin \
  --manifest-overlay contracts/move-coin/manifest.overlay.json
```

You can also supply raw Move bytecode (`.mv`) to `wasm-neovm` with `--source-chain move`; it will first run `move-neovm` to produce WASM and then complete the NeoVM translation.

**Move Resource Semantics:**

Move's linear resource types are mapped to Neo storage:

- `move_to<T>()` → `System.Storage.Put`
- `move_from<T>()` → `System.Storage.Delete`
- `borrow_global<T>()` → `System.Storage.Get`
- `exists<T>()` → Storage existence check

See `contracts/move-coin/src/lib.rs` for a minimal Move-inspired coin example that uses
storage-backed balances and witness checks. The Move path is still experimental: not every
bytecode feature is modelled yet, but control flow and basic resource semantics are translated
end-to-end.

### Running Cross-Chain Tests

```bash
# Solana compatibility tests (26 tests)
cargo test --manifest-path solana-compat/Cargo.toml

# Move translator tests (experimental)
cargo test --manifest-path move-neovm/Cargo.toml

# Cross-chain integration tests (9 tests)
cargo test --manifest-path wasm-neovm/Cargo.toml cross_chain

# Build all cross-chain examples
make examples
```

See [`docs/CROSS_CHAIN_SPEC.md`](docs/CROSS_CHAIN_SPEC.md) for the full technical specification.

## What's Included

- **`wasm-neovm`** – a Rust CLI/library that translates a Wasm module into NeoVM bytecode and emits the accompanying NEF+manifest pair.
- **`rust-devpack`** – the existing Rust developer tooling (types, macros, runtime stubs) for authoring Neo contracts.
- **`contracts/`** – assemble-ready Rust smart-contracts (`hello-world`, `nep17-token`, `constant-product`) showcasing different patterns.
- See [`contracts/README.md`](contracts/README.md) for per-contract entry points and build notes.
- **`scripts/build_contract.sh`** – helper script that builds a Rust contract to Wasm and invokes the translator in a single step.
- **`scripts/build_c_contract.sh`** – clang-based helper that compiles plain C contracts to Wasm before translating them.
- **`integration-tests/`** – optional Neo Express harness (see [`docs/neoexpress-integration.md`](docs/neoexpress-integration.md)) for exercising generated NEF artefacts.
- **Documentation** – updated notes on the new pipeline in [`docs/wasm-pipeline.md`](docs/wasm-pipeline.md) and the NEF container format in [`docs/nef-format-specification.md`](docs/nef-format-specification.md). See [`spec/wasm-neovm-spec.tex`](spec/wasm-neovm-spec.tex) for the full normative translation spec (buildable via `make -C spec`).
- **Rust contract quickstart** – step-by-step instructions for authoring and compiling contracts live in [`docs/rust-smart-contract-quickstart.md`](docs/rust-smart-contract-quickstart.md).

## Quick Start

1. Install Rust and the Wasm target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
2. Build your contract (for example `contracts/hello-world` or the C sample in `contracts/c-hello`):

   ```bash
   scripts/build_contract.sh contracts/hello-world
   # or
   scripts/build_c_contract.sh contracts/c-hello
   ```

   The Rust helper compiles the crate for `wasm32-unknown-unknown` (release mode) and then runs the translator to produce `hello_world.nef` and `hello_world.manifest.json` next to the `.wasm` artefact.  
   The C helper wraps `clang --target wasm32-unknown-unknown` (with `-nostdlib`/`-fno-builtin` to avoid `env::` imports) and writes the Wasm/NEF/manifest trio into `contracts/c-hello/build/`.
   Append additional translator flags after the optional contract name if needed. Safe methods are typically declared inside the contract (via `#[neo_safe]`) so no CLI flags are required for that metadata.

3. Alternatively, run the translator manually:

   ```bash
   cargo build --manifest-path contracts/hello-world/Cargo.toml \
     --release --target wasm32-unknown-unknown

   cargo run --manifest-path wasm-neovm/Cargo.toml -- \
     --input contracts/hello-world/target/wasm32-unknown-unknown/release/hello_world.wasm \
     --nef build/hello_world.nef \
     --manifest build/hello_world.manifest.json \
     --name hello-world \
     --manifest-overlay contracts/hello-world/manifest.overlay.json \
     --compare-manifest contracts/hello-world/expected.manifest.json
   ```

Supply `--manifest-overlay <file>` to merge additional JSON metadata when needed (for example, create `contracts/hello-world/manifest.overlay.json`). Overlay entries must reference exports that actually exist in the Wasm module—the translator now errors if an overlay adds or removes ABI methods. Use the `#[neo_safe]` attribute (or manifest overlays) inside your contract to declare safe methods.

Use `--compare-manifest <file>` to assert that the generated manifest matches a checked-in reference; any difference aborts the translation after printing a unified diff.

4. To compile _all_ bundled examples (Wasm build + NEF/manifest generation) run the Makefile target:

   ```bash
   make examples
   ```

   Outputs are written to `build/`. Use `make clean` to remove generated artefacts.

5. For the cross-chain samples (Solana + experimental Move), run:

   ```bash
   make cross-chain
   ```

   This produces `build/solana_hello.nef/.manifest.json` and `build/MoveCoin.nef/.manifest.json` using the appropriate `--source-chain` flag. Move support remains experimental, but control flow and storage-backed resource operations (with ability checks) are now lowered end-to-end.

   You can also drive individual builds via `scripts/build_contract.sh <path> [name] [translator-flags...]`.
   To pin a source chain without repeating flags, set `SOURCE_CHAIN=solana` (or `move`) in the environment;
   the script will pass `--source-chain` automatically when not already provided.

6. To run just the cross-chain translator tests:

   ```bash
   make test-cross-chain
   ```

7. Individual contracts can be built with their dedicated targets, for example:

   ```bash
   make nep11-nft
   ```

8. To deploy a generated contract to a running Neo Express instance you can use the
   helper script:
   ```bash
   export NEO_EXPRESS_RPC=http://localhost:50012
   scripts/neoexpress_deploy.sh build/HelloWorld.nef build/HelloWorld.manifest.json HelloWorld
   ```

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

Each `#[neo_event]` declaration automatically registers the event schema using canonical manifest parameter types (Boolean, Integer, ByteArray, …), and the helper macros (`neo_permission!`, `neo_trusts!`, `neo_supported_standards!`) record additional metadata. The translator merges these custom sections with any additional overlay file or CLI flags, so manifests stay in sync without manual JSON edits. Storage-heavy contracts no longer need to opt into the `storage` feature manually—the translator watches for `System.Storage.*` syscalls and flips `features.storage` on their behalf. Likewise, exporting `onPayment`/`onNEP17Payment`/`onNEP11Payment` automatically enables `features.payable`.

### Supported Wasm Features & Limits

The translator currently supports integer-only Wasm modules with a single linear memory and funcref tables. Arithmetic, control flow, locals/globals, data segments, tables, bulk memory instructions, and `call_indirect` lowering are available today. Floating-point/SIMD operators, multi-memory modules, and reference types beyond `funcref` will produce descriptive translation errors. See [`docs/wasm-pipeline.md`](docs/wasm-pipeline.md#10-current-compatibility-matrix) for the up-to-date compatibility matrix covering imports, globals, signatures, and runtime helpers.

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

### Feature Checklist

**Implemented**

- [x] Wasm → NeoVM translation pipeline (`wasm-neovm`) with manifest/NEF generation
- [x] Safe method detection via in-contract attributes/overlays (no CLI flags required)
- [x] Manifest overlay merge + permission deduplication
- [x] Method-token inference for `System.Contract.Call` and syscall usage
- [x] Comprehensive unit tests for translator modules
- [x] Production-grade Rust contract examples (10 templates covering NEP‑17, NEP‑11, multisig, escrow, DAO, oracle, NFT marketplace, etc.)
- [x] Makefile automation (`make examples`) to build and translate every contract
- [x] Documentation for quick start, contract catalogue, and Neo Express deployment
- [x] **Solana compatibility layer** (`neo-solana-compat`) with full API support (Pubkey, AccountInfo, ProgramError, invoke)
- [x] **Move-style contract support** with resource semantics emulation
- [x] **Cross-chain integration tests** (9 tests covering Solana/Move compilation)
- [x] **Cross-chain specification document** (`docs/CROSS_CHAIN_SPEC.md`)

**Planned / In Progress**

- [ ] Extend Wasm coverage (floating-point operations, SIMD, multi-memory)
- [ ] Enhanced Neo Express integration tests with automated deploy/invoke smoke checks
- [ ] Additional dApp templates (router/aggregator, oracle publisher, governance extensions)
- [ ] CLI UX improvements (manifest diffing, dry-run validation)
- [ ] Linting rules for contract ABI compliance and storage layout verification

`wasm-neovm` already lowers a useful subset of Wasm into NeoVM bytecode:

- Immediate folding for `i32.const` and `i64.const`, choosing the smallest available `PUSHINT*` opcode and propagating literal values through locals.
- Integer arithmetic and comparisons – `add`, `sub`, `mul`, `eq`, `ne`, `eqz`, `lt`, `le`, `gt`, and `ge` – shared between 32-bit and 64-bit Wasm operands.
- Bitwise logic, shifts, and rotations – `and`, `or`, `xor`, `shl`, `shr_s`/`shr_u`, and `rotl`/`rotr` (with shift masking and dynamic support).
- Bit counting – `clz`, `ctz`, and `popcnt` fold literals to immediates and fall back to small NeoVM helpers for dynamic operands.
- Globals – `global.get`/`global.set` for `i32`/`i64` globals, initialised from constant expressions and stored in module-scoped static slots.
- Indirect calls – `call_indirect` over funcref tables populated via `elem` segments, lowering to bounds-checked dispatch that traps on uninitialised entries.
- Reference types – `ref.null`, `ref.func`, `ref.is_null`, `ref.eq`, and `ref.as_non_null`, with funcref values represented as sentinel-aware integers.
- Table operations – full support for `table.get/set/size/grow/fill/copy` across any declared `funcref` tables (used internally for `call_indirect`), passive element initialisation via `table.init`, inline table initialisers, and `elem.drop`, all routed through shared runtime helpers with bounds checks. ABI signatures still need to stay in the supported `i32`/`i64` space, so reference types cannot cross the module boundary.
- Structured control flow – `block`, `loop`, `if`/`else`, `br`, `br_if`, and `br_table`, using patched `JMP*_L` sequences while maintaining Wasm stack height guarantees (single-value or void blocks today).
- Conditional selection – `select` (and typed select with a single `i32`/`i64` result) lowered via `JMPIFNOT_L`, `DROP`, and `NIP` patterns.
- Integer conversions – `i32.wrap_i64`, `i64.extend_i32_{s,u}`, and sign-extension helpers (`i32.extend{8,16}_s`, `i64.extend{8,16,32}_s`).
- Signed and unsigned division/remainder (`div_s`, `div_u`, `rem_s`, `rem_u`) lowered to `DIV`/`MOD`, masking operands to preserve Wasm semantics.
- Full support for `local.get`, `local.set`, and `local.tee`, mapping function arguments to `LDARG*` and stack locals to `LDLOC*`/`STLOC*` opcodes.
- Linear memory – single-memory modules can use the full `load*`/`store*` family, `memory.size`, `memory.grow`, bulk operations (`memory.fill`, `memory.copy`), and data-segment primitives (`memory.init`, `data.drop`). Passive segment bytes are cached in static slots, active segments are copied in during the first initialisation, and helpers enforce bounds checks before every access.
- Exported signatures may use `i32` or `i64` parameters; literal tracking carries through both kinds of locals.
- `drop` elimination (removing dead literals) and `unreachable` lowering to the NeoVM `ABORT` opcode.
- Import bridges for every NeoVM opcode (`opcode::<NAME>`) and syscall (`syscall::<Descriptor>`), including helpers for emitting raw immediates (`opcode::RAW`/`RAW4`).

Unsupported instructions (floating-point, reference types beyond funcref, and multi-memory) surface descriptive errors. [`docs/wasm-pipeline.md`](docs/wasm-pipeline.md) tracks the roadmap toward full coverage.

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
- Rebuild the formal spec PDF (optional, requires LaTeX tooling):
  ```bash
  make -C spec
  ```

## Directory Layout

```
.
├── .github/
│   └── workflows/        # CI/CD configuration (tests, clippy, security audit)
├── docs/                 # Technical documentation
│   ├── CROSS_CHAIN_SPEC.md           # Cross-chain compilation specification
│   ├── wasm-pipeline.md              # WebAssembly translation pipeline
│   ├── nef-format-specification.md   # NEF container format
│   ├── rust-smart-contract-quickstart.md  # Contract development guide
│   └── neoexpress-integration.md     # Neo Express deployment guide
├── wasm-neovm/           # Core WebAssembly to NeoVM translator
│   ├── src/              # Translation engine, NEF builder, manifest generator
│   ├── tests/            # Integration tests including cross-chain tests
│   └── benches/          # Performance benchmarks
├── rust-devpack/         # Rust SDK for Neo smart contracts
│   ├── neo-runtime/      # Runtime stubs and syscall bindings
│   ├── neo-syscalls/     # Low-level syscall definitions
│   ├── neo-types/        # Neo type system (NeoInteger, NeoByteString, etc.)
│   ├── neo-macros/       # Procedural macros (#[neo_event], #[neo_safe])
│   └── examples/         # Example contract patterns
├── solana-compat/        # Solana compatibility layer (neo-solana-compat)
│   ├── src/              # Pubkey, AccountInfo, ProgramError implementations
│   └── tests/            # Solana API compatibility tests (26 tests)
├── move-neovm/           # Move bytecode to WebAssembly translator (experimental)
│   ├── src/              # Move bytecode parser and WASM lowering
│   └── tests/            # Move translation tests
├── contracts/            # Production-ready example contracts
│   ├── hello-world/      # Basic Neo contract
│   ├── nep17-token/      # NEP-17 fungible token standard
│   ├── nep11-nft/        # NEP-11 non-fungible token standard
│   ├── solana-hello/     # Cross-chain Solana program example
│   ├── move-coin/        # Move-style coin with resource semantics
│   ├── multisig-wallet/  # Multi-signature wallet
│   ├── escrow/           # Escrow contract
│   ├── governance-dao/   # Decentralized autonomous organization
│   ├── oracle-consumer/  # Oracle integration example
│   └── ...               # Additional templates
├── integration-tests/    # End-to-end integration tests
│   └── tests/            # Neo Express deployment and invocation tests
├── scripts/              # Build and deployment helper scripts
│   ├── build_contract.sh      # Rust contract build + translate
│   ├── build_c_contract.sh    # C contract build + translate
│   └── neoexpress_deploy.sh   # Neo Express deployment helper
├── spec/                 # Formal specifications (LaTeX)
│   └── wasm-neovm-spec.tex    # Normative translation specification
├── CONTRIBUTING.md       # Contribution guidelines
├── SECURITY.md           # Security policy and vulnerability reporting
├── CODE_OF_CONDUCT.md    # Community code of conduct
├── CHANGELOG.md          # Version history and release notes
└── Makefile              # Build automation (examples, tests, cross-chain)
```

## Contributing

We welcome contributions from the community! Please read our [Contributing Guidelines](CONTRIBUTING.md) to get started.

### Quick Contribution Checklist

- Read [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and workflow
- Follow the [Code of Conduct](CODE_OF_CONDUCT.md)
- Check [open issues](https://github.com/r3e-network/neo-llvm/issues) for tasks
- Review [SECURITY.md](SECURITY.md) for security reporting procedures
- Join our [Discord](https://discord.io/neo) for discussions

### Development Roadmap

- Broaden instruction coverage with floating-point/SIMD lowering once the current integer semantics are fully settled.
- Surface additional devpack metadata (events, permissions, supported standards) directly into manifest generation so JSON overlays remain optional.
- Tighten end-to-end validation by replaying generated NEFs in the NeoVM reference runner / neo-cli as part of CI.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/r3e-network/neo-llvm/issues)
- **Discord**: [https://discord.io/neo](https://discord.io/neo)
- **Neo Developer Portal**: [https://neo.org/dev](https://neo.org/dev)

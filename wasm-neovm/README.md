# wasm-neovm

WebAssembly to NeoVM translator - converts WASM modules to Neo N3 compatible NEF format.

## Overview

This crate provides a translator that compiles WebAssembly (WASM) modules into NeoVM bytecode (NEF format), enabling smart contract development in languages that compile to WASM (Rust, C, C++, etc.) for the Neo N3 blockchain.

## Features

- **WASM to NeoVM Translation**: Converts WASM bytecode to NeoVM opcodes
- **Manifest Generation**: Automatically generates Neo N3 contract manifests
- **Cross-chain Support**: Adapters for compiling from other chain formats (Solana, Move)
- **Metadata Extraction**: Extracts and embeds contract metadata in NEF files

## Usage

### As a Library

```rust
use wasm_neovm::{translate_module, write_nef, SourceChain};

fn main() -> anyhow::Result<()> {
    let wasm_bytes = std::fs::read("contract.wasm")?;
    let translation = translate_module(&wasm_bytes, "MyContract")?;
    
    // Write the NEF file
    write_nef(&translation.script, "contract.nef")?;
    
    // The manifest is available as JSON
    let manifest_json = translation.manifest.to_string()?;
    
    Ok(())
}
```

### As a CLI Tool

```bash
# Basic usage
wasm-neovm input.wasm --name MyContract

# With manifest overlay
wasm-neovm input.wasm --name MyContract --manifest-overlay overlay.json

# Compare generated manifest against reference
wasm-neovm input.wasm --name MyContract --compare-manifest reference.json

# Specify output paths
wasm-neovm input.wasm --name MyContract --nef output.nef --manifest manifest.json
```

## Project Structure

```
src/
├── adapters/          # Cross-chain compilation adapters
│   └── solana/       # Solana program adapter
├── manifest/         # Contract manifest generation
│   ├── build.rs      # Manifest building
│   ├── builder.rs    # Manifest builder API
│   ├── merge.rs      # Manifest merging utilities
│   └── parity.rs     # Method parity checking
├── metadata/         # NEF metadata handling
├── translator/       # Core WASM translation logic
│   ├── frontend.rs   # WASM module parsing
│   ├── ir.rs         # Intermediate representation
│   ├── runtime/      # Runtime helper generation
│   └── translation/  # Translation passes
├── cli.rs            # Command-line interface
├── lib.rs            # Library exports
└── main.rs           # CLI entry point
```

## Architecture

The translation process follows these steps:

1. **Parsing**: WASM module is parsed using `wasmparser`
2. **Frontend**: Module structure is analyzed and validated
3. **Translation**: WASM instructions are translated to NeoVM opcodes
4. **Runtime Generation**: Helper functions for memory, tables, etc. are generated
5. **Manifest Building**: Contract manifest is generated from exported functions
6. **NEF Generation**: Final NEF file with metadata is produced

## Dependencies

- `wasmparser` - WASM parsing
- `serde` / `serde_json` - JSON handling for manifests
- `clap` - CLI argument parsing
- `anyhow` / `thiserror` - Error handling

## Testing

```bash
# Run tests
cargo test --all-features

# Run benchmarks
cargo bench
```

## Neo Source Verification

`build.rs` prefers local canonical sources:
- Syscalls from `../neo/src/Neo/SmartContract`
- Opcodes from `../neo-vm/src/Neo.VM/OpCode.cs` (or legacy `../neo/src/Neo.VM/OpCode.cs`)

When those sources are missing, the crate falls back to bundled snapshots in
`src/generated/`.

For CI or release pipelines that must not use fallback snapshots, set:

```bash
WASM_NEOVM_REQUIRE_NEO_CHECKOUT=1 cargo build --all-features
```

This makes the build fail if the required canonical sources are missing or incomplete.

## License

See the workspace root LICENSE file.

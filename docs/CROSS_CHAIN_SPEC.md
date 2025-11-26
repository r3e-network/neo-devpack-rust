# Cross-Chain Compilation Specification

Version: 1.0.0
Status: Production Ready
Date: 2025-01-20

## Overview

This specification defines the cross-chain compilation pipeline that translates smart contracts from Solana (Rust/WASM) and Move-based chains (Aptos, Sui) to Neo N3 NeoVM format (NEF + Manifest).

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Cross-Chain Compilation Pipeline                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐    ┌──────────────┐    ┌────────────────────────────┐ │
│  │ Source Chain │    │  Intermediate│    │      Target Output         │ │
│  │   Contract   │───▶│    Format    │───▶│                            │ │
│  └──────────────┘    └──────────────┘    └────────────────────────────┘ │
│                                                                          │
│  Solana Path:                                                            │
│  ┌──────────┐  ┌───────────────┐  ┌─────────┐  ┌──────────────────────┐ │
│  │  Rust    │─▶│solana-compat  │─▶│  WASM   │─▶│   wasm-neovm         │ │
│  │ Contract │  │(API shimming) │  │ Binary  │  │ (NEF + Manifest)     │ │
│  └──────────┘  └───────────────┘  └─────────┘  └──────────────────────┘ │
│                                                                          │
│  Move Path:                                                              │
│  ┌──────────┐  ┌───────────────┐  ┌─────────┐  ┌──────────────────────┐ │
│  │   Move   │─▶│  move-neovm   │─▶│  WASM   │─▶│   wasm-neovm         │ │
│  │ Bytecode │  │ (translator)  │  │ Binary  │  │ (NEF + Manifest)     │ │
│  └──────────┘  └───────────────┘  └─────────┘  └──────────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## 1. Solana Compatibility Layer

### 1.1 Package: `neo-solana-compat`

Location: `solana-compat/`

#### Purpose
Provides a drop-in replacement for the `solana_program` crate that compiles to WASM and maps Solana concepts to Neo equivalents.

#### API Compatibility

| Solana API | Neo-Solana-Compat | Notes |
|------------|-------------------|-------|
| `Pubkey` | `Pubkey` | 32-byte key, converts to UInt160 |
| `AccountInfo` | `AccountInfo` | Maps to storage operations |
| `ProgramError` | `ProgramError` | Full enum support |
| `entrypoint!` | `entrypoint!` | WASM export generation |
| `invoke()` | `invoke()` | Maps to System.Contract.Call |

#### Usage

```rust
// Replace solana_program imports
use neo_solana_compat::prelude::*;

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Solana-compatible logic
    Ok(())
}
```

### 1.2 Syscall Mapping

| Solana Syscall | Neo Interop Service | Notes |
|----------------|---------------------|-------|
| `sol_log` | `System.Runtime.Log` | Direct mapping |
| `sol_sha256` | `Neo.Crypto.SHA256` | Direct mapping |
| `sol_keccak256` | `Neo.Crypto.Keccak256` | Available in Neo |
| `sol_get_clock_sysvar` | `System.Runtime.GetTime` | Unix timestamp |
| `sol_invoke` | `System.Contract.Call` | Cross-contract calls |
| `sol_verify_signature` | `System.Runtime.CheckWitness` | Different sig scheme |
| Account read/write | `System.Storage.Get/Put` | Storage mapping |

### 1.3 Semantic Differences

#### Account Model
- **Solana**: Account-based with owner programs
- **Neo**: Contract storage slots
- **Mapping**: Account data → Storage keys prefixed by pubkey

#### Signatures
- **Solana**: Ed25519
- **Neo**: secp256r1, secp256k1, Ed25519
- **Mapping**: CheckWitness for authorization

#### Rent
- **Solana**: Rent-exempt threshold
- **Neo**: GAS fees (no rent)
- **Mapping**: N/A (fees are different model)

## 2. Move Language Support

### 2.1 Package: `move-neovm`

Location: `move-neovm/`

#### Purpose
Translates Move bytecode to WASM, enabling Move contracts to run on NeoVM.

#### Pipeline
```
Move Source (.move)
       ↓
Move Compiler (external)
       ↓
Move Bytecode (.mv)
       ↓
move-neovm (parse + translate)
       ↓
WASM Module (.wasm)
       ↓
wasm-neovm
       ↓
NEF + Manifest
```

### 2.2 Bytecode Format

Move bytecode magic: `0xa1 0x1c 0xeb 0x0b`

#### Supported Opcodes

| Move Opcode | WASM Equivalent | Notes |
|-------------|-----------------|-------|
| `LdU64` | `i64.const` | Load constant |
| `Add/Sub/Mul/Div` | `i64.add/sub/mul/div_s` | Arithmetic |
| `Lt/Gt/Le/Ge/Eq` | `i64.lt_s/gt_s/le_s/ge_s/eq` | Comparison |
| `CopyLoc/MoveLoc` | `local.get` | Local access |
| `StLoc` | `local.set` | Local store |
| `Branch/BrTrue/BrFalse` | `br/br_if` | Control flow |
| `Call` | `call` | Function call |
| `Ret` | `return` | Function return |
| `BorrowGlobal` | Storage.Get | Resource access |
| `MoveTo/MoveFrom` | Storage.Put/Delete | Resource ops |

### 2.3 Resource Semantics

Move's linear types are emulated via:

1. **Storage Tracking**: Resources stored with type-prefixed keys
2. **Existence Checks**: `exists<T>()` → Storage.Get != null
3. **Runtime Validation**: ResourceTracker ensures linear semantics

```rust
// Storage key format
fn global_storage_key(address: &[u8], type_name: &str) -> Vec<u8> {
    // "R" + address + ":" + type_name
}
```

### 2.4 Standard Library Mapping

| Move Stdlib | Neo Equivalent |
|-------------|---------------|
| `hash::sha256` | `Neo.Crypto.SHA256` |
| `timestamp::now_seconds` | `System.Runtime.GetTime` |
| `event::emit` | `System.Runtime.Notify` |
| `signer::address_of` | `System.Runtime.GetCallingScriptHash` |

## 3. Adapter System

### 3.1 Interface: `ChainAdapter`

```rust
pub trait ChainAdapter {
    fn source_chain(&self) -> SourceChain;
    fn resolve_syscall(&self, module: &str, name: &str) -> Option<&'static str>;
    fn recognizes_module(&self, module: &str) -> bool;
}
```

### 3.2 Supported Chains

| Chain | Identifier | Status |
|-------|------------|--------|
| Neo (native) | `neo`, `native` | ✅ Production |
| Solana | `solana`, `sol` | ✅ Production |
| Move (Aptos/Sui) | `move`, `aptos`, `sui` | ✅ Functional |

## 4. CLI Usage

### 4.1 Solana Contract

```bash
# Step 1: Build with neo-solana-compat
cargo build --manifest-path contracts/solana-hello/Cargo.toml \
  --target wasm32-unknown-unknown --release

# Step 2: Translate to NEF
cargo run --manifest-path wasm-neovm/Cargo.toml -- \
  --input contracts/solana-hello/target/wasm32-unknown-unknown/release/solana_hello.wasm \
  --nef build/solana_hello.nef \
  --manifest build/solana_hello.manifest.json \
  --name solana-hello \
  --source-chain solana
```

### 4.2 Move Contract

```bash
# Using move-neovm library
use move_neovm::{parse_move_bytecode, translate_to_wasm};

let module = parse_move_bytecode(&bytecode)?;
let wasm = translate_to_wasm(&module)?;

# Then translate WASM to NEF as above
```

### 4.3 Move-Style Rust Contract

For Move-inspired patterns in Rust:

```bash
cargo build --manifest-path contracts/move-coin/Cargo.toml \
  --target wasm32-unknown-unknown --release

cargo run --manifest-path wasm-neovm/Cargo.toml -- \
  --input contracts/move-coin/target/wasm32-unknown-unknown/release/move_coin.wasm \
  --nef build/MoveCoin.nef \
  --manifest build/MoveCoin.manifest.json \
  --name MoveCoin \
  --source-chain move
```

## 5. Manifest Generation

### 5.1 Method Token Generation

Imported syscalls are converted to NEF method tokens:

```json
"extra": {
  "nefMethodTokens": [
    {
      "hash": "0x0000000000000000000000000000000000000000",
      "method": "System.Runtime.Log",
      "parametersCount": 1,
      "hasReturnValue": false,
      "callFlags": "All"
    }
  ]
}
```

### 5.2 Feature Detection

| Import Pattern | Feature Enabled |
|----------------|----------------|
| `storage_get/put` | `"storage": true` |
| `contract_call` | Dynamic contract calls |
| `check_witness` | Signature verification |

## 6. Limitations

### 6.1 Solana
- Parallel execution not replicated
- PDA derivation uses different algorithm
- Ed25519 signatures require CheckWitness workaround

### 6.2 Move
- Resource linearity is runtime-checked, not compile-time enforced
- Generic type instantiation simplified
- Formal verification properties not preserved

### 6.3 General
- Gas/fee models incompatible
- Original chain security guarantees may differ
- Some native types require emulation

## 7. Testing

### 7.1 Unit Tests

```bash
# Solana compat tests (26 tests)
cargo test --manifest-path solana-compat/Cargo.toml

# Move translator tests (17 tests)
cargo test --manifest-path move-neovm/Cargo.toml

# Cross-chain integration tests
cargo test --manifest-path wasm-neovm/Cargo.toml cross_chain
```

### 7.2 E2E Validation

```bash
# Build all examples including cross-chain
make examples
```

## 8. Security Considerations

1. **Signature Scheme Differences**: Ed25519 → secp256r1 mapping loses original verification
2. **Type Safety**: Move linear types emulated, not enforced by VM
3. **Cross-Contract Calls**: Different security models between chains
4. **Audit Requirement**: Cross-compiled contracts should be audited for Neo-specific concerns

## 9. Versioning

| Component | Version | Compatibility |
|-----------|---------|---------------|
| neo-solana-compat | 0.1.0 | Solana ~1.16 API surface |
| move-neovm | 0.1.0 | Move Bytecode v6 |
| wasm-neovm | 0.1.0 | Neo N3 3.x |

## 10. References

- [Neo N3 Documentation](https://docs.neo.org)
- [Solana Program Library](https://github.com/solana-labs/solana-program-library)
- [Move Language Specification](https://github.com/move-language/move)
- [WebAssembly Specification](https://webassembly.github.io/spec/)

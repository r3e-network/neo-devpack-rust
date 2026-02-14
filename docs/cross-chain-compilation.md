# Cross-Chain Smart Contract Compilation to NeoVM

This document outlines the technical architecture for compiling smart contracts from other blockchain platforms (Solana, Move-based chains) to NeoVM NEF format.

## Implementation Status

| Chain | Status | Components |
|-------|--------|------------|
| Solana | ✅ **Working** | `solana-compat/`, `wasm-neovm/src/adapters/solana/mod.rs` |
| Move | ⚠️ **Experimental** | `move-neovm/` lowers Move bytecode → WASM with control flow + storage-backed resources (ability-checked); `wasm-neovm` will auto-run Move bytecode through this step |

## Quick Start

### Solana Program Compilation

```bash
# 1. Build your Solana-style contract for WASM
cargo build --manifest-path contracts/solana-hello/Cargo.toml \
  --target wasm32-unknown-unknown --release

# 2. Translate to NEF
cargo run --manifest-path wasm-neovm/Cargo.toml -- \
  --input contracts/solana-hello/target/wasm32-unknown-unknown/release/solana_hello_neo.wasm \
  --nef build/solana_hello.nef \
  --manifest build/solana_hello.manifest.json \
  --name solana-hello \
  --source-chain solana
```

### Move Contract Compilation

```rust
// Using the move-neovm library
use move_neovm::{parse_move_bytecode, translate_to_wasm};

// Parse Move bytecode (.mv file)
let module = parse_move_bytecode(&bytecode)?;

// Translate to WASM
let wasm = translate_to_wasm(&module)?;

// Then use wasm-neovm to generate NEF from the WASM
```

## 1. Overview

The goal is to extend the existing `wasm-neovm` pipeline to support:
1. **Solana Programs** (Rust/C → BPF/SBF → WASM → NeoVM)
2. **Move Contracts** (Move → Move Bytecode → WASM → NeoVM)

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Cross-Chain Compilation Pipeline                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Solana Path:                                                        │
│  ┌──────────┐    ┌─────────┐    ┌──────────┐    ┌─────────────────┐ │
│  │  Rust/C  │───▶│ WASM32  │───▶│ Syscall  │───▶│ wasm-neovm      │ │
│  │ Contract │    │ Target  │    │ Adapter  │    │ (NEF+Manifest)  │ │
│  └──────────┘    └─────────┘    └──────────┘    └─────────────────┘ │
│                                                                      │
│  Move Path:                                                          │
│  ┌──────────┐    ┌─────────┐    ┌──────────┐    ┌─────────────────┐ │
│  │   Move   │───▶│  Move   │───▶│ Move2Wasm│───▶│ wasm-neovm      │ │
│  │ Contract │    │Bytecode │    │Translator│    │ (NEF+Manifest)  │ │
│  └──────────┘    └─────────┘    └──────────┘    └─────────────────┘ │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 2. Solana Program Compilation

### 2.1 Current Solana Architecture

Solana programs compile to **SBF (Solana BPF)**, a variant of eBPF:
- Target: `sbf-solana-solana` (formerly `bpfel-unknown-unknown`)
- Runtime: Custom BPF VM with Solana-specific syscalls
- Memory model: 32KB stack, heap via bump allocator

### 2.2 Feasibility Analysis

**Option A: Direct WASM Compilation (Recommended)**

Solana programs are written in Rust. The same source can target `wasm32-unknown-unknown`:

```rust
// Solana program structure
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Program logic
    Ok(())
}
```

**Key Challenge**: The `solana_program` crate assumes BPF target. We need:
1. A shim crate `neo-solana-compat` that provides the same API
2. Syscall mapping from Solana → Neo equivalents

### 2.3 Syscall Mapping: Solana → Neo

| Solana Syscall | Description | Neo Equivalent |
|----------------|-------------|----------------|
| `sol_log_` | Logging | `System.Runtime.Log` |
| `sol_invoke_signed` | CPI calls | `System.Contract.Call` |
| `sol_get_clock_sysvar` | Get time | `System.Runtime.GetTime` |
| `sol_sha256` | SHA256 hash | `Neo.Crypto.SHA256` |
| `sol_keccak256` | Keccak256 | `Neo.Crypto.RIPEMD160` (partial) |
| `sol_get_return_data` | Return data | Stack return values |
| `sol_set_return_data` | Set return | Stack manipulation |
| Account data read | Storage read | `System.Storage.Get` |
| Account data write | Storage write | `System.Storage.Put` |

### 2.4 Implementation Plan

```
solana-compat/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Re-exports
│   ├── account_info.rs     # AccountInfo → Neo storage mapping
│   ├── pubkey.rs           # Pubkey → UInt160/UInt256
│   ├── program.rs          # Program invocation → Contract.Call
│   ├── syscalls.rs         # Syscall shims → Neo syscalls
│   └── entrypoint.rs       # Entrypoint macro adaptation
```

**Phase 1**: Create `neo-solana-compat` crate with API surface
**Phase 2**: Implement syscall bridging in `wasm-neovm`
**Phase 3**: Add Solana-specific manifest generation

### 2.5 Semantic Differences

| Aspect | Solana | Neo | Adaptation |
|--------|--------|-----|------------|
| Account Model | Account-based with owner | Contract storage | Map accounts to storage keys |
| Signatures | Ed25519 | Various (secp256r1, etc.) | `CheckWitness` for auth |
| Rent | Rent exemption required | GAS fees | Different fee model |
| PDAs | Program Derived Addresses | Contract hashes | Map PDA → contract hash |
| CPI | Cross-program invocation | Contract.Call | Direct mapping |

## 3. Move Language Compilation

> **Status:** Move bytecode translation is experimental but now includes control flow lowering,
> storage-backed resource operations, and ability checks. Some Move semantics remain unsupported.

### 3.1 Current Move Architecture

Move compiles to **Move Bytecode**, a stack-based VM format:
- Platforms: Aptos, Sui, 0L, Starcoin
- Features: Resource types, linear logic, formal verification
- VM: Custom Move VM with module/resource semantics

### 3.2 Feasibility Analysis

**Challenge Level: HIGH**

Move's type system fundamentally differs from WASM/NeoVM:
- **Resources**: Move enforces linear types (no copy/drop without ability)
- **Modules**: First-class module system with publishing
- **Global Storage**: Typed global storage model

**Option A: Move Bytecode → WASM Translation**

Create a `move2wasm` translator that:
1. Parses Move bytecode
2. Converts stack operations to WASM equivalents
3. Emulates resource semantics via runtime checks

**Option B: Move Source → Rust → WASM (Easier)**

Provide a Move-like DSL in Rust that compiles to WASM:
```rust
// move-like syntax via Rust macros
neo_move::module! {
    module 0x1::Coin {
        struct Coin has key, store {
            value: u64
        }

        public fun transfer(from: &signer, to: address, amount: u64) {
            // ...
        }
    }
}
```

### 3.3 Move Bytecode Instruction Mapping

| Move Instruction | Description | WASM/NeoVM Equivalent |
|------------------|-------------|----------------------|
| `LdU64` | Load u64 constant | `i64.const` |
| `Add` | Integer add | `i64.add` → `ADD` |
| `MoveLoc` | Move local to stack | `local.get` |
| `StLoc` | Store to local | `local.set` |
| `Call` | Function call | `call` |
| `BorrowGlobal` | Borrow resource | Storage.Get + type check |
| `MoveToSender` | Publish resource | Storage.Put |
| `Exists` | Check resource exists | Storage.Get != null |
| `Pack` | Create struct | Array/struct encoding |
| `Unpack` | Destructure | Array access |

### 3.4 Resource Semantics Emulation

```rust
// Runtime helper for resource tracking
struct ResourceTracker {
    owned: HashMap<TypeTag, HashSet<Address>>,
}

impl ResourceTracker {
    fn move_to(&mut self, addr: Address, type_tag: TypeTag) -> Result<()> {
        if self.owned.get(&type_tag).map_or(false, |s| s.contains(&addr)) {
            return Err("Resource already exists");
        }
        self.owned.entry(type_tag).or_default().insert(addr);
        Ok(())
    }

    fn move_from(&mut self, addr: Address, type_tag: TypeTag) -> Result<()> {
        if !self.owned.get(&type_tag).map_or(false, |s| s.contains(&addr)) {
            return Err("Resource does not exist");
        }
        self.owned.get_mut(&type_tag).unwrap().remove(&addr);
        Ok(())
    }
}
```

### 3.5 Implementation Plan

```
move-neovm/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── bytecode/
│   │   ├── mod.rs          # Move bytecode parser
│   │   ├── instruction.rs  # Instruction definitions
│   │   └── module.rs       # Module structure
│   ├── translator/
│   │   ├── mod.rs          # Main translation logic
│   │   ├── stack.rs        # Stack operation mapping
│   │   ├── resources.rs    # Resource emulation
│   │   └── stdlib.rs       # Move stdlib → Neo mapping
│   └── runtime/
│       ├── mod.rs          # Runtime helpers
│       └── type_check.rs   # Type checking at runtime
```

**Phase 1**: Parse Move bytecode format
**Phase 2**: Translate basic instructions to WASM
**Phase 3**: Implement resource tracking runtime
**Phase 4**: Map Move stdlib to Neo equivalents

## 4. Unified Architecture

### 4.1 Extended wasm-neovm

```
wasm-neovm/
├── src/
│   ├── translator/
│   │   ├── mod.rs
│   │   ├── translation.rs
│   │   ├── runtime.rs
│   │   └── adapters/           # NEW: Chain-specific adapters
│   │       ├── mod.rs
│   │       ├── solana.rs       # Solana syscall mapping
│   │       └── move_compat.rs  # Move runtime helpers
│   ├── frontends/              # NEW: Alternative frontends
│   │   ├── mod.rs
│   │   ├── wasm.rs             # Current WASM frontend
│   │   └── move_bytecode.rs    # Move bytecode frontend
```

### 4.2 CLI Extensions

```bash
# Solana program (via WASM)
wasm-neovm translate \
  --input solana_program.wasm \
  --source-chain solana \
  --nef output.nef \
  --manifest output.manifest.json

# Move program (direct bytecode)
wasm-neovm translate \
  --input move_module.mv \
  --source-chain move \
  --nef output.nef \
  --manifest output.manifest.json
```

### 4.3 Configuration

```toml
# neo-cross-chain.toml
[solana]
enabled = true
account_mapping = "storage"  # or "contract-per-account"
signature_scheme = "checkwitness"

[move]
enabled = true
resource_tracking = "runtime"  # or "compile-time"
stdlib_path = "./move-stdlib-neo"
```

## 5. Limitations and Considerations

### 5.1 Solana Limitations
- **Account Model**: Solana's account model differs fundamentally from Neo's contract storage
- **Parallelism**: Solana's parallel execution cannot be replicated
- **Rent**: No direct equivalent in Neo
- **PDA derivation**: Different address derivation scheme

### 5.2 Move Limitations
- **Resource Types**: Must be emulated, not enforced by VM
- **Formal Verification**: Lost in translation
- **Module Publishing**: Different deployment model
- **Generics**: Complex generic instantiation

### 5.3 Security Considerations
- Cross-chain contracts lose original chain's security guarantees
- Type safety must be re-implemented at runtime
- Signature schemes may differ
- Gas/fee models are incompatible

## 6. Implementation Roadmap

### Phase 1: Solana Basic Support (4-6 weeks)
1. Create `neo-solana-compat` shim crate
2. Implement basic syscall mapping
3. Add account-to-storage translation
4. Test with simple Solana programs

### Phase 2: Move Bytecode Parser (4-6 weeks)
1. Implement Move bytecode deserializer
2. Create instruction → WASM mapping
3. Basic resource tracking runtime
4. Test with simple Move modules

### Phase 3: Full Integration (6-8 weeks)
1. Extend `wasm-neovm` CLI
2. Comprehensive syscall coverage
3. Move stdlib Neo mappings
4. Documentation and examples

### Phase 4: Production Hardening (4 weeks)
1. Security audit
2. Edge case handling
3. Performance optimization
4. Integration tests

## 7. Conclusion

**Solana → NeoVM**: Feasible with moderate effort. The Rust-based toolchain allows direct WASM targeting with syscall adaptation.

**Move → NeoVM**: More challenging due to semantic differences. Requires either bytecode translation or a Move-compatible DSL approach.

Recommended starting point: **Solana support first**, as it shares the Rust/WASM foundation with the existing pipeline.

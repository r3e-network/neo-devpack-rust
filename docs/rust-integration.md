# Rust-to-NeoVM Integration Plan

## End-to-End Pipeline
1. **Rust Frontend**: Standard `rustc` produces MIR.
2. **Codegen Backend**: Custom `rustc_codegen_neovm` crate implementing `CodegenBackend` trait, delegating to LLVM with target triple `neovm-unknown-neo3`.
3. **LLVM IR Generation**: Use existing `rustc_codegen_llvm` to emit optimized LLVM IR modules.
4. **NeoVM Codegen**: Invoke LLVM `TargetMachine` for `neovm` to produce Neo bytecode (`.nef`).
5. **Contract Packaging**: Post-process emitted code to produce
   - `.nef` script bytes
   - `.manifest.json` describing ABI, permissions, features
   - optional debug info (`.nefdbgnfo`, `.pdb.json`).
6. **Deployment Tooling**: CLI (`neo-llvm-ld`) to bundle artifacts, sign deploy transactions, run tests.

## rustc Integration Details
- Provide JSON target spec `neovm-unknown-neo3.json`:
  ```json
  {
    "llvm-target": "neovm-unknown-neo3",
    "pointer-width": 64,
    "arch": "neovm",
    "os": "neo3",
    "cpu": "generic",
    "linker": "neo-ld",
    "executables": false,
    "features": "",
    "data-layout": "e-m:e-p:64:64-i1:8:8-i8:8:8-i16:16:16-i32:32:32-i64:64:64-f32:32:32-f64:64:64-n8:16:32:64",
    "panic-strategy": "abort",
    "executables": false,
    "disable-redzone": true,
    "emit-debug-gdb-scripts": false,
    "target-family": ["neo"],
    "pre-link-args": {},
    "linker-flavor": "ld"
  }
  ```
- Provide `rustc_codegen_neovm` crate (fork of `rustc_codegen_llvm`) that:
  - Forces `panic=abort`.
  - Provides custom intrinsics for syscalls via lang items or `extern "system"` functions.
  - Implements `codegen_crate` to invoke backend and run manifest generator.

## Syscall Binding Strategy
- Auto-generate Rust externs from syscall registry (YAML/JSON).
- Provide attribute macro `#[syscall("System.Runtime.GetTime")]` for manual mapping.
- Use `llvm.x.neo.syscall.*` intrinsics to link to backend `SYSCALL` instructions.

## Serialization & ABI Helpers
- Provide derive macros for `neo_vm::abi::FromStack` and `ToStack` traits to map Rust structs/enums to stack values.
- Follow canonical order: arguments pushed in reverse order, results returned via evaluation stack.
- Provide `neo_vm::storage` module for `System.Storage.*` operations.

## Development Workflow
- `cargo new --lib my_contract` with `edition = "2021"`.
- Add `[lib] crate-type = ["cdylib"]` and `Cargo.toml` target-specific metadata.
- Use `cargo neo-build` (custom Cargo subcommand) to invoke `cargo build --target neovm-unknown-neo3.json -Zbuild-std=core,alloc` with nightly compiler.
- Provide `cargo neo-test` to run simulated execution via NeoVM interpreter (Rust binding around C++ NeoVM or reimplementation).
- Provide `cargo neo-deploy` to package `.nef` + manifest + `deploy.neo` transaction shell.

## Tooling Components
- `neo-asm`: command-line assembler/disassembler using LLVM MC layer.
- `neo-llvm-ld`: packages modules, resolves syscalls, generates manifest.
- `neo-abi-gen`: reads Rust metadata (`rustdoc JSON` or compiler plugin) to emit manifest ABI methods, events, permissions.
- `neo-debug`: attaches to NeoVM debugger, uses DWARF-like info from backend to map instructions to source.

## Testing & CI
- Unit tests for ABI conversions using `cargo test` host-mode (with `cfg(test)` interpreter stubs).
- Integration tests run compiled scripts on NeoVM CLI (via docker or local binary) verifying gas usage and functional correctness.
- Provide reference contracts (NEP-17 token, oracle consumer, storage) as examples.

## Deliverables for Initial Milestone
1. `rustc_codegen_neovm` crate skeleton establishing backend invocation.
2. Target spec JSON file + Cargo target alias.
3. Syscall registry generator.
4. Minimal runtime crate `neo-sdk` with stack conversion traits and storage API stubs.
5. Example contract demonstrating end-to-end build to `.nef`.


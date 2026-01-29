# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Note: This changelog tracks the `wasm-neovm` crate and repo-level tooling. Other crates in
this repository follow independent versioning (currently 0.1.x).

## [Unreleased]

### Added
- Enhanced CI/CD with dependency auditing workflows
- Automated cargo-machete checks for unused dependencies
- Version consistency validation across workspace
- Improved code quality gates

### Changed
- Updated CHANGELOG format to follow Keep a Changelog standards
- Enhanced documentation with additional badges and links

### Fixed
- Minor clippy warning in neo-runtime (unit struct construction)
- Code formatting consistency in json.rs

## [0.4.0] - 2025-01-20

### Added

#### Cross-Chain Compilation Support
- **Solana Compatibility Layer** (`solana-compat/`)
  - Full `neo-solana-compat` crate providing drop-in replacement for `solana_program`
  - Supported types: `Pubkey`, `AccountInfo`, `ProgramError`, `Instruction`
  - `entrypoint!` macro for WASM export generation
  - `invoke()` function mapping to `System.Contract.Call`
  - 26 unit tests covering API compatibility

- **Move Language Support** (`move-neovm/`)
  - Move bytecode parser supporting bytecode v6 format
  - WASM code generator translating Move opcodes
  - Resource semantics emulation via Neo storage
  - Standard library mapping (hash, timestamp, events, signer)
  - 8 unit tests for bytecode translation

- **Cross-Chain Integration Tests**
  - `wasm-neovm/tests/solana_move_integration.rs` with 9 integration tests
  - Solana storage/token contract compilation tests
  - Move coin/NFT contract compilation tests
  - Source chain parsing validation

- **Example Contracts**
  - `contracts/move-coin/` - Move-style fungible token with resource semantics
  - `contracts/solana-hello/` - Solana-compatible hello world contract

- **Documentation**
  - `docs/CROSS_CHAIN_SPEC.md` - Full technical specification
  - Updated README with cross-chain compilation usage examples
  - Syscall mapping tables and architecture diagrams

#### Translator Improvements
- Chain adapter system for syscall resolution
- `SourceChain` enum supporting Neo, Solana, and Move identifiers
- Enhanced manifest generation with cross-chain metadata

### Changed
- Updated README to reflect production-ready cross-chain support
- Feature checklist now includes cross-chain compilation components
- Directory layout documentation includes new crates

### Fixed
- `scripts/build_c_contract.sh` - Changed invalid `-mattr=` flags to `-mno-*` format for clang 18+ compatibility

## [0.3.0] - 2025-01-15

### Added
- Production-grade Rust contract examples (10 templates)
- NEP-17/NEP-11 token implementations
- Multisig, escrow, DAO, oracle contract templates
- NFT marketplace example
- Makefile automation for building all examples
- Manifest overlay merge and permission deduplication
- Method-token inference for syscall usage

### Changed
- Improved translator error messages
- Enhanced NEF generation with proper method tokens

## [0.2.0] - 2025-01-10

### Added
- Full support for linear memory operations
- `call_indirect` lowering with bounds checking
- Reference types (funcref) support
- Table operations (`table.get/set/size/grow/fill/copy`)
- Bulk memory instructions (`memory.fill/copy/init`, `data.drop`)
- Control flow improvements (`br_table`, multi-value blocks)

### Changed
- Improved stack height tracking
- Better literal propagation through locals

## [0.1.0] - 2025-01-01

### Added
- Initial WASM → NeoVM translation pipeline
- Basic integer arithmetic and comparisons
- Bitwise operations and shifts
- Local/global variable support
- Neo syscall and opcode import bridges
- NEF + manifest generation
- Rust DevPack for contract authoring

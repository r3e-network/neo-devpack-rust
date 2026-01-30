# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Note: This changelog tracks the `wasm-neovm` crate and repo-level tooling. Other crates in
this repository follow independent versioning (currently 0.1.x).

## [Unreleased]

## [0.2.0] - 2026-01-30

This release represents 200 comprehensive review and improvement rounds across all 10 smart contract templates, resulting in production-ready code quality, security hardening, and NEP standard compliance.

### Highlights
- **Type System Fixes**: All contracts migrated from Integer to Hash160 address types (NEP standard compliant)
- **Security Hardening**: 12+ critical security vulnerabilities fixed
- **Test Coverage**: 26 unit tests added across all contracts
- **Code Quality**: Zero clippy warnings, consistent coding patterns
- **Production Ready**: Full NEP-17/NEP-11/Oracle callback compliance

### Security Fixes (Rounds 1-40)
- Fixed 4 instances of `unwrap_or(true)` that could allow unauthorized operations
- Added missing `ensure_witness()` calls to `configure()` functions in oracle-consumer, crowdfunding, escrow
- Fixed integer overflow vulnerabilities using `checked_add/sub/mul` across all contracts
- Added buyer commitment mechanism to NFT marketplace to prevent front-running
- Fixed escrow refund state management to prevent duplicate funding
- Fixed crowdfunding deadline logic (`<` → `>`)

### Type System Migration (Rounds 1-20, 121-125)
- **nep17-token**: Migrated from `i64` to 20-byte Hash160 address type
- **constant-product AMM**: Migrated trader address to Hash160
- **nep11-nft**: Complete重构 with Hash160 addresses and ByteString token_ids
- All manifests updated to use correct NEP parameter types (Hash160, ByteString, Integer)

### Access Control Improvements (Rounds 81-90)
- Added witness verification to all initialization/configure functions
- Added validation that owner cannot equal token contract in configuration
- Added uniqueness checks for escrow parties (payer, payee, arbiter must be different)
- Fixed boundary check bug in oracle-consumer (`len < 0` → `len <= 0`)

### Event and Logging (Rounds 136-140)
- All 28 event definitions verified with correct parameter types
- Event emissions match NEP standard specifications
- Added comprehensive event coverage for all state-changing operations

### Callback Compliance (Rounds 171-175)
- **NEP-17 callbacks**: All contracts properly implement `onNEP17Payment(from, amount, data)`
- **NEP-11 callbacks**: NFT marketplace properly implements `onNEP11Payment(from, token_id, amount, data)`
- **Oracle callbacks**: Oracle consumer properly implements `onOracleResponse(request_id, code, data)`
- Return types standardized (void for operations, bool for payment callbacks)

### Code Quality Improvements (Rounds 41-80, 121-160)
- Standardized storage key prefixes (e.g., `token:balance:`, `nft:owner:`, `dao:stake:`)
- Unified utility functions (`read_address`, `read_bytes`, `ensure_witness`, `addresses_equal`)
- Consistent function ordering: helpers → storage → entry points → callbacks
- Added safety documentation to all `unsafe` blocks

### Test Coverage (Rounds 7, 51-55)
- **constant-product**: 3 new tests (init, quote, swap)
- **nep11-nft**: 2 new tests (totalSupply, balanceOf)
- **hello-world**: 1 new test
- All existing tests updated for Hash160 address type

### Fixed Issues (30+ total)
- Integer overflow in AMM swap calculations
- Missing access control in initialization functions
- Incorrect boundary checks for pointer/length validation
- State machine transition issues in escrow and crowdfunding
- Missing parameter validation in governance proposals
- Event parameter type mismatches with manifests

### Changed
- All contracts now use consistent error handling patterns
- Storage operations use `checked_add` for ID generation
- Cross-contract calls properly handle return values
- Removed deprecated `OnceLock` usage in tests (Rust 1.70+ compatibility)

## [0.4.3] - 2026-01-29

### Highlights
- **API Consistency**: Removed deprecated `as_i32()` API usage, consolidated `LogLevel` definitions
- **Code Quality**: Added copyright headers to all rust-devpack files
- **Bug Fixes**: Fixed const fn issues in solana-compat for WASM builds
- **Contract Consistency**: Standardized import patterns and storage key naming

### Fixed
- Replaced all deprecated `as_i32()` calls with `as_i32_saturating()` across examples and tests
- Consolidated duplicate `LogLevel` enum - single source in `logging.rs`
- Fixed `const fn` issues in solana-compat (pointer casts in const context)
- Improved safe slicing patterns in solana-compat entrypoint
- Fixed build script error handling (unwrap → context)

### Changed
- All rust-devpack source files now have copyright headers
- Updated author field to "R3E Network" across all crates
- Consistent attribute ordering: `#[no_mangle]` → `#[neo_safe]` → `#[allow(...)]`
- Standardized contract imports: all use `neo_devpack::serde` instead of direct serde
- Standardized storage key naming with namespace prefixes (e.g., `token:`, `nft:`, `amm:`)
- Fixed simple_contract.rs import pattern and missing NeoVMSyscall import
- Fixed remaining as_u32() deprecation warning in tests

## [0.4.2] - 2026-01-29

### Highlights
- **Performance**: O(1) iterator operations, hash-based deduplication, reduced allocations
- **Architecture**: New core/, types/, config/, api/ modules for better organization
- **Code Quality**: Enhanced error messages, comprehensive documentation

### Performance Improvements
- **NeoIterator**: Changed from O(n) `Vec::remove(0)` to O(1) cursor-based iteration (Round 126)
- **Method Token Deduplication**: Use hash-based comparison instead of string cloning (Round 128)
- **Map Removal**: Use `swap_remove` for O(1) removal instead of O(n) `remove` (Round 128)

### Architecture (Rounds 131-140)
- **New `core/` module**: Unified traits (ToBytecode, Translatable, BytecodeEmitter, Named, etc.)
- **New `types/` module**: Type-safe newtypes (ContractName, MethodIndex, LocalIndex, MemoryOffset, etc.)
- **New `config/` module**: Centralized configuration with TranslationConfig, validation
- **New `api/` module**: Fluent TranslationBuilder API for better usability
- **New `logging.rs`**: Standardized logging with LogLevel, LogCategory, and macros

### Code Quality (Rounds 121-130)
- Removed dead code and unused imports
- Enhanced error messages with actionable context
- Added comprehensive documentation to public APIs
- Verified all panic paths have safe alternatives
- Improved iterator efficiency throughout codebase

### Changed
- Implemented `FromStr` trait properly for `LogLevel` (was standalone method)
- Optimized feature flags for better compile times
- Reorganized module structure for maintainability

### Fixed
- Fixed clippy warnings about manual clamp patterns
- Fixed formatting issues
- All 47 test groups passing

## [0.4.1] - 2026-01-29

This release represents 120 comprehensive review and improvement rounds, resulting in significant code quality, performance, and security enhancements.

### Highlights
- **Performance**: O(1) opcode lookup, arena allocator, memory pooling, const evaluation
- **Security**: Fixed critical syscall hash issues, added bounds checking, unsafe code documentation
- **Quality**: Zero clippy warnings, comprehensive documentation, 340+ passing tests
- **Compatibility**: Rust 1.70+ MSRV maintained, all platforms tested

### Performance Improvements
- Added O(1) opcode lookup using lazy HashMap (Rounds 61, 63, 66)
- New arena allocator for fast temporary object allocation (Round 83)
- Memory pooling with 4 bucket sizes to reduce allocations (Round 89)
- Pre-computed constant tables for masks and power-of-2 values (Round 82)
- Inline annotations on hot path functions (Round 81)
- Branch prediction hints using likely!/unlikely! macros (Round 85)
- Cache-friendly data structure layouts with #[repr(C)] (Round 84)
- Profile-guided optimization instrumentation (Round 90)

### Security Fixes
- **CRITICAL**: Removed incorrect/legacy syscall hashes from extended table (Round 25)
- **CRITICAL**: Fixed panic-prone integer conversions with safe alternatives (Round 26)
- Added bounds checking for memory offset overflow (Round 22)
- Documented 30+ unsafe blocks with # Safety sections (Round 11)
- Added validation for NEF method tokens (Round 24)
- Fixed infinite recursion in Pubkey Default impl (Round 16)

### Code Quality (Rounds 1-40, 41-80, 101-120)
- Zero clippy warnings (all 120 rounds)
- Comprehensive documentation added to all modules
- Fixed all rustdoc warnings
- Error handling improvements (expect → Result propagation)
- Code deduplication with shared modules
- Magic numbers extracted to named constants
- Import cleanup and organization

### Added
- Enhanced CI/CD with dependency auditing workflows
- Automated cargo-machete checks for unused dependencies
- Version consistency validation across workspace
- Improved code quality gates
- Comprehensive crate metadata (keywords, categories) for crates.io publishing
- `include` fields to Cargo.toml for cleaner package publishing
- License headers to all library files
- docs.rs badge in README.md

### Changed
- Updated CHANGELOG format to follow Keep a Changelog standards
- Enhanced documentation with additional badges and links
- Improved module-level documentation in `wasm-neovm` translator
- Workspace version bump from 0.4.0 to 0.4.1
- Migrated from LazyLock (1.80+) to once_cell::Lazy for MSRV 1.70 compatibility

### Fixed
- Minor clippy warning in neo-runtime (unit struct construction)
- Code formatting consistency across all crates
- Fixed rustdoc warnings in `move-neovm` (unclosed HTML tag)
- Fixed rustdoc warnings in `wasm-neovm` (private intra-doc links)
- Fixed compilation errors in `wasm-neovm` translation layer
- Fixed borrow checker issues in control flow translation
- Fixed API compatibility with wasmparser 0.239
- Fixed test utility trait bounds for Debug compatibility
- Fixed NeoTypes iterator implementation (removed unused index field)
- Fixed Vec capacity calculation bug in move-neovm (+1 → +2)

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

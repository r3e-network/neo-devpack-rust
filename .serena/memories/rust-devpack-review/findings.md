# Rust-Devpack SDK Review Findings

## Overview
Systematically reviewed all 5 core rust-devpack SDK crates for correctness, API quality, and Neo N3 specification compliance.

## Review Status: COMPLETE

### Crates Reviewed
1. **neo-types** (type definitions)
2. **neo-syscalls** (syscall bindings)
3. **neo-runtime** (runtime helpers)
4. **neo-macros** (proc macros)
5. **neo-test** (test framework)

### Build Status
- ✅ Builds cleanly with zero warnings
- ✅ All tests pass (4/4 passing)
- ✅ Workspace properly configured

## CRITICAL ISSUES IDENTIFIED

### 1. Incomplete Syscall Coverage
**Severity: HIGH**
- Only 38 syscalls defined vs. Neo N3 specification requirements
- Missing critical syscall categories:
  * **Ledger syscalls** (Ledger.GetBlock, GetTransaction, etc.)
  * **Consensus syscalls** (Consensus.GetValidators, etc.)
  * **Oracle syscalls** (Oracle.GetPrice, etc.)
  * **Attributes syscalls** (Attributes.GetHeight, etc.)
  * **Standard Contracts** (NEO, GAS, Policy, RoleManagement, NameService native contracts)

**Impact**: Contracts cannot call many essential Neo N3 operations

### 2. Missing NEP Standard Support
**Severity: HIGH**
- No explicit NEP support abstractions found:
  * NEP-17 (Token standard) - partially implemented via example but no canonical support
  * NEP-11 (NFT standard) - no implementation
  * NEP-24 (Oracle standard) - no implementation
  * NEP-26 (Royalty standard) - no implementation
- Token example uses storage via NeoMap but lacks formal NEP-17 traits/interfaces

**Impact**: Contracts lack standardized interfaces for interoperability

### 3. Proc Macro Expansion Issues
**Severity: MEDIUM**
- `neo_contract` macro documentation shows type annotations (`TokenStorage: Default, Serialize, Deserialize`) but these are not enforced
- No manifest generation logic visible in expand module (expected functionality missing)
- Event macro generates emit() but parameter ordering/encoding not validated against spec
- Safe methods marker macro exists but integration with contract manifest is unclear

**Impact**: Generated exports may not match Neo N3 specification requirements

### 4. Type System Gaps
**Severity: MEDIUM**
- Missing native Neo N3 types:
  * **Hash160** - no first-class type (using NeoByteString with 20-byte convention)
  * **Hash256** - no first-class type (using NeoByteString with 32-byte convention)
  * **Signature** - no first-class type
  * **PublicKey** - no first-class type
- NeoInteger uses BigInt (correct) but saturating conversions may hide overflow errors
  * `as_i32_saturating()` silently truncates instead of returning Result
  * This violates fail-safe principle for financial code

**Impact**: Type safety issues, potential logic errors in contracts handling large numbers

### 5. Storage API Safety
**Severity: MEDIUM**
- `storage_find()` returns NeoIterator but iterator state management unclear
- No guidance on iterator lifetime or re-entrancy safety during storage iterations
- `NeoStorageContext::is_read_only()` not consistently enforced across all storage operations
- Write operations can silently fail if context doesn't have write permission (no Result return pattern in some cases)

**Impact**: Storage corruption or silent failures possible

### 6. Runtime Context Isolation
**Severity: MEDIUM**
- Host-mode simulation uses global thread-local state (STORAGE_STATE, current_call_flags, etc.)
- No clear documentation of which syscalls require real Wasm context vs. which work in host mode
- `with_contract_invocation()` frame management may not properly simulate nested invocations

**Impact**: Tests may pass in host mode but fail on-chain

### 7. Documentation Deficiencies
**Severity: MEDIUM**
- No specification document mapping syscalls to Neo N3 official documentation
- Macro `#[neo_contract]`, `#[neo_entry]` lack examples showing expected Wasm export names
- NEP standard requirements not documented
- Gas cost values in syscall registry not validated against official Neo N3 spec

## POSITIVE FINDINGS

✅ **Strong Points**:
- Clean, modular architecture with good separation of concerns
- Comprehensive arithmetic operators on NeoInteger
- Serialization framework (postcard) well-integrated
- Test framework environment mocking is reasonable for unit tests
- Error handling with NeoError/NeoResult pattern is consistent
- Type safety for most common operations (boolean, string, array)

## MISSING FUNCTIONALITY

### Critical for Production
1. Hash160/Hash256 first-class types with validation
2. Full NEP-17/NEP-11 trait definitions
3. Comprehensive native contract bindings (NEO, GAS, Policy, RoleManagement)
4. All System.* and Neo.* syscalls from specification
5. Manifest generation and validation

### Important for Robustness
1. Checked integer conversions with proper error propagation
2. Iterator lifetime/safety documentation
3. Storage permission enforcement at type level (newtype wrappers)
4. Syscall parameter validation before host-mode invocation

## RECOMMENDATIONS

### Priority 1 (Blocking for Production)
- [ ] Add Hash160, Hash256 as distinct types with validation
- [ ] Implement NEP-17 trait system with token contract abstractions
- [ ] Complete syscall coverage (add Ledger, Consensus, Oracle, Attributes)
- [ ] Generate proper Wasm exports with manifest metadata

### Priority 2 (Before Mainnet)
- [ ] Replace saturating integer conversions with fallible methods
- [ ] Add storage permission enforcement at type level
- [ ] Document host-mode vs. Wasm execution differences
- [ ] Add syscall spec validation tool

### Priority 3 (Quality of Life)
- [ ] NEP-11, NEP-24, NEP-26 support crates
- [ ] More comprehensive examples (complex storage patterns, nested calls)
- [ ] Benchmark suite for common operations

## TESTING COVERAGE

Current: 4 tests all passing
- Basic type creation tests
- Native contract constant validation

Gaps:
- No syscall wrapper validation tests
- No macro expansion correctness tests
- No NEP standard compliance tests
- No storage permission tests
- No cross-contract call simulation tests

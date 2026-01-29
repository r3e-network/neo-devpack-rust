# Test Organization

This directory contains organized tests for the wasm-neovm translator.

## Directory Structure

```
tests/
├── unit/           # Unit tests for individual components
├── integration/    # Integration tests
├── benchmarks/     # Performance benchmarks
└── README.md       # This file
```

## Test Files

### Unit Tests (Individual Test Files)

- `arithmetic_tests.rs` - Arithmetic operation tests
- `comparison_tests.rs` - Comparison operation tests
- `control_flow_tests.rs` - Control flow tests (branches, loops)
- `edge_case_tests.rs` - Edge case handling
- `error_handling_tests.rs` - Error handling tests
- `function_call_tests.rs` - Function call tests
- `memory_tests.rs` - Memory operation tests
- `memory_operation_tests.rs` - Additional memory tests
- `nef_format_tests.rs` - NEF format tests
- `opcode_consistency.rs` - Opcode consistency tests
- `optimization_tests.rs` - Optimization pass tests
- `stack_tests.rs` - Stack operation tests
- `syscall_tests.rs` - Syscall tests
- `syscall_consistency.rs` - Syscall consistency tests
- `table_tests.rs` - Table operation tests
- `runtime_init_tests.rs` - Runtime initialization tests
- `manifest_coverage.rs` - Manifest coverage tests

### Integration Tests

- `integration_tests.rs` - Main integration tests
- `cross_chain_tests.rs` - Cross-chain compilation tests
- `solana_move_integration.rs` - Solana/Move integration tests
- `example_contract_manifests.rs` - Example contract manifest tests

### Benchmarks

- `benches/translation.rs` - Translation benchmarks

## Running Tests

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --test '*_tests'

# Run specific test category
cargo test arithmetic
cargo test memory
cargo test control_flow

# Run integration tests
cargo test --test integration_tests
cargo test --test cross_chain_tests

# Run benchmarks
cargo bench

# Run with specific features
cargo test --features profiling
cargo test --features validation
```

## Test Naming Conventions

- Unit tests: `test_<component>_<scenario>`
- Integration tests: `test_<feature>_<scenario>`
- Benchmarks: `bench_<operation>_<size>`

## Adding New Tests

1. Add unit tests to the appropriate `*_tests.rs` file
2. Add integration tests to the appropriate integration test file
3. Add benchmarks to `benches/translation.rs`

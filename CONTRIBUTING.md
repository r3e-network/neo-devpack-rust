# Contributing to Neo-LLVM

Thank you for your interest in contributing to the Neo-LLVM project. This document provides guidelines and instructions for contributing to the WebAssembly to NeoVM compilation toolchain.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment Setup](#development-environment-setup)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Code Style Guidelines](#code-style-guidelines)
- [Testing Requirements](#testing-requirements)
- [Pull Request Process](#pull-request-process)
- [Commit Message Format](#commit-message-format)
- [Issue Reporting](#issue-reporting)

## Code of Conduct

This project adheres to the Contributor Covenant Code of Conduct. By participating, you are expected to uphold this code. Please read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for details.

## Getting Started

Before you begin contributing, please:

1. Read the [README.md](README.md) to understand the project's goals and architecture
2. Review existing [issues](https://github.com/neo-project/neo-llvm/issues) and [pull requests](https://github.com/neo-project/neo-llvm/pulls)
3. Check the [documentation](docs/) for technical specifications
4. Join our community discussions on [Discord](https://discord.io/neo)

## Development Environment Setup

### Prerequisites

- **Rust Toolchain**: Install Rust 1.70 or later via [rustup](https://rustup.rs/)
- **WebAssembly Target**: Required for contract compilation
- **Git**: Version control system
- **Optional**: LaTeX tooling for building formal specifications

### Installation Steps

1. **Clone the repository**:

   ```bash
   git clone https://github.com/neo-project/neo-llvm.git
   cd neo-llvm
   ```

2. **Install Rust toolchain and wasm32 target**:

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add wasm32-unknown-unknown
   ```

3. **Install development tools**:

   ```bash
   rustup component add rustfmt clippy
   cargo install cargo-audit cargo-deny cargo-tarpaulin
   ```

4. **Verify installation**:

   ```bash
   cargo --version
   rustc --version
   rustup target list --installed | grep wasm32
   ```

5. **Build the project**:

   ```bash
   cargo build --all
   ```

6. **Run tests**:
   ```bash
   cargo test --all
   ```

## Project Structure

```
neo-llvm/
├── wasm-neovm/           # Core WebAssembly to NeoVM translator
├── rust-devpack/         # Rust SDK for Neo smart contracts
├── solana-compat/        # Solana compatibility layer
├── move-neovm/           # Move bytecode translator (experimental)
├── contracts/            # Example smart contracts
├── integration-tests/    # End-to-end integration tests
├── scripts/              # Build and deployment helper scripts
├── docs/                 # Technical documentation
└── spec/                 # Formal specifications (LaTeX)
```

### Key Components

- **wasm-neovm**: Translates WebAssembly modules to NeoVM bytecode (NEF format)
- **rust-devpack**: Provides types, macros, and runtime stubs for contract development
- **solana-compat**: Cross-chain compatibility layer for Solana programs
- **move-neovm**: Experimental Move bytecode to WebAssembly translator

## Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 2. Make Your Changes

- Write clean, idiomatic Rust code
- Follow the project's code style guidelines
- Add or update tests as needed
- Update documentation if you change APIs or behavior

### 3. Test Your Changes

```bash
# Run all tests
cargo test --all

# Run specific crate tests
cargo test --manifest-path wasm-neovm/Cargo.toml

# Run cross-chain tests
make test-cross-chain

# Build all examples
make examples
```

### 4. Format and Lint

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings
```

### 5. Commit Your Changes

Follow the [Commit Message Format](#commit-message-format) guidelines.

### 6. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a pull request on GitHub.

## Code Style Guidelines

### Rust Style

We follow the official [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/) with these additional conventions:

#### Formatting

- **Use `rustfmt`**: All code must be formatted with `cargo fmt`
- **Line length**: Maximum 100 characters (enforced by rustfmt)
- **Indentation**: 4 spaces (no tabs)
- **Imports**: Group and sort imports using rustfmt

#### Naming Conventions

- **Types**: `PascalCase` (e.g., `NeoVmTranslator`, `WasmModule`)
- **Functions/Methods**: `snake_case` (e.g., `translate_instruction`, `emit_syscall`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_STACK_SIZE`, `NEF_MAGIC`)
- **Modules**: `snake_case` (e.g., `wasm_parser`, `nef_builder`)

#### Documentation

- **Public APIs**: Must have doc comments (`///`)
- **Modules**: Should have module-level documentation (`//!`)
- **Examples**: Include usage examples in doc comments when appropriate
- **Panics**: Document panic conditions with `# Panics` section
- **Errors**: Document error conditions with `# Errors` section

Example:

````rust
/// Translates a WebAssembly module to NeoVM bytecode.
///
/// # Arguments
///
/// * `wasm_bytes` - The WebAssembly module bytes
/// * `config` - Translation configuration options
///
/// # Returns
///
/// Returns a `NefFile` containing the translated bytecode and metadata.
///
/// # Errors
///
/// Returns `TranslationError` if:
/// - The WebAssembly module is invalid
/// - Unsupported instructions are encountered
/// - Memory limits are exceeded
///
/// # Examples
///
/// ```
/// use wasm_neovm::{translate, TranslationConfig};
///
/// let wasm_bytes = include_bytes!("contract.wasm");
/// let config = TranslationConfig::default();
/// let nef = translate(wasm_bytes, config)?;
/// ```
pub fn translate(wasm_bytes: &[u8], config: TranslationConfig) -> Result<NefFile, TranslationError> {
    // Implementation
}
````

#### Error Handling

- **Use `Result<T, E>`**: Prefer `Result` over panics for recoverable errors
- **Use `thiserror`**: Define custom error types with `thiserror` crate
- **Avoid `unwrap()`**: Use `?` operator or `expect()` with descriptive messages
- **Context**: Add context to errors using `anyhow::Context` or custom error types

#### Testing

- **Unit tests**: Place in the same file as the code being tested
- **Integration tests**: Place in `tests/` directory
- **Test naming**: Use descriptive names (e.g., `test_translate_i32_add_instruction`)
- **Test organization**: Group related tests in modules

### Clippy Lints

We enforce clippy warnings as errors in CI. Common lints to watch for:

- `clippy::all`: All clippy lints
- `clippy::pedantic`: Pedantic lints (with some exceptions)
- `clippy::nursery`: Experimental lints (reviewed case-by-case)

Run clippy before submitting:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## Testing Requirements

### Test Coverage

- **Minimum coverage**: 70% for new code (enforced by CI)
- **Critical paths**: 90%+ coverage for core translation logic
- **Edge cases**: Test boundary conditions and error paths

### Test Types

#### 1. Unit Tests

Test individual functions and methods:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_pushint() {
        let mut emitter = Emitter::new();
        emitter.emit_pushint(42);
        assert_eq!(emitter.bytecode(), &[0x00, 0x2a]); // PUSHINT8 42
    }
}
```

#### 2. Integration Tests

Test end-to-end workflows in `tests/` directory:

```rust
#[test]
fn test_translate_hello_world_contract() {
    let wasm = include_bytes!("fixtures/hello_world.wasm");
    let result = translate(wasm, TranslationConfig::default());
    assert!(result.is_ok());

    let nef = result.unwrap();
    assert_eq!(nef.compiler, "wasm-neovm");
    assert!(!nef.script.is_empty());
}
```

#### 3. Cross-Chain Tests

Test Solana and Move compatibility:

```bash
cargo test --manifest-path wasm-neovm/Cargo.toml cross_chain
cargo test --manifest-path solana-compat/Cargo.toml
cargo test --manifest-path move-neovm/Cargo.toml
```

#### 4. Property-Based Tests

Use `proptest` or `quickcheck` for property-based testing when appropriate.

### Running Tests

```bash
# All tests
cargo test --all

# Specific crate
cargo test --manifest-path wasm-neovm/Cargo.toml

# Specific test
cargo test test_translate_i32_add

# With output
cargo test -- --nocapture

# Coverage report
cargo tarpaulin --out Html --output-dir coverage
```

## Pull Request Process

### Before Submitting

1. **Ensure all tests pass**: `cargo test --all`
2. **Format code**: `cargo fmt --all`
3. **Run clippy**: `cargo clippy --all-targets --all-features -- -D warnings`
4. **Update documentation**: If you changed APIs or behavior
5. **Add tests**: For new features or bug fixes
6. **Update CHANGELOG.md**: Add entry under "Unreleased" section

### PR Requirements

- **Title**: Clear, descriptive title following commit message format
- **Description**: Explain what changes were made and why
- **Issue reference**: Link to related issues (e.g., "Fixes #123")
- **Tests**: Include test results or screenshots if applicable
- **Breaking changes**: Clearly document any breaking changes
- **Checklist**: Complete the PR template checklist

### PR Template

```markdown
## Description

Brief description of changes

## Related Issues

Fixes #123

## Type of Change

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing

- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] All tests pass locally
- [ ] Manual testing performed

## Checklist

- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex logic
- [ ] Documentation updated
- [ ] No new warnings introduced
- [ ] CHANGELOG.md updated
```

### Review Process

1. **Automated checks**: CI must pass (tests, clippy, formatting, security audit)
2. **Code review**: At least one maintainer approval required
3. **Discussion**: Address reviewer feedback and questions
4. **Approval**: Maintainer will merge once approved

### After Merge

- Your contribution will be included in the next release
- You'll be credited in the CHANGELOG.md
- Thank you for contributing!

## Commit Message Format

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting, no logic change)
- **refactor**: Code refactoring (no feature change or bug fix)
- **perf**: Performance improvements
- **test**: Adding or updating tests
- **chore**: Maintenance tasks (dependencies, build config)
- **ci**: CI/CD configuration changes

### Scopes

- **wasm-neovm**: Core translator
- **devpack**: Rust devpack
- **solana**: Solana compatibility
- **move**: Move translator
- **contracts**: Example contracts
- **docs**: Documentation
- **ci**: CI/CD

### Examples

```
feat(wasm-neovm): add support for bulk memory operations

Implement memory.fill, memory.copy, memory.init, and data.drop
instructions with bounds checking and runtime helpers.

Closes #234
```

```
fix(solana): correct Pubkey to UInt160 conversion

The conversion was truncating bytes incorrectly. Now properly
converts 32-byte Solana public keys to 20-byte Neo addresses
using the last 20 bytes.

Fixes #456
```

```
docs(devpack): add examples for storage operations

Add comprehensive examples showing how to use storage_get,
storage_put, and storage_delete with different data types.
```

### Breaking Changes

Prefix the commit body with `BREAKING CHANGE:`:

```
feat(wasm-neovm): change manifest generation API

BREAKING CHANGE: The `generate_manifest` function now requires
a `ManifestConfig` parameter instead of individual arguments.

Migration guide:
- Old: generate_manifest(name, methods, events)
- New: generate_manifest(ManifestConfig { name, methods, events })
```

## Issue Reporting

### Bug Reports

Use the bug report template and include:

- **Description**: Clear description of the bug
- **Steps to reproduce**: Minimal reproduction steps
- **Expected behavior**: What should happen
- **Actual behavior**: What actually happens
- **Environment**: OS, Rust version, project version
- **Logs**: Relevant error messages or logs

### Feature Requests

Use the feature request template and include:

- **Problem**: What problem does this solve?
- **Proposed solution**: How should it work?
- **Alternatives**: Other solutions considered
- **Use cases**: Real-world usage scenarios

### Questions

For questions and support:

- Check existing [documentation](docs/)
- Search [existing issues](https://github.com/neo-project/neo-llvm/issues)
- Ask on [Discord](https://discord.io/neo)

## Additional Resources

- [Neo Documentation](https://docs.neo.org/)
- [WebAssembly Specification](https://webassembly.github.io/spec/)
- [NeoVM Specification](https://github.com/neo-project/neo-vm)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

## License

By contributing to Neo-LLVM, you agree that your contributions will be licensed under the MIT License.

## Questions?

If you have questions about contributing, please:

- Open a [discussion](https://github.com/neo-project/neo-llvm/discussions)
- Ask on [Discord](https://discord.io/neo)
- Email the maintainers

Thank you for contributing to Neo-LLVM!

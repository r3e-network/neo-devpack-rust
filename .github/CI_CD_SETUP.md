# CI/CD Infrastructure Setup

This document describes the CI/CD infrastructure for the neo-llvm project.

## Overview

The CI/CD pipeline is implemented using GitHub Actions and includes:

- Multi-platform testing (Linux, macOS, Windows)
- Code quality checks (clippy, rustfmt)
- Package verification for publishable crates
- Test coverage reporting
- Security vulnerability scanning
- Performance benchmarking
- Automated dependency updates

## GitHub Actions Workflows

### Main CI Pipeline (`.github/workflows/ci.yml`)

The main CI workflow runs on every push and pull request to the `master` branch.

#### Jobs

1. **Test Suite** (`test`)
   - Runs on: Linux, macOS, Windows
   - Rust versions: stable, nightly
   - Installs `wasm32-unknown-unknown` target
   - Tests all crates: wasm-neovm, move-neovm, rust-devpack, integration-tests, solana-compat
   - Uses cargo caching for faster builds

2. **Clippy** (`clippy`)
   - Runs on: Linux (stable)
   - Checks all crates with `--all-targets --all-features`
   - Treats warnings as errors (`-D warnings`)

3. **Rustfmt** (`fmt`)
   - Runs on: Linux (stable)
   - Checks code formatting across all crates
   - Fails if code is not properly formatted

4. **Code Coverage** (`coverage`)
   - Runs on: Linux (stable)
   - Uses `cargo-tarpaulin` for coverage generation
   - Uploads coverage reports to Codecov
   - Separate coverage for each major crate

5. **Package Verification** (`package`)
   - Runs on: Linux (stable)
   - Executes `make package-check`
   - Verifies every publishable crate can be packaged and compiled from its tarball
   - Catches publish-only dependency/API mismatches before release

6. **Security Audit** (`security`)
   - Runs on: Linux (stable)
   - Uses `scripts/run_cargo_audit.sh` to check lockfiles and fall back to the local advisory clone if the RustSec fetch fails
   - Uses `scripts/run_cargo_deny.sh` to enforce cargo-deny policy with the same fallback behavior
   - Fails on advisories, unmaintained crates, and notice-level policy violations

7. **Benchmark** (`benchmark`)
   - Runs on: Linux (stable)
   - Runs criterion benchmarks for wasm-neovm and move-neovm
   - Detects performance regressions (±5% threshold)
   - Uses `benchmark-action/github-action-benchmark`

## Dependabot Configuration (`.github/dependabot.yml`)

Automated dependency updates are configured for:

- All Rust crates (wasm-neovm, move-neovm, rust-devpack, integration-tests, solana-compat)
- GitHub Actions workflows

**Schedule**: Weekly on Mondays at 09:00
**PR Limit**: 10 per crate, 5 for GitHub Actions

## Security Configuration (`deny.toml`)

The `cargo-deny` configuration enforces:

### Advisories

- **Vulnerability**: deny (fails on any vulnerability)
- **Unmaintained**: deny in enforced CI/local checks
- **Yanked**: warn
- **Notice**: deny in enforced CI/local checks

### Licenses

**Allowed**:

- MIT
- Apache-2.0
- Apache-2.0 WITH LLVM-exception
- BSD-2-Clause, BSD-3-Clause
- ISC, Unicode-DFS-2016, Zlib, 0BSD

**Denied**:

- GPL-2.0, GPL-3.0, AGPL-3.0

### Bans

- Multiple versions: warn
- Wildcards: warn

### Sources

- Only allows crates.io registry
- Warns on unknown registries/git sources

## Benchmarks

### wasm-neovm (`wasm-neovm/benches/translation.rs`)

Benchmarks for WebAssembly to NeoVM translation:

- Simple WASM translation
- Control flow translation
- Different configuration options
- Repeated translations (1, 10, 100 iterations)

### move-neovm (`move-neovm/benches/bytecode_translation.rs`)

Benchmarks for Move bytecode to WASM translation:

- Move bytecode parsing
- WASM translation
- Different module sizes
- End-to-end translation

## Setup Requirements

### For Contributors

1. **Install Rust toolchain**:

   ```bash
   rustup install stable nightly
   rustup target add wasm32-unknown-unknown
   ```

2. **Install development tools**:

   ```bash
   rustup component add clippy rustfmt
   cargo install cargo-tarpaulin cargo-audit cargo-deny
   ```

3. **Run checks locally**:

   ```bash
   # Format check
   cargo fmt --all -- --check

   # Clippy
   cargo clippy --all-targets --all-features -- -D warnings

   # Tests
   cargo test --all-features

   # Security audit
   make security-check

   # Package verification
   make package-check

   # Benchmarks
   cargo bench
   ```

### For Repository Maintainers

1. **Codecov Setup**:
   - Add `CODECOV_TOKEN` to repository secrets
   - Get token from https://codecov.io/

2. **GitHub Actions Permissions**:
   - Ensure Actions have write permissions for benchmark results
   - Settings → Actions → General → Workflow permissions

3. **Branch Protection**:
   - Require CI checks to pass before merging
   - Settings → Branches → Branch protection rules

## Monitoring

### CI Status

- Check GitHub Actions tab for workflow runs
- All jobs must pass for PR approval

### Coverage Reports

- View at Codecov dashboard
- Target: >80% coverage for core crates

### Security Alerts

- GitHub Security tab shows Dependabot alerts
- Review and merge security PRs promptly

### Performance

- Benchmark results tracked over time
- Alerts on ±5% performance regression

## Troubleshooting

### CI Failures

**Test failures**:

- Check test logs in GitHub Actions
- Run tests locally: `cargo test --all-features`

**Clippy warnings**:

- Fix warnings: `cargo clippy --fix`
- Run locally: `cargo clippy --all-targets --all-features`

**Format issues**:

- Auto-fix: `cargo fmt --all`
- Check: `cargo fmt --all -- --check`

**Coverage upload fails**:

- Verify `CODECOV_TOKEN` is set
- Check Codecov service status

**Security audit fails**:

- Review vulnerability details
- Update dependencies: `cargo update`
- Check `deny.toml` for policy violations
- If GitHub advisory-db fetches are flaky, rerun `make security-check`; the wrapper scripts will retry using the local advisory clone when available

**Package verification fails**:

- Run `make package-check` locally
- Inspect the crate tarball compile error, not just workspace builds
- Check whether a packaged crate is using newer local APIs than the published dependency version

**Benchmark regression**:

- Review performance changes
- Profile code if needed
- Adjust threshold if intentional

## Future Enhancements

- [ ] Add deployment pipeline for releases
- [ ] Integrate with Neo testnet for integration tests
- [ ] Add mutation testing
- [ ] Set up nightly builds
- [ ] Add Docker image builds
- [ ] Implement canary deployments

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny)
- [Criterion.rs](https://github.com/bheisler/criterion.rs)

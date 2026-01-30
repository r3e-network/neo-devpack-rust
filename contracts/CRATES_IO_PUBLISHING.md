# crates.io Publishing Guide

## Prerequisites

Before publishing contracts to crates.io, the following底层 crates must be published first:

1. `neo-types` (rust-devpack/neo-types/)
2. `neo-syscalls` (rust-devpack/neo-syscalls/)
3. `neo-runtime` (rust-devpack/neo-runtime/)
4. `neo-macros` (rust-devpack/neo-macros/)
5. `neo-devpack` (rust-devpack/)

## Publishing Steps

### 1. Publish Workspace Crates

```bash
# Publish neo-types
cd rust-devpack/neo-types && cargo publish

# Publish neo-syscalls
cd rust-devpack/neo-syscalls && cargo publish

# Publish neo-runtime
cd rust-devpack/neo-runtime && cargo publish

# Publish neo-macros
cd rust-devpack/neo-macros && cargo publish

# Publish neo-devpack
cd rust-devpack && cargo publish
```

### 2. Update Contract Dependencies

Update each contract's `Cargo.toml` to use crates.io versions:

```toml
[dependencies]
neo-devpack = "0.4"  # Use the published version
serde = { version = "1.0", features = ["derive"] }
```

### 3. Publish Contracts

```bash
# Publish each contract
cd contracts/hello-world && cargo publish
cd contracts/nep17-token && cargo publish
cd contracts/nep11-nft && cargo publish
cd contracts/constant-product && cargo publish
cd contracts/crowdfunding && cargo publish
cd contracts/escrow && cargo publish
cd contracts/governance-dao && cargo publish
cd contracts/multisig-wallet && cargo publish
cd contracts/nft-marketplace && cargo publish
cd contracts/oracle-consumer && cargo publish
```

## Contract Crate Names on crates.io

| Local Name | crates.io Name |
|------------|----------------|
| hello-world | hello-world-neo |
| nep17-token | nep17-token-neo |
| nep11-nft | nep11-nft-neo |
| constant-product | constant-product-neo |
| crowdfunding | crowdfunding-neo |
| escrow | escrow-neo |
| governance-dao | governance-dao-neo |
| multisig-wallet | multisig-wallet-neo |
| nft-marketplace | nft-marketplace-neo |
| oracle-consumer | oracle-consumer-neo |

Note: Consider using `-neo` suffix to avoid naming conflicts.

## Version Compatibility

- Contracts: v0.2.0
- neo-devpack: v0.4.2+ (required)
- Rust: 1.70+

## Verification

After updating dependencies, verify compilation:

```bash
cargo check --manifest-path contracts/<name>/Cargo.toml
```

## Current Status

- [x] Metadata added to all Cargo.toml files
- [ ] neo-devpack workspace crates published
- [ ] Contract dependencies updated
- [ ] Contracts published to crates.io

## Important Notes

1. **Path Dependencies**: Current contracts use `path = "../../rust-devpack"` which cannot be published
2. **Version Matching**: Published contracts must match the neo-devpack version they were tested with
3. **Workspace vs Standalone**: Contracts are currently in a workspace-excluded directory for flexibility
4. **Crates.io Limits**: Publishing requires verified publisher for certain names

# crates.io Publishing Guide

## Prerequisites

Before publishing contracts to crates.io, you need:

1. **crates.io account** with publishing access
2. **API token** set in environment:
   ```bash
   export CRATES_IO_TOKEN=your_token_here
   ```

3. **Published devpack crates** in this order:
   - `neo-types` v0.1.0
   - `neo-syscalls` v0.1.0
   - `neo-runtime` v0.1.0
   - `neo-macros` v0.1.0
   - `neo-devpack` v0.1.0

## Publishing Steps

### Step 1: Publish DevPack Crates

```bash
# Each publish may take a few minutes due to compilation

# 1. Publish neo-types
cd rust-devpack/neo-types
cargo publish --token $CRATES_IO_TOKEN

# 2. Publish neo-syscalls (depends on neo-types)
cd rust-devpack/neo-syscalls
cargo publish --token $CRATES_IO_TOKEN

# 3. Publish neo-runtime (depends on neo-types, neo-syscalls)
cd rust-devpack/neo-runtime
cargo publish --token $CRATES_IO_TOKEN

# 4. Publish neo-macros
cd rust-devpack/neo-macros
cargo publish --token $CRATES_IO_TOKEN

# 5. Publish neo-devpack (depends on all above)
cd rust-devpack
cargo publish --token $CRATES_IO_TOKEN
```

### Step 2: Update Contract Dependencies

Edit each contract's `Cargo.toml` to use crates.io versions:

```toml
[package]
name = "nep17-token"
version = "0.2.0"
# ... metadata ...

[dependencies]
neo-devpack = "0.1"
serde = { version = "1.0", features = ["derive"] }
```

### Step 3: Publish Contracts

```bash
# Publish each contract
cd contracts/hello-world && cargo publish --token $CRATES_IO_TOKEN
cd contracts/nep17-token && cargo publish --token $CRATES_IO_TOKEN
cd contracts/nep11-nft && cargo publish --token $CRATES_IO_TOKEN
cd contracts/constant-product && cargo publish --token $CRATES_IO_TOKEN
cd contracts/crowdfunding && cargo publish --token $CRATES_IO_TOKEN
cd contracts/escrow && cargo publish --token $CRATES_IO_TOKEN
cd contracts/governance-dao && cargo publish --token $CRATES_IO_TOKEN
cd contracts/multisig-wallet && cargo publish --token $CRATES_IO_TOKEN
cd contracts/nft-marketplace && cargo publish --token $CRATES_IO_TOKEN
cd contracts/oracle-consumer && cargo publish --token $CRATES_IO_TOKEN
```

## Automated Publishing

Use the provided script for automated publishing:

```bash
# Preview what will be published
./scripts/publish-to-cratesio.sh --prepare-only

# Actually publish (requires CRATES_IO_TOKEN)
./scripts/publish-to-cratesio.sh
```

## Crates.io Names

| Local Name | crates.io Name | Status |
|------------|----------------|--------|
| hello-world | hello-world-neo | pending |
| nep17-token | nep17-token-neo | pending |
| nep11-nft | nep11-nft-neo | pending |
| constant-product | constant-product-neo | pending |
| crowdfunding | crowdfunding-neo | pending |
| escrow | escrow-neo | pending |
| governance-dao | governance-dao-neo | pending |
| multisig-wallet | multisig-wallet-neo | pending |
| nft-marketplace | nft-marketplace-neo | pending |
| oracle-consumer | oracle-consumer-neo | pending |

Note: Consider using `-neo` suffix to avoid naming conflicts with existing crates.

## Version Compatibility

| Component | Local Version | Published Version | Required |
|-----------|---------------|-------------------|----------|
| neo-types | 0.1.0 | 0.1 | ✓ |
| neo-syscalls | 0.1.0 | 0.1 | ✓ |
| neo-runtime | 0.1.0 | 0.1 | ✓ |
| neo-macros | 0.1.0 | 0.1 | ✓ |
| neo-devpack | 0.1.0 | 0.1 | ✓ |
| contracts | 0.2.0 | 0.2 | ✓ |

## Verification

After updating dependencies, verify compilation:

```bash
# Check all contracts compile
for dir in contracts/*/; do
    echo "Checking $(basename $dir)..."
    cargo check --manifest-path "$dir"Cargo.toml"
done
```

## Troubleshooting

### "dependency not found"
Make sure devpack crates are published first. Check with:
```bash
cargo search neo-types --limit 1
```

### "version mismatch"
Ensure all devpack crates use matching versions (e.g., all 0.1).

### "API rate limited"
Wait a few minutes between publishes or use `--干燥运行` to check first.

## Current Publishing Status

| Step | Status |
|------|--------|
| GitHub Release | ✓ v0.2.0 |
| Metadata Added | ✓ All crates |
| DevPack Published | ⏳ Pending |
| Contracts Published | ⏳ Pending |

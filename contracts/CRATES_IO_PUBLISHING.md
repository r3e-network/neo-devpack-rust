# crates.io Publishing Guide

## Prerequisites

Before publishing contracts to crates.io, you need:

1. **crates.io account** with publishing access
2. **API token** set in environment:
   ```bash
   export CRATES_IO_TOKEN=your_token_here
   ```

3. **Published workspace crates** in this order:
   - `neo-types` v0.5.8
   - `neo-syscalls` v0.5.8
   - `neo-runtime` v0.5.8
   - `neo-macros` v0.5.8
   - `neo-devpack` v0.5.8
   - `neo-test` v0.5.8
   - `move-neovm` v0.5.8
   - `neo-solana-compat` v0.5.8
   - `wasm-neovm` v0.5.8

## Publishing Steps

### Step 1: Publish Workspace Crates

```bash
./scripts/publish-to-cratesio.sh --dry-run
./scripts/publish-to-cratesio.sh --publish
```

The script publishes registry-facing workspace crates in dependency order:
`neo-types`, `neo-syscalls`, `neo-runtime`, `neo-macros`, `neo-devpack`,
`neo-test`, `move-neovm`, `neo-solana-compat`, then `wasm-neovm`.

### Step 2: Update Contract Dependencies

Edit each contract's `Cargo.toml` to use crates.io versions:

```toml
[package]
name = "nep17-token"
version = "0.5.8"
# ... metadata ...

[dependencies]
neo-devpack = "0.5.8"
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
./scripts/publish-to-cratesio.sh --dry-run

# Actually publish (requires CRATES_IO_TOKEN or saved cargo credentials)
./scripts/publish-to-cratesio.sh --publish

# Optional: include example/template contract crates
./scripts/publish-to-cratesio.sh --dry-run --include-contracts
./scripts/publish-to-cratesio.sh --publish --include-contracts
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
| neo-types | 0.5.8 | 0.5.8 | ✓ |
| neo-syscalls | 0.5.8 | 0.5.8 | ✓ |
| neo-runtime | 0.5.8 | 0.5.8 | ✓ |
| neo-macros | 0.5.8 | 0.5.8 | ✓ |
| neo-devpack | 0.5.8 | 0.5.8 | ✓ |
| neo-test | 0.5.8 | 0.5.8 | ✓ |
| move-neovm | 0.5.8 | 0.5.8 | ✓ |
| neo-solana-compat | 0.5.8 | 0.5.8 | ✓ |
| wasm-neovm | 0.5.8 | 0.5.8 | ✓ |
| contracts | 0.5.8 | optional | ✓ |

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
Ensure all workspace crates and contract templates use the same release version
(e.g., all 0.5.8 for this release).

### "API rate limited"
Wait a few minutes between publishes or use `--dry-run` to check first.

## Current Publishing Status

| Step | Status |
|------|--------|
| GitHub Release | ✓ v0.5.8 |
| Metadata Added | ✓ All crates |
| DevPack Published | ⏳ Pending |
| Contracts Published | ⏳ Pending |

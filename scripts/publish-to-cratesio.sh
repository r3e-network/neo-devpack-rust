#!/bin/bash
# publish-to-cratesio.sh - Publish neo-llvm contracts to crates.io
#
# Usage: ./scripts/publish-to-cratesio.sh [--prepare-only]
#
# This script prepares and publishes all contracts to crates.io.
# Requires: cargo, crates.io API token

set -e

echo "=============================================="
echo "Neo N3 Contracts - crates.io Publishing Script"
echo "=============================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if running prepare only
PREPARE_ONLY=false
if [ "$1" == "--prepare-only" ]; then
    PREPARE_ONLY=true
fi

# Check for crates.io token
if [ -z "$CRATES_IO_TOKEN" ]; then
    echo -e "${YELLOW}Warning: CRATES_IO_TOKEN not set${NC}"
    echo "Set it with: export CRATES_IO_TOKEN=your_token"
    echo ""
fi

# Step 1: Publish devpack crates
echo "Step 1: Publishing devpack crates..."
echo "====================================="

publish_crate() {
    local dir=$1
    local name=$(grep "^name" "$dir/Cargo.toml" | head -1 | cut -d'"' -f2)
    local version=$(grep "^version" "$dir/Cargo.toml" | head -1 | cut -d'"' -f2)
    
    echo "Publishing $name v$version..."
    if cargo publish --manifest-path "$dir/Cargo.toml" 2>&1; then
        echo -e "${GREEN}✓ $name published${NC}"
    else
        echo -e "${RED}✗ $name failed to publish${NC}"
        return 1
    fi
}

# Publish neo-types first (it's a dependency of others)
if ! publish_crate "rust-devpack/neo-types"; then
    echo "Failed to publish neo-types"
    exit 1
fi

# Publish neo-syscalls
if ! publish_crate "rust-devpack/neo-syscalls"; then
    echo "Failed to publish neo-syscalls"
    exit 1
fi

# Publish neo-runtime
if ! publish_crate "rust-devpack/neo-runtime"; then
    echo "Failed to publish neo-runtime"
    exit 1
fi

# Publish neo-macros
if ! publish_crate "rust-devpack/neo-macros"; then
    echo "Failed to publish neo-macros"
    exit 1
fi

# Publish neo-devpack
if ! publish_crate "rust-devpack"; then
    echo "Failed to publish neo-devpack"
    exit 1
fi

echo ""
echo "Step 2: Publishing contracts..."
echo "================================="

# Contracts to publish
CONTRACTS=(
    "hello-world"
    "nep17-token"
    "nep11-nft"
    "constant-product"
    "crowdfunding"
    "escrow"
    "governance-dao"
    "multisig-wallet"
    "nft-marketplace"
    "oracle-consumer"
)

for contract in "${CONTRACTS[@]}"; do
    if ! publish_crate "contracts/$contract"; then
        echo "Failed to publish $contract"
        exit 1
    fi
    sleep 2  # Rate limiting
done

echo ""
echo -e "${GREEN}==============================================${NC}"
echo -e "${GREEN}All crates published successfully!${NC}"
echo -e "${GREEN}==============================================${NC}"
echo ""
echo "Published crates:"
for contract in "${CONTRACTS[@]}"; do
    echo "  - $contract"
done

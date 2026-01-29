#!/bin/bash
# Version consistency check script for neo-llvm workspace
# Ensures all crates have consistent versioning

set -e

WORKSPACE_VERSION=$(grep "^version = " Cargo.toml | head -1 | sed 's/.*= "\(.*\)".*/\1/')
echo "Workspace version: $WORKSPACE_VERSION"

echo ""
echo "Checking version consistency across workspace crates..."

# List of crates to check
crates=(
    "wasm-neovm"
    "move-neovm"
    "solana-compat"
    "integration-tests"
    "rust-devpack"
)

inconsistent=0

for crate in "${crates[@]}"; do
    if [ -f "$crate/Cargo.toml" ]; then
        # Check if using workspace inheritance
        if grep -q "version.workspace = true" "$crate/Cargo.toml"; then
            echo "✅ $crate: using workspace version ($WORKSPACE_VERSION)"
        else
            crate_version=$(grep "^version = " "$crate/Cargo.toml" | head -1 | sed 's/.*= "\(.*\)".*/\1/')
            if [ "$crate_version" != "$WORKSPACE_VERSION" ]; then
                echo "❌ $crate: version mismatch (found $crate_version, expected $WORKSPACE_VERSION or workspace inheritance)"
                inconsistent=1
            else
                echo "✅ $crate: $crate_version"
            fi
        fi
    fi
done

# Check rust-devpack subcrates
subcrates=(
    "rust-devpack/neo-types"
    "rust-devpack/neo-syscalls"
    "rust-devpack/neo-runtime"
    "rust-devpack/neo-macros"
)

for crate in "${subcrates[@]}"; do
    if [ -f "$crate/Cargo.toml" ]; then
        # Check if using workspace inheritance
        if grep -q "version.workspace = true" "$crate/Cargo.toml"; then
            echo "✅ $crate: using workspace version"
        else
            crate_version=$(grep "^version = " "$crate/Cargo.toml" | head -1 | sed 's/.*= "\(.*\)".*/\1/')
            # Subcrates use independent 0.1.x versioning
            echo "ℹ️  $crate: $crate_version (independent versioning)"
        fi
    fi
done

if [ $inconsistent -eq 1 ]; then
    echo ""
    echo "Note: Consider using 'version.workspace = true' for consistent versioning"
    exit 1
fi

echo ""
echo "✅ All workspace crate versions are consistent!"

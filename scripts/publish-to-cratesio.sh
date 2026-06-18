#!/bin/bash
# Publish the repository's registry-facing crates to crates.io.
#
# Usage:
#   scripts/publish-to-cratesio.sh --dry-run
#   scripts/publish-to-cratesio.sh --publish
#   scripts/publish-to-cratesio.sh --dry-run --include-contracts
#
# The default release path publishes every registry-facing workspace crate at
# the same version. Contract examples are optional because they are templates,
# not required dependencies of wasm-neovm.

set -euo pipefail

MODE=""
INCLUDE_CONTRACTS=false

for arg in "$@"; do
    case "$arg" in
        --dry-run|--prepare-only)
            MODE="dry-run"
            ;;
        --publish)
            MODE="publish"
            ;;
        --include-contracts)
            INCLUDE_CONTRACTS=true
            ;;
        *)
            echo "Unknown argument: $arg" >&2
            exit 2
            ;;
    esac
done

if [ -z "$MODE" ]; then
    echo "Usage: $0 --dry-run|--publish [--include-contracts]" >&2
    exit 2
fi

publish_args=()
if [ -n "${CRATES_IO_TOKEN:-}" ]; then
    publish_args+=(--token "$CRATES_IO_TOKEN")
fi

workspace_version=$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' Cargo.toml | head -n 1)
dry_run_patch_args=(
    --config 'patch.crates-io.neo-types.path="rust-devpack/neo-types"'
    --config 'patch.crates-io.neo-syscalls.path="rust-devpack/neo-syscalls"'
    --config 'patch.crates-io.neo-runtime.path="rust-devpack/neo-runtime"'
    --config 'patch.crates-io.neo-macros.path="rust-devpack/neo-macros"'
    --config 'patch.crates-io.neo-devpack.path="rust-devpack"'
    --config 'patch.crates-io.move-neovm.path="move-neovm"'
    --config 'patch.crates-io.neo-solana-compat.path="solana-compat"'
)

if [ "$MODE" = "publish" ] && [ -z "${CRATES_IO_TOKEN:-}${CARGO_REGISTRY_TOKEN:-}" ]; then
    echo "No CRATES_IO_TOKEN/CARGO_REGISTRY_TOKEN detected; cargo may still use saved credentials." >&2
fi

release_crates=(
    "rust-devpack/neo-types"
    "rust-devpack/neo-syscalls"
    "rust-devpack/neo-runtime"
    "rust-devpack/neo-macros"
    "rust-devpack"
    "rust-devpack/neo-test"
    "move-neovm"
    "solana-compat"
    "wasm-neovm"
)

contract_crates=(
    "contracts/hello-world"
    "contracts/nep17-token"
    "contracts/nep11-nft"
    "contracts/constant-product"
    "contracts/uniswap-v2"
    "contracts/staking-rewards"
    "contracts/timelock-vault"
    "contracts/flashloan-pool"
    "contracts/multisig-wallet"
    "contracts/escrow"
    "contracts/crowdfunding"
    "contracts/governance-dao"
    "contracts/oracle-consumer"
    "contracts/nft-marketplace"
    "contracts/storage-smoke"
    "contracts/move-coin"
)

run_for_crate() {
    local dir="$1"
    local manifest="$dir/Cargo.toml"
    local name version
    name=$(sed -n 's/^name[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$manifest" | head -n 1)
    version=$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$manifest" | head -n 1)
    if [ -z "$version" ]; then
        version=$(sed -n 's/^version\.workspace[[:space:]]*=[[:space:]]*true.*/'"$workspace_version"'/p' "$manifest" | head -n 1)
    fi

    echo "==> $MODE $name $version ($manifest)"
    if [ "$MODE" = "dry-run" ]; then
        cargo publish --manifest-path "$manifest" --dry-run --allow-dirty "${publish_args[@]+"${publish_args[@]}"}" "${dry_run_patch_args[@]}"
    else
        cargo publish --manifest-path "$manifest" "${publish_args[@]+"${publish_args[@]}"}"
        sleep 20
    fi
}

for dir in "${release_crates[@]}"; do
    run_for_crate "$dir"
done

if [ "$INCLUDE_CONTRACTS" = true ]; then
    for dir in "${contract_crates[@]}"; do
        run_for_crate "$dir"
    done
fi

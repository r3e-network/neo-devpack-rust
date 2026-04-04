#!/bin/bash
# Version consistency check script for neo-devpack-rust workspace
# Verifies the mixed versioning policy used by this repository:
# - wasm-neovm follows the workspace version
# - devpack, cross-chain, and test crates keep independent versions pinned in workspace.dependencies

set -euo pipefail

package_section() {
    local manifest="$1"
    awk '
        /^\[package\]/ { in_package = 1; next }
        /^\[/ && $0 !~ /^\[package\]/ { in_package = 0 }
        in_package { print }
    ' "$manifest"
}

workspace_dependencies_section() {
    awk '
        /^\[workspace\.dependencies\]/ { in_deps = 1; next }
        /^\[/ && $0 !~ /^\[workspace\.dependencies\]/ { in_deps = 0 }
        in_deps { print }
    ' Cargo.toml
}

workspace_package_section() {
    awk '
        /^\[workspace\.package\]/ { in_package = 1; next }
        /^\[/ && $0 !~ /^\[workspace\.package\]/ { in_package = 0 }
        in_package { print }
    ' Cargo.toml
}

get_package_version() {
    local manifest="$1"
    package_section "$manifest" |
        sed -n 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' |
        head -n 1
}

uses_workspace_version() {
    local manifest="$1"
    package_section "$manifest" |
        sed 's/[[:space:]]//g' |
        grep -Fxq 'version.workspace=true'
}

get_workspace_dependency_version() {
    local package_name="$1"
    workspace_dependencies_section |
        sed -n "s/^[[:space:]]*${package_name}[[:space:]]*=.*version[[:space:]]*=[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" |
        head -n 1
}

get_workspace_version() {
    workspace_package_section |
        sed -n 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' |
        head -n 1
}

WORKSPACE_VERSION=$(get_workspace_version)
WASM_DEP_VERSION=$(get_workspace_dependency_version wasm-neovm)

echo "Workspace version: $WORKSPACE_VERSION"
echo ""
echo "Checking version consistency across workspace crates..."

inconsistent=0

check_workspace_crate() {
    local manifest="$1"
    local display_name="$2"

    if uses_workspace_version "$manifest"; then
        echo "✅ $display_name: uses workspace version ($WORKSPACE_VERSION)"
        return
    fi

    local crate_version
    crate_version=$(get_package_version "$manifest")
    if [ "$crate_version" = "$WORKSPACE_VERSION" ]; then
        echo "✅ $display_name: explicit workspace version ($crate_version)"
    else
        echo "❌ $display_name: expected workspace version $WORKSPACE_VERSION, found $crate_version"
        inconsistent=1
    fi
}

check_pinned_crate() {
    local manifest="$1"
    local display_name="$2"
    local package_name="$3"

    local crate_version expected_version
    crate_version=$(get_package_version "$manifest")
    expected_version=$(get_workspace_dependency_version "$package_name")

    if [ -z "$expected_version" ]; then
        echo "❌ $display_name: missing workspace dependency pin for $package_name"
        inconsistent=1
        return
    fi

    if [ "$crate_version" = "$expected_version" ]; then
        echo "✅ $display_name: $crate_version (matches workspace dependency pin)"
    else
        echo "❌ $display_name: expected $expected_version from workspace dependency pin, found $crate_version"
        inconsistent=1
    fi
}

check_info_crate() {
    local manifest="$1"
    local display_name="$2"

    local crate_version
    crate_version=$(get_package_version "$manifest")
    echo "ℹ️  $display_name: $crate_version (independent/unpublished crate)"
}

if [ "$WASM_DEP_VERSION" = "$WORKSPACE_VERSION" ]; then
    echo "✅ workspace dependency wasm-neovm: $WASM_DEP_VERSION"
else
    echo "❌ workspace dependency wasm-neovm: expected $WORKSPACE_VERSION, found $WASM_DEP_VERSION"
    inconsistent=1
fi

check_workspace_crate "wasm-neovm/Cargo.toml" "wasm-neovm"
check_pinned_crate "move-neovm/Cargo.toml" "move-neovm" "move-neovm"
check_pinned_crate "solana-compat/Cargo.toml" "solana-compat" "neo-solana-compat"
check_info_crate "integration-tests/Cargo.toml" "integration-tests"
check_pinned_crate "rust-devpack/Cargo.toml" "rust-devpack" "neo-devpack"
check_pinned_crate "rust-devpack/neo-types/Cargo.toml" "rust-devpack/neo-types" "neo-types"
check_pinned_crate "rust-devpack/neo-syscalls/Cargo.toml" "rust-devpack/neo-syscalls" "neo-syscalls"
check_pinned_crate "rust-devpack/neo-runtime/Cargo.toml" "rust-devpack/neo-runtime" "neo-runtime"
check_pinned_crate "rust-devpack/neo-macros/Cargo.toml" "rust-devpack/neo-macros" "neo-macros"

if [ "$inconsistent" -eq 1 ]; then
    echo ""
    echo "❌ Version inconsistencies detected."
    exit 1
fi

echo ""
echo "✅ Workspace version metadata is consistent."

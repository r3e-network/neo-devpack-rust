#!/bin/bash
# Version consistency check script for neo-devpack-rust.
#
# Release policy:
# - Every workspace package uses the workspace version, either explicitly or via
#   `version.workspace = true`.
# - Every internal workspace dependency pin in Cargo.toml equals that same
#   workspace version.
# - Contract template crates under contracts/ also use the same package version
#   and pin local internal dependencies to that version.

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

get_manifest_dependency_version() {
    local manifest="$1"
    local package_name="$2"
    sed -n "s/^[[:space:]]*${package_name}[[:space:]]*=.*version[[:space:]]*=[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" "$manifest" |
        head -n 1
}

has_manifest_dependency() {
    local manifest="$1"
    local package_name="$2"
    grep -Eq "^[[:space:]]*${package_name}[[:space:]]*=" "$manifest"
}

get_workspace_version() {
    workspace_package_section |
        sed -n 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' |
        head -n 1
}

WORKSPACE_VERSION=$(get_workspace_version)

echo "Workspace version: $WORKSPACE_VERSION"
echo ""
echo "Checking unified version policy..."

inconsistent=0

fail() {
    echo "❌ $1"
    inconsistent=1
}

pass() {
    echo "✅ $1"
}

check_workspace_crate() {
    local manifest="$1"
    local display_name="$2"

    if uses_workspace_version "$manifest"; then
        pass "$display_name: uses workspace version ($WORKSPACE_VERSION)"
        return
    fi

    local crate_version
    crate_version=$(get_package_version "$manifest")
    if [ "$crate_version" = "$WORKSPACE_VERSION" ]; then
        pass "$display_name: explicit workspace version ($crate_version)"
    else
        fail "$display_name: expected workspace version $WORKSPACE_VERSION, found ${crate_version:-<missing>}"
    fi
}

check_workspace_dependency_pin() {
    local package_name="$1"
    local dep_version
    dep_version=$(get_workspace_dependency_version "$package_name")

    if [ -z "$dep_version" ]; then
        fail "workspace dependency $package_name: missing version pin"
    elif [ "$dep_version" = "$WORKSPACE_VERSION" ]; then
        pass "workspace dependency $package_name: $dep_version"
    else
        fail "workspace dependency $package_name: expected $WORKSPACE_VERSION, found $dep_version"
    fi
}

check_contract_crate() {
    local manifest="$1"
    local display_name
    display_name=${manifest#contracts/}
    display_name=${display_name%/Cargo.toml}

    local crate_version
    crate_version=$(get_package_version "$manifest")
    if [ "$crate_version" = "$WORKSPACE_VERSION" ]; then
        pass "contract $display_name: $crate_version"
    else
        fail "contract $display_name: expected $WORKSPACE_VERSION, found ${crate_version:-<missing>}"
    fi

    local dep dep_version
    for dep in neo-devpack neo-solana-compat; do
        if has_manifest_dependency "$manifest" "$dep"; then
            dep_version=$(get_manifest_dependency_version "$manifest" "$dep")
            if [ "$dep_version" = "$WORKSPACE_VERSION" ]; then
                pass "contract $display_name dependency $dep: $dep_version"
            else
                fail "contract $display_name dependency $dep: expected $WORKSPACE_VERSION, found ${dep_version:-<missing>}"
            fi
        fi
    done
}

for dep in \
    neo-types \
    neo-syscalls \
    neo-runtime \
    neo-macros \
    neo-devpack \
    wasm-neovm \
    move-neovm \
    neo-solana-compat; do
    check_workspace_dependency_pin "$dep"
done

check_workspace_crate "wasm-neovm/Cargo.toml" "wasm-neovm"
check_workspace_crate "move-neovm/Cargo.toml" "move-neovm"
check_workspace_crate "solana-compat/Cargo.toml" "solana-compat"
check_workspace_crate "integration-tests/Cargo.toml" "integration-tests"
check_workspace_crate "rust-devpack/Cargo.toml" "rust-devpack"
check_workspace_crate "rust-devpack/neo-types/Cargo.toml" "rust-devpack/neo-types"
check_workspace_crate "rust-devpack/neo-syscalls/Cargo.toml" "rust-devpack/neo-syscalls"
check_workspace_crate "rust-devpack/neo-runtime/Cargo.toml" "rust-devpack/neo-runtime"
check_workspace_crate "rust-devpack/neo-macros/Cargo.toml" "rust-devpack/neo-macros"
check_workspace_crate "rust-devpack/neo-test/Cargo.toml" "rust-devpack/neo-test"

for manifest in contracts/*/Cargo.toml; do
    check_contract_crate "$manifest"
done

if [ "$inconsistent" -eq 1 ]; then
    echo ""
    echo "❌ Version inconsistencies detected."
    exit 1
fi

echo ""
echo "✅ Unified version metadata is consistent."

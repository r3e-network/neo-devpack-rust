#!/usr/bin/env bash

set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "usage: $0 <Cargo.toml path>" >&2
    exit 2
fi

manifest="$1"
db_path="${CARGO_AUDIT_DB:-$HOME/.cargo/advisory-db}"
deny_args=(cargo deny --manifest-path "$manifest" check -D unmaintained -D notice)

if "${deny_args[@]}"; then
    exit 0
fi

if [ -d "$db_path/.git" ]; then
    echo "cargo deny fetch failed, retrying with local advisory DB: $db_path" >&2
    "${deny_args[@]}" --disable-fetch
    exit 0
fi

exit 1

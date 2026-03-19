#!/usr/bin/env bash

set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "usage: $0 <Cargo.lock path>" >&2
    exit 2
fi

lockfile="$1"
db_path="${CARGO_AUDIT_DB:-$HOME/.cargo/advisory-db}"
audit_args=(cargo audit --file "$lockfile" --deny warnings)

if "${audit_args[@]}"; then
    exit 0
fi

if [ -d "$db_path/.git" ]; then
    echo "cargo audit fetch failed, retrying with local advisory DB: $db_path" >&2
    "${audit_args[@]}" --db "$db_path" --no-fetch --stale
    exit 0
fi

exit 1

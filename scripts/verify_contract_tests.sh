#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

missing=0
checked=0

while IFS= read -r manifest; do
  crate_dir=$(dirname "$manifest")
  checked=$((checked + 1))
  has_test=0

  if [ -f "$crate_dir/src/lib.rs" ] && grep -Eq '^[[:space:]]*#\[(test|neo_test)\]' "$crate_dir/src/lib.rs"; then
    has_test=1
  fi
  if [ "$has_test" -eq 0 ] && [ -f "$crate_dir/src/main.rs" ] && grep -Eq '^[[:space:]]*#\[(test|neo_test)\]' "$crate_dir/src/main.rs"; then
    has_test=1
  fi

  if [ "$has_test" -eq 0 ] && [ -d "$crate_dir/tests" ]; then
    while IFS= read -r -d '' test_file; do
      if grep -Eq '^[[:space:]]*#\[(test|neo_test)\]' "$test_file"; then
        has_test=1
        break
      fi
    done < <(find "$crate_dir/tests" -type f -name '*.rs' -print0)
  fi

  if [ "$has_test" -eq 0 ]; then
    echo "ERROR: no test markers found for contract crate: $crate_dir" >&2
    missing=1
  fi
done < <(find contracts -name Cargo.toml ! -path 'contracts/Cargo.toml' | sort)

if [ "$checked" -eq 0 ]; then
  echo "ERROR: no contract Cargo.toml files found under contracts/" >&2
  exit 1
fi

if [ "$missing" -ne 0 ]; then
  exit 1
fi

echo "Verified test markers for $checked contract crates."

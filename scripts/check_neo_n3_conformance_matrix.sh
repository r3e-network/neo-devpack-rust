#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
MATRIX_FILE="$ROOT_DIR/docs/neo-n3-conformance-matrix.md"

if [[ ! -f "$MATRIX_FILE" ]]; then
  echo "ERROR: conformance matrix not found: $MATRIX_FILE" >&2
  exit 1
fi

violations=$(
  awk -F'|' '
    function trim(s) {
      gsub(/^[ \t]+|[ \t]+$/, "", s)
      return s
    }

    $0 ~ /^\| `System\./ || $0 ~ /^\| `Neo\.Crypto\./ {
      descriptor = trim($2)
      direct_test = trim($5)
      if (direct_test == "No") {
        print descriptor
      }
    }
  ' "$MATRIX_FILE"
)

if [[ -n "$violations" ]]; then
  echo "ERROR: unresolved Neo N3 conformance rows detected (Direct Syscall-Specific Test = No):" >&2
  while IFS= read -r row; do
    [[ -n "$row" ]] && echo "  - $row" >&2
  done <<< "$violations"
  echo "Update tests and set those rows to 'Yes' before merging." >&2
  exit 1
fi

echo "Neo N3 conformance matrix gate passed: all syscall/native rows are marked with direct test coverage."

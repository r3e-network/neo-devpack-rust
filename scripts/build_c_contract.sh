#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  cat >&2 <<'USAGE'
Usage: build_c_contract.sh <contract-dir> [contract-name] [-- clang-flags...] [-- translator-flags...]

Examples:
  scripts/build_c_contract.sh contracts/c-hello
  scripts/build_c_contract.sh contracts/c-hello CExample -- -DWASM_DEBUG -- --source-url https://example.com/c-example

The first optional `--` marks additional clang flags.
The second optional `--` marks additional translator flags.
USAGE
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CONTRACT_DIR="${1%/}"
shift

if [[ ! -d "$CONTRACT_DIR" ]]; then
  echo "error: contract directory '$CONTRACT_DIR' does not exist" >&2
  exit 1
fi

if [[ $# -gt 0 && $1 != --* ]]; then
  CONTRACT_NAME="$1"
  shift
else
  CONTRACT_NAME="$(basename "$CONTRACT_DIR")"
fi

CLANG_FLAGS=()
TRANSLATOR_FLAGS=()

if [[ $# -gt 0 && $1 == -- ]]; then
  shift
  while [[ $# -gt 0 && $1 != -- ]]; do
    CLANG_FLAGS+=("$1")
    shift
  done
fi

if [[ $# -gt 0 && $1 == -- ]]; then
  shift
  while [[ $# -gt 0 ]]; do
    TRANSLATOR_FLAGS+=("$1")
    shift
  done
fi

SOURCE_FILE="${CONTRACT_DIR}/contract.c"
if [[ ! -f "$SOURCE_FILE" ]]; then
  echo "error: expected C source at $SOURCE_FILE" >&2
  exit 1
fi

BUILD_DIR="${CONTRACT_DIR}/build"
mkdir -p "$BUILD_DIR"

BASENAME="${CONTRACT_NAME//-/_}"
WASM_OUT="$BUILD_DIR/${BASENAME}.wasm"
NEF_OUT="$BUILD_DIR/${BASENAME}.nef"
MANIFEST_OUT="$BUILD_DIR/${BASENAME}.manifest.json"

CLANG_BIN="${CLANG:-clang}"
TARGET="${WASM_TARGET:-wasm32-unknown-unknown}"

# Reasonable defaults for freestanding Wasm output. Users can extend/override
# via CLANG_FLAGS if they need additional features.
DEFAULT_CFLAGS=(
  -O3
  -nostdlib
  -fno-builtin
  -ffreestanding
  -fvisibility=hidden
  -fno-exceptions
  -fno-rtti
  -mattr=-simd128,-atomics,-reference-types,-multivalue,-tail-call
)
DEFAULT_LDFLAGS=(
  -Wl,--no-entry
  -Wl,--export-all
)

echo "==> Compiling C contract ($CONTRACT_NAME)"
set -x
"$CLANG_BIN" \
  --target="$TARGET" \
  "${DEFAULT_CFLAGS[@]}" \
  "${CLANG_FLAGS[@]}" \
  "$SOURCE_FILE" \
  -o "$WASM_OUT" \
  "${DEFAULT_LDFLAGS[@]}"
set +x

OVERLAY_PATH="$CONTRACT_DIR/manifest.overlay.json"
TRANSLATOR_ARGS=(
  --input "$WASM_OUT"
  --nef "$NEF_OUT"
  --manifest "$MANIFEST_OUT"
  --name "$CONTRACT_NAME"
)
if [[ -f "$OVERLAY_PATH" ]]; then
  TRANSLATOR_ARGS+=(--manifest-overlay "$OVERLAY_PATH")
fi
TRANSLATOR_ARGS+=("${TRANSLATOR_FLAGS[@]}")

echo "==> Translating Wasm to NeoVM"
cargo run --manifest-path "$REPO_ROOT/wasm-neovm/Cargo.toml" -- \
  "${TRANSLATOR_ARGS[@]}"

echo "==> Outputs"
echo "Wasm:      $WASM_OUT"
echo "NEF:       $NEF_OUT"
echo "Manifest:  $MANIFEST_OUT"

#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <contract-crate-path> [contract-name] [translator-flags...]" >&2
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CONTRACT_DIR="${1%/}"

shift

if [[ $# -gt 0 && $1 != --* ]]; then
  CONTRACT_NAME="$1"
  shift
else
  CONTRACT_NAME="$(basename "$CONTRACT_DIR")"
fi

TRANSLATOR_ARGS=("$@")

has_overlay_flag=false
for arg in "${TRANSLATOR_ARGS[@]}"; do
  if [[ "$arg" == --manifest-overlay || "$arg" == --manifest-overlay=* ]]; then
    has_overlay_flag=true
    break
  fi
done

if [[ "$has_overlay_flag" == false ]]; then
  DEFAULT_OVERLAY="$CONTRACT_DIR/manifest.overlay.json"
  if [[ -f "$DEFAULT_OVERLAY" ]]; then
    TRANSLATOR_ARGS+=("--manifest-overlay" "$DEFAULT_OVERLAY")
  fi
fi

if [[ ! -f "$CONTRACT_DIR/Cargo.toml" ]]; then
  echo "error: $CONTRACT_DIR does not contain a Cargo.toml" >&2
  exit 1
fi

echo "==> Building Wasm contract ($CONTRACT_NAME)"
RUSTFLAGS_TO_USE="${NEO_WASM_RUSTFLAGS:--C opt-level=z -C strip=symbols}"
echo "    RUSTFLAGS=$RUSTFLAGS_TO_USE"
RUSTFLAGS="$RUSTFLAGS_TO_USE" cargo build --manifest-path "$CONTRACT_DIR/Cargo.toml" \
  --target wasm32-unknown-unknown \
  --release

BASENAME="${CONTRACT_NAME//-/_}"
WASM_PATH="$CONTRACT_DIR/target/wasm32-unknown-unknown/release/${BASENAME}.wasm"
if [[ ! -f "$WASM_PATH" ]]; then
  echo "error: expected Wasm artefact at $WASM_PATH" >&2
  exit 1
fi

NEF_OUT="${WASM_PATH%.wasm}.nef"
MANIFEST_OUT="${NEF_OUT%.nef}.manifest.json"

echo "==> Translating Wasm to NeoVM"
cargo run --manifest-path "$REPO_ROOT/wasm-neovm/Cargo.toml" -- \
  --input "$WASM_PATH" \
  --nef "$NEF_OUT" \
  --manifest "$MANIFEST_OUT" \
  --name "$CONTRACT_NAME" \
  "${TRANSLATOR_ARGS[@]}"

echo "==> Outputs"
echo "NEF:        $NEF_OUT"
echo "Manifest:   $MANIFEST_OUT"

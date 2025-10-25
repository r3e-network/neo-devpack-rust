#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 3 ]; then
  echo "Usage: $0 <nef-path> <manifest-path> <contract-name> [--account <script-hash>]" >&2
  exit 1
fi

NEF_PATH=$1
MANIFEST_PATH=$2
CONTRACT_NAME=$3
shift 3

ACCOUNT_FLAG=""
if [ $# -gt 0 ]; then
  ACCOUNT_FLAG="$*"
fi

if [ ! -f "$NEF_PATH" ]; then
  echo "NEF file not found: $NEF_PATH" >&2
  exit 1
fi

if [ ! -f "$MANIFEST_PATH" ]; then
  echo "Manifest file not found: $MANIFEST_PATH" >&2
  exit 1
fi

NEO_EXPRESS_CLI=${NEO_EXPRESS_CLI:-neo-express}
NEO_EXPRESS_RPC=${NEO_EXPRESS_RPC:-http://localhost:50012}

if ! command -v "$NEO_EXPRESS_CLI" >/dev/null 2>&1; then
  echo "neo-express CLI not found. Set NEO_EXPRESS_CLI to its path." >&2
  exit 1
fi

echo "Deploying $CONTRACT_NAME via $NEO_EXPRESS_CLI (RPC: $NEO_EXPRESS_RPC)" >&2

"$NEO_EXPRESS_CLI" contract deploy \
  --rpc "$NEO_EXPRESS_RPC" \
  --nef "$NEF_PATH" \
  --manifest "$MANIFEST_PATH" \
  --force \
  $ACCOUNT_FLAG

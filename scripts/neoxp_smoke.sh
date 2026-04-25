#!/usr/bin/env bash
set -euo pipefail

require_command() {
  local command_name="$1"
  local guidance="$2"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "ERROR: required command '$command_name' not found." >&2
    echo "$guidance" >&2
    exit 1
  fi
}

NEOXP_BIN=${NEOXP_BIN:-/tmp/neo-tools/neoxp}
NEOXP_TIMEOUT=${NEOXP_TIMEOUT:-60s}
WASM_SNIP_BIN=${WASM_SNIP:-wasm-snip}
ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

BUILD_TARGETS=(
  hello-world
  nep17-token
  nep11-nft
  constant-product
  uniswap-v2
  staking-rewards
  timelock-vault
  flashloan-pool
  multisig-wallet
  escrow
  crowdfunding
  governance-dao
  oracle-consumer
  nft-marketplace
  solana-hello
  move-coin
  storage-smoke
)

CONTRACT_NEF_PATHS=(
  build/HelloWorld.nef
  build/NEP17.nef
  build/NEP11.nef
  build/AMM.nef
  build/UniswapV2.nef
  build/StakingRewards.nef
  build/TimelockVault.nef
  build/FlashLoanPool.nef
  build/MultisigWallet.nef
  build/Escrow.nef
  build/Crowdfunding.nef
  build/GovernanceDAO.nef
  build/OracleConsumer.nef
  build/NFTMarketplace.nef
  build/solana_hello.nef
  build/MoveCoin.nef
  build/StorageSmoke.nef
)

CONTRACT_DEPLOY_NAMES=(
  HelloWorld
  SampleNEP17
  SampleNEP11
  ConstantProductAMM
  UniswapV2Router
  StakingRewards
  TimelockVault
  FlashLoanPool
  SampleMultisig
  NeoEscrow
  NeoCrowdfund
  NeoGovernanceDAO
  NeoOracleConsumer
  NeoNFTMarketplace
  solana-hello
  MoveCoin
  StorageSmoke
)

if [ "${#CONTRACT_NEF_PATHS[@]}" -ne "${#CONTRACT_DEPLOY_NAMES[@]}" ]; then
  echo "ERROR: CONTRACT_NEF_PATHS and CONTRACT_DEPLOY_NAMES length mismatch." >&2
  exit 1
fi

require_command jq "Install jq and retry."
require_command timeout "Install GNU coreutils timeout and retry."
require_command "$WASM_SNIP_BIN" "Install wasm-snip (cargo install wasm-snip --locked) or set WASM_SNIP."

if [ ! -x "$NEOXP_BIN" ]; then
  echo "ERROR: Neo Express binary not found at $NEOXP_BIN" >&2
  echo "Set NEOXP_BIN to the neoxp executable path." >&2
  exit 1
fi

cd "$ROOT_DIR"

echo "[smoke] rebuilding target contracts"
make -B "${BUILD_TARGETS[@]}" >/dev/null

echo "[smoke] generated NEF sizes"
for nef in "${CONTRACT_NEF_PATHS[@]}"; do
  if [ ! -f "$nef" ]; then
    echo "ERROR: expected translated artifact not found: $nef" >&2
    exit 1
  fi
  printf "  %9d  %s\n" "$(wc -c < "$nef")" "$nef"
done

CHAIN_DIR=$(mktemp -d)
CHAIN="$CHAIN_DIR/default.neo-express"

cleanup() {
  rm -rf "$CHAIN_DIR"
}
trap cleanup EXIT

neoxp() {
  timeout "$NEOXP_TIMEOUT" "$NEOXP_BIN" "$@"
}

neoxp create -o "$CHAIN" -f >/dev/null

deploy_contract() {
  local nef_path="$1"
  local expected_name="$2"
  local out

  out=$(neoxp contract deploy -i "$CHAIN" -j -f "$nef_path" genesis)
  local deployed_name
  deployed_name=$(echo "$out" | jq -r '.["contract-name"]')
  local deployed_hash
  deployed_hash=$(echo "$out" | jq -r '.["contract-hash"]')

  if [ "$deployed_name" != "$expected_name" ]; then
    echo "ERROR: expected contract name '$expected_name' but got '$deployed_name'" >&2
    echo "$out" >&2
    exit 1
  fi

  echo "  deployed $deployed_name at $deployed_hash"
}

run_expect() {
  local contract="$1"
  local method="$2"
  local expected_value="$3"
  shift 3

  local out
  out=$(neoxp contract run -i "$CHAIN" -r -j "$contract" "$method" "$@")

  local state
  state=$(echo "$out" | jq -r '.state')
  if [ "$state" != "HALT" ]; then
    echo "ERROR: $contract.$method did not HALT" >&2
    echo "$out" >&2
    exit 1
  fi

  local actual_value
  actual_value=$(echo "$out" | jq -r '.stack[0].value // ""')
  if [ "$actual_value" != "$expected_value" ]; then
    echo "ERROR: $contract.$method expected '$expected_value' but got '$actual_value'" >&2
    echo "$out" >&2
    exit 1
  fi

  echo "  ok $contract.$method => $actual_value"
}

run_expect_int_min() {
  local contract="$1"
  local method="$2"
  local min_value="$3"
  shift 3

  local out
  out=$(neoxp contract run -i "$CHAIN" -r -j "$contract" "$method" "$@")

  local state
  state=$(echo "$out" | jq -r '.state')
  if [ "$state" != "HALT" ]; then
    echo "ERROR: $contract.$method did not HALT" >&2
    echo "$out" >&2
    exit 1
  fi

  local actual_value
  actual_value=$(echo "$out" | jq -r '.stack[0].value // ""')
  if ! [[ "$actual_value" =~ ^-?[0-9]+$ ]]; then
    echo "ERROR: $contract.$method returned non-integer value '$actual_value'" >&2
    echo "$out" >&2
    exit 1
  fi

  if (( actual_value < min_value )); then
    echo "ERROR: $contract.$method expected >= $min_value but got $actual_value" >&2
    echo "$out" >&2
    exit 1
  fi

  echo "  ok $contract.$method => $actual_value (>= $min_value)"
}

echo "[smoke] deploying contracts"
for i in "${!CONTRACT_NEF_PATHS[@]}"; do
  deploy_contract "${CONTRACT_NEF_PATHS[$i]}" "${CONTRACT_DEPLOY_NAMES[$i]}"
done

echo "[smoke] HelloWorld invoke"
run_expect HelloWorld hello 42

echo "[smoke] NEP-17 invoke set"
run_expect SampleNEP17 totalSupply 1000000
run_expect SampleNEP17 balanceOf 750000 1
run_expect SampleNEP17 transfer 1 1 2 250

echo "[smoke] NEP-11 invoke set"
run_expect SampleNEP11 totalSupply 1000
run_expect SampleNEP11 mint 1 1 10001
run_expect SampleNEP11 ownerOf 2 10001
run_expect SampleNEP11 transfer 1 1 2 10001

echo "[smoke] AMM invoke set"
run_expect ConstantProductAMM init 1 10000 5000
run_expect ConstantProductAMM getReserves 42949672965000
run_expect ConstantProductAMM quote 49 100
run_expect ConstantProductAMM swap 49 1 100

echo "[smoke] Uniswap invoke set"
run_expect UniswapV2Router getReserves 4294967296500000
run_expect UniswapV2Router addLiquidity 1 100 50
run_expect UniswapV2Router quote 498 1000
run_expect UniswapV2Router swapExactTokensForTokens 498 1000 498

echo "[smoke] Staking invoke set"
# previewReward(amount, days_staked) — amount=10000, days=365 stays within
# MAX_DAYS=3650 and MAX_PREVIEW_AMOUNT=1e12. Earlier the args were swapped
# (365, 10000) and the symmetric reward formula `amount*days*APR/(BPS*Y)`
# happened to produce the same number, masking a translator bug where args
# to internal wasm calls were reversed.
run_expect StakingRewards previewReward 1200 10000 365

echo "[smoke] Timelock invoke set"
run_expect TimelockVault isMature 1 10 10

echo "[smoke] Flashloan invoke set"
run_expect FlashLoanPool maxFlashLoan 1000000
run_expect FlashLoanPool flashFee 9 10000
run_expect FlashLoanPool flashLoan 9 1 10000
run_expect FlashLoanPool repay 1 10000 10009

echo "[smoke] Storage round-trip"
# Real persistent storage: setValue commits a writeable transaction (no -r),
# then getValue reads back the same i64 from chain state. This exercises
# `System.Storage.GetContext + Put + Get` end-to-end on Neo Express, proving
# the wasm-side storage facade reaches actual contract storage rather than
# the previous in-process simulation `Vec`.
neoxp contract run -i "$CHAIN" StorageSmoke setValue 31415 -a genesis >/dev/null
echo "  ok StorageSmoke.setValue committed"
run_expect StorageSmoke getValue 31415

echo "[smoke] Multisig invoke set"
echo "  deploy-only SampleMultisig (stateful witness paths covered by Rust tests)"

echo "[smoke] Escrow invoke set"
echo "  deploy-only NeoEscrow (stateful paths covered by Rust tests)"

echo "[smoke] Crowdfunding invoke set"
echo "  deploy-only NeoCrowdfund (stateful witness paths covered by Rust tests)"

echo "[smoke] Governance invoke set"
echo "  deploy-only NeoGovernanceDAO (stateful paths covered by Rust tests)"

echo "[smoke] Oracle invoke set"
echo "  deploy-only NeoOracleConsumer (stateful witness paths covered by Rust tests)"

echo "[smoke] NFT marketplace invoke set"
run_expect NeoNFTMarketplace onNEP11Payment 1 1 1 10001 0
run_expect NeoNFTMarketplace onNEP17Payment 1 1 100 0

echo "[smoke] Cross-chain invoke set"
echo "  deploy-only solana-hello.main (Solana ABI payload requires linear-memory adapter)"
run_expect_int_min solana-hello get_time 1
run_expect MoveCoin total_supply 1000000
run_expect MoveCoin has_coin 1 1
run_expect MoveCoin mint 1 1 10
run_expect MoveCoin transfer 1 1 2 5
run_expect MoveCoin burn 1 1 5
run_expect MoveCoin balance 1000 1

echo "[smoke] all deploy/invoke checks passed"

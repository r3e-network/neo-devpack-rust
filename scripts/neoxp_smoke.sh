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
)

if [ "${#CONTRACT_NEF_PATHS[@]}" -ne "${#CONTRACT_DEPLOY_NAMES[@]}" ]; then
  echo "ERROR: CONTRACT_NEF_PATHS and CONTRACT_DEPLOY_NAMES length mismatch." >&2
  exit 1
fi

require_command jq "Install jq and retry."
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

"$NEOXP_BIN" create -o "$CHAIN" -f >/dev/null

deploy_contract() {
  local nef_path="$1"
  local expected_name="$2"
  local out

  out=$("$NEOXP_BIN" contract deploy -i "$CHAIN" -j -f "$nef_path" genesis)
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
  out=$("$NEOXP_BIN" contract run -i "$CHAIN" -r -j "$contract" "$method" "$@")

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
  out=$("$NEOXP_BIN" contract run -i "$CHAIN" -r -j "$contract" "$method" "$@")

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
run_expect UniswapV2Router addLiquidity 1 5000 2500
run_expect UniswapV2Router getReserves 4294967296500000
run_expect UniswapV2Router quote 498 1000
run_expect UniswapV2Router swapExactTokensForTokens 498 1000 400

echo "[smoke] Staking invoke set"
run_expect StakingRewards stake 1 1 100000
run_expect StakingRewards previewReward 986 100000 30
run_expect StakingRewards claim 986 1 100000 30

echo "[smoke] Timelock invoke set"
run_expect TimelockVault queueRelease 1 1 100000 100
run_expect TimelockVault isMature 0 100 90
run_expect TimelockVault release 1 1 100000 100 110

echo "[smoke] Flashloan invoke set"
run_expect FlashLoanPool maxFlashLoan 1000000
run_expect FlashLoanPool flashFee 900 1000000
run_expect FlashLoanPool flashLoan 90 1 100000
run_expect FlashLoanPool repay 1 100000 100090

echo "[smoke] Multisig invoke set"
run_expect SampleMultisig configure 1 11 3 2
run_expect SampleMultisig propose 1 11 22 33 44 55 66 77 88
run_expect SampleMultisig approve 1 11 1 22
run_expect SampleMultisig execute 1 11 1 22

echo "[smoke] Escrow invoke set"
run_expect NeoEscrow configure 1 11 22 33 44 55 66 77 88 99
run_expect NeoEscrow release 1 11 22
run_expect NeoEscrow refund 1 11 22

echo "[smoke] Crowdfunding invoke set"
run_expect NeoCrowdfund configure 1 11 22 33 44 55 66
run_expect NeoCrowdfund contributionOf 100 11 22
run_expect NeoCrowdfund finalize 1 11
run_expect NeoCrowdfund claimRefund 1 11 22

echo "[smoke] Governance invoke set"
run_expect NeoGovernanceDAO configure 1 11 22 33 44 55
run_expect NeoGovernanceDAO stakeOf 1000 11 22
run_expect NeoGovernanceDAO vote 1 11 1 22 1 5
run_expect NeoGovernanceDAO execute 1 11
run_expect NeoGovernanceDAO unstake 1 11 22 5

echo "[smoke] Oracle invoke set"
run_expect NeoOracleConsumer lastRequestId 1
run_expect NeoOracleConsumer configure 1 11 20 22 20
run_expect NeoOracleConsumer request 1 11 3 22 3 33 3
run_expect NeoOracleConsumer onOracleResponse 1 1 200 44 4

echo "[smoke] NFT marketplace invoke set"
run_expect NeoNFTMarketplace createListing 1 11 22 33 44 55 66 77 88 99
run_expect NeoNFTMarketplace cancelListing 1 11 22 33
run_expect NeoNFTMarketplace onNEP11Payment 1 11 22 33 44
run_expect NeoNFTMarketplace onNEP17Payment 1 11 22 33

echo "[smoke] Cross-chain invoke set"
run_expect solana-hello main 0 1 2
run_expect_int_min solana-hello get_time 1
run_expect MoveCoin total_supply 1000000
run_expect MoveCoin has_coin 1 1
run_expect MoveCoin mint 1 1 10
run_expect MoveCoin transfer 1 1 2 5
run_expect MoveCoin burn 1 1 5
run_expect MoveCoin balance 1000 1

echo "[smoke] all deploy/invoke checks passed"

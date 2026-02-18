#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

GENERATIONS=100
FULL_INTERVAL=25
REPORT_PATH="$ROOT_DIR/build/neo_n3_iteration_report.txt"
REFERENCE_MODE="auto"

NEO_SYSCALL_ROOT="$ROOT_DIR/neo/src/Neo/SmartContract"
NEO_OPCODE_PATH="$ROOT_DIR/neo-vm/src/Neo.VM/OpCode.cs"

usage() {
  cat <<'USAGE'
Usage: scripts/iterate_neo_n3_quality.sh [options]

Options:
  --generations <n>    Number of iterations to run (default: 100)
  --full-interval <n>  Run heavier full checks every n generations (default: 25, 0 disables)
  --report <path>      Report output path (default: build/neo_n3_iteration_report.txt)
  --reference-mode <m> Neo reference strictness: auto|always|never (default: auto)
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --generations)
      GENERATIONS="$2"
      shift 2
      ;;
    --full-interval)
      FULL_INTERVAL="$2"
      shift 2
      ;;
    --report)
      REPORT_PATH="$2"
      shift 2
      ;;
    --reference-mode)
      REFERENCE_MODE="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! [[ "$GENERATIONS" =~ ^[0-9]+$ ]] || [[ "$GENERATIONS" -lt 1 ]]; then
  echo "ERROR: --generations must be a positive integer" >&2
  exit 1
fi

if ! [[ "$FULL_INTERVAL" =~ ^[0-9]+$ ]]; then
  echo "ERROR: --full-interval must be a non-negative integer" >&2
  exit 1
fi

if ! [[ "$REFERENCE_MODE" =~ ^(auto|always|never)$ ]]; then
  echo "ERROR: --reference-mode must be one of: auto, always, never" >&2
  exit 1
fi

reference_available=false
if [[ -d "$NEO_SYSCALL_ROOT" && -f "$NEO_OPCODE_PATH" ]]; then
  reference_available=true
fi

strict_reference=false
if [[ "$REFERENCE_MODE" == "always" ]]; then
  strict_reference=true
elif [[ "$REFERENCE_MODE" == "auto" && "$reference_available" == true ]]; then
  strict_reference=true
fi

if [[ "$strict_reference" == true ]]; then
  export WASM_NEOVM_REQUIRE_NEO_CHECKOUT=1
else
  unset WASM_NEOVM_REQUIRE_NEO_CHECKOUT || true
fi

mkdir -p "$(dirname "$REPORT_PATH")"

{
  echo "# Neo N3 Iteration Report"
  echo "date=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "generations=${GENERATIONS}"
  echo "full_interval=${FULL_INTERVAL}"
  echo "reference_mode=${REFERENCE_MODE}"
  echo "reference_available=${reference_available}"
  echo "strict_reference=${strict_reference}"
  echo
} > "$REPORT_PATH"

run_check() {
  local label="$1"
  shift
  echo "  -> ${label}: $*" >> "$REPORT_PATH"
  "$@" >> "$REPORT_PATH" 2>&1
}

start_epoch=$(date +%s)
passed=0
full_checks=0
failed_generation=0
failed_step=""

echo "Starting ${GENERATIONS} Neo N3 quality generations..."

for ((gen = 1; gen <= GENERATIONS; gen++)); do
  gen_start=$(date +%s)
  echo "GEN ${gen}/${GENERATIONS}" | tee -a "$REPORT_PATH"

  if ! run_check "direct-syscall-coverage" cargo test -p wasm-neovm --test neo_n3_direct_syscall_coverage --quiet; then
    failed_generation="$gen"
    failed_step="direct-syscall-coverage"
    break
  fi

  if ! run_check "manifest-parity" cargo test -p wasm-neovm --test manifest_parity --quiet; then
    failed_generation="$gen"
    failed_step="manifest-parity"
    break
  fi

  if ! run_check "on-nep17-adapter-coverage" cargo test -p wasm-neovm --test on_nep17_adapter --quiet; then
    failed_generation="$gen"
    failed_step="on-nep17-adapter-coverage"
    break
  fi

  if ! run_check "syscall-parity" cargo test -p integration-tests --test syscall_parity --quiet; then
    failed_generation="$gen"
    failed_step="syscall-parity"
    break
  fi

  if ! run_check "devpack-runtime-syscalls" cargo test -p neo-devpack --test neo_syscalls_tests --test neo_runtime_tests --test comprehensive_test_suite --quiet; then
    failed_generation="$gen"
    failed_step="devpack-runtime-syscalls"
    break
  fi

  if [[ "$FULL_INTERVAL" -gt 0 ]] && (( gen % FULL_INTERVAL == 0 )); then
    full_checks=$((full_checks + 1))

    if ! run_check "full-wasm-neovm-suite" cargo test -p wasm-neovm --quiet; then
      failed_generation="$gen"
      failed_step="full-wasm-neovm-suite"
      break
    fi

    if ! run_check "full-neo-devpack-suite" cargo test -p neo-devpack --quiet; then
      failed_generation="$gen"
      failed_step="full-neo-devpack-suite"
      break
    fi

    if ! run_check "full-integration-suite" cargo test -p integration-tests --quiet; then
      failed_generation="$gen"
      failed_step="full-integration-suite"
      break
    fi

    if ! run_check "full-workspace-suite" cargo test --workspace --quiet; then
      failed_generation="$gen"
      failed_step="full-workspace-suite"
      break
    fi

    if ! run_check "strict-clippy" cargo clippy -p wasm-neovm -p neo-devpack -p integration-tests --all-targets -- -D warnings; then
      failed_generation="$gen"
      failed_step="strict-clippy"
      break
    fi

    if ! run_check "conformance-matrix-gate" bash scripts/check_neo_n3_conformance_matrix.sh; then
      failed_generation="$gen"
      failed_step="conformance-matrix-gate"
      break
    fi
  fi

  gen_elapsed=$(( $(date +%s) - gen_start ))
  echo "  status=PASS duration=${gen_elapsed}s" | tee -a "$REPORT_PATH"
  passed=$((passed + 1))
done

total_elapsed=$(( $(date +%s) - start_epoch ))

{
  echo
  echo "summary:"
  echo "  passed_generations=${passed}"
  echo "  requested_generations=${GENERATIONS}"
  echo "  full_checks_executed=${full_checks}"
  echo "  elapsed_seconds=${total_elapsed}"
} >> "$REPORT_PATH"

if [[ "$failed_generation" -ne 0 ]]; then
  {
    echo "  failed_generation=${failed_generation}"
    echo "  failed_step=${failed_step}"
  } >> "$REPORT_PATH"
  echo "FAILED at generation ${failed_generation} (${failed_step}). See ${REPORT_PATH}" >&2
  exit 1
fi

echo "Completed ${passed}/${GENERATIONS} generations. Report: ${REPORT_PATH}"

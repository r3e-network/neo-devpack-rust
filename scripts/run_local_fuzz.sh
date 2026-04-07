#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
WASM_DIR="$ROOT_DIR/wasm-neovm"
FUZZ_DIR="$WASM_DIR/fuzz"
LOG_ROOT="$ROOT_DIR/build/fuzz-local"
MODE="parallel"
MAX_TOTAL_TIME=0
RSS_LIMIT_MB=0
TIMEOUT_SECONDS=30
MAX_LEN=""
BUILD_FIRST=1

KNOWN_TARGETS=(
  fuzz_translate
  fuzz_translate_config
  fuzz_nef
  fuzz_numeric
  fuzz_structured_pipeline
  fuzz_devpack_codec
  fuzz_syscall_surface
  fuzz_rust_contract
  fuzz_rust_contract_differential
)

TARGETS=("${KNOWN_TARGETS[@]}")
declare -a PIDS=()
declare -A PID_TO_TARGET=()
SESSION_DIR=""

usage() {
  cat <<'USAGE'
Usage: scripts/run_local_fuzz.sh [options]

Starts local cargo-fuzz runs for wasm-neovm and related devpack surfaces.
Default mode is parallel and unbounded: all fuzz targets keep running until interrupted.

Options:
  --mode <parallel|sequential>  Execution mode (default: parallel)
  --targets <csv>               Comma-separated target list (default: all targets)
  --max-total-time <seconds>    Per-target libFuzzer time budget; 0 means no limit (default: 0)
  --rss-limit-mb <mb>           Pass -rss_limit_mb to libFuzzer; 0 keeps libFuzzer default
  --timeout <seconds>           Pass -timeout to libFuzzer (default: 30)
  --max-len <bytes>             Pass -max_len to libFuzzer
  --log-root <path>             Session log root (default: build/fuzz-local)
  --no-build                    Skip cargo-fuzz build step
  -h, --help                    Show this help

Examples:
  scripts/run_local_fuzz.sh
  scripts/run_local_fuzz.sh --targets fuzz_structured_pipeline,fuzz_devpack_codec
  scripts/run_local_fuzz.sh --targets fuzz_rust_contract,fuzz_rust_contract_differential --timeout 120
  scripts/run_local_fuzz.sh --max-total-time 86400
USAGE
}

require_command() {
  local command_name="$1"
  local guidance="$2"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "ERROR: required command '$command_name' not found." >&2
    echo "$guidance" >&2
    exit 1
  fi
}

join_by() {
  local delimiter="$1"
  shift
  local first=1
  for value in "$@"; do
    if [[ "$first" -eq 1 ]]; then
      printf "%s" "$value"
      first=0
    else
      printf "%s%s" "$delimiter" "$value"
    fi
  done
}

cleanup() {
  local status="${1:-$?}"
  trap - EXIT INT TERM
  if [[ ${#PIDS[@]} -gt 0 ]]; then
    for pid in "${PIDS[@]}"; do
      kill "$pid" 2>/dev/null || true
    done
    for pid in "${PIDS[@]}"; do
      wait "$pid" 2>/dev/null || true
    done
  fi
  exit "$status"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode)
      MODE="$2"
      shift 2
      ;;
    --targets)
      IFS=',' read -r -a TARGETS <<< "$2"
      shift 2
      ;;
    --max-total-time)
      MAX_TOTAL_TIME="$2"
      shift 2
      ;;
    --rss-limit-mb)
      RSS_LIMIT_MB="$2"
      shift 2
      ;;
    --timeout)
      TIMEOUT_SECONDS="$2"
      shift 2
      ;;
    --max-len)
      MAX_LEN="$2"
      shift 2
      ;;
    --log-root)
      LOG_ROOT="$2"
      shift 2
      ;;
    --no-build)
      BUILD_FIRST=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: unknown option '$1'" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! [[ "$MODE" =~ ^(parallel|sequential)$ ]]; then
  echo "ERROR: --mode must be 'parallel' or 'sequential'" >&2
  exit 1
fi

if ! [[ "$MAX_TOTAL_TIME" =~ ^[0-9]+$ ]]; then
  echo "ERROR: --max-total-time must be a non-negative integer" >&2
  exit 1
fi

if ! [[ "$RSS_LIMIT_MB" =~ ^[0-9]+$ ]]; then
  echo "ERROR: --rss-limit-mb must be a non-negative integer" >&2
  exit 1
fi

if ! [[ "$TIMEOUT_SECONDS" =~ ^[0-9]+$ ]] || [[ "$TIMEOUT_SECONDS" -lt 1 ]]; then
  echo "ERROR: --timeout must be a positive integer" >&2
  exit 1
fi

if [[ -n "$MAX_LEN" ]] && ! [[ "$MAX_LEN" =~ ^[0-9]+$ ]]; then
  echo "ERROR: --max-len must be a non-negative integer" >&2
  exit 1
fi

if [[ ${#TARGETS[@]} -eq 0 ]]; then
  echo "ERROR: at least one target is required" >&2
  exit 1
fi

for target in "${TARGETS[@]}"; do
  known=false
  for candidate in "${KNOWN_TARGETS[@]}"; do
    if [[ "$candidate" == "$target" ]]; then
      known=true
      break
    fi
  done
  if [[ "$known" == false ]]; then
    echo "ERROR: unknown fuzz target '$target'" >&2
    exit 1
  fi
done

export PATH="$HOME/.cargo/bin:$PATH"
require_command cargo "Install Rust cargo and retry."
require_command rustc "Install Rust toolchain and retry."
require_command cargo-fuzz "Install cargo-fuzz with: cargo install cargo-fuzz --locked"

HOST_TRIPLE=$(rustc +nightly -vV | sed -n 's/^host: //p')
if [[ -z "$HOST_TRIPLE" ]]; then
  echo "ERROR: failed to determine nightly host triple" >&2
  exit 1
fi

SESSION_DIR="$LOG_ROOT/$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p "$SESSION_DIR"
mkdir -p "$LOG_ROOT"
ln -sfn "$SESSION_DIR" "$LOG_ROOT/latest"

{
  echo "started_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "runner_pid=$$"
  echo "mode=${MODE}"
  echo "host=${HOST_TRIPLE}"
  echo "max_total_time=${MAX_TOTAL_TIME}"
  echo "rss_limit_mb=${RSS_LIMIT_MB}"
  echo "timeout=${TIMEOUT_SECONDS}"
  echo "max_len=${MAX_LEN:-default}"
  echo "targets=$(join_by , "${TARGETS[@]}")"
} > "$SESSION_DIR/session.txt"

build_target() {
  local target="$1"
  echo "[build] $target"
  (
    cd "$WASM_DIR"
    cargo +nightly fuzz build "$target"
  ) >> "$SESSION_DIR/build.log" 2>&1
}

run_parallel_target() {
  local target="$1"
  local corpus_dir="$FUZZ_DIR/corpus/$target"
  local artifact_dir="$FUZZ_DIR/artifacts/$target"
  local binary_path="$FUZZ_DIR/target/$HOST_TRIPLE/release/$target"
  local log_path="$SESSION_DIR/${target}.log"
  mkdir -p "$corpus_dir" "$artifact_dir"

  if [[ ! -x "$binary_path" ]]; then
    echo "ERROR: fuzz binary missing: $binary_path" >&2
    exit 1
  fi

  local cmd=("$binary_path" "-artifact_prefix=${artifact_dir}/")
  if [[ "$MAX_TOTAL_TIME" -gt 0 ]]; then
    cmd+=("-max_total_time=$MAX_TOTAL_TIME")
  fi
  if [[ "$RSS_LIMIT_MB" -gt 0 ]]; then
    cmd+=("-rss_limit_mb=$RSS_LIMIT_MB")
  fi
  cmd+=("-timeout=$TIMEOUT_SECONDS")
  if [[ -n "$MAX_LEN" ]]; then
    cmd+=("-max_len=$MAX_LEN")
  fi
  cmd+=("$corpus_dir")

  {
    echo "# started_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "# target=$target"
    printf "# command="
    printf "%q " "${cmd[@]}"
    echo
  } > "$log_path"

  (
    cd "$WASM_DIR"
    exec stdbuf -oL -eL "${cmd[@]}"
  ) >> "$log_path" 2>&1 &

  local pid=$!
  PIDS+=("$pid")
  PID_TO_TARGET["$pid"]="$target"
  echo "$pid" > "$SESSION_DIR/${target}.pid"
  echo "[run] $target pid=$pid log=$log_path"
}

run_sequential_target() {
  local target="$1"
  local corpus_dir="$FUZZ_DIR/corpus/$target"
  local log_path="$SESSION_DIR/${target}.log"
  local cmd=(cargo +nightly fuzz run "$target" -- "-max_total_time=$MAX_TOTAL_TIME" "-timeout=$TIMEOUT_SECONDS")
  mkdir -p "$corpus_dir"
  if [[ -n "$MAX_LEN" ]]; then
    cmd+=("-max_len=$MAX_LEN")
  fi
  cmd+=("$corpus_dir")
  echo "[run] $target log=$log_path"
  (
    cd "$WASM_DIR"
    "${cmd[@]}"
  ) >> "$log_path" 2>&1
}

wait_for_parallel_targets() {
  declare -A RUNNING=()
  local pid
  local other_pid
  for pid in "${PIDS[@]}"; do
    RUNNING["$pid"]=1
  done

  while [[ ${#RUNNING[@]} -gt 0 ]]; do
    for pid in "${!RUNNING[@]}"; do
      if kill -0 "$pid" 2>/dev/null; then
        continue
      fi

      if wait "$pid"; then
        unset "RUNNING[$pid]"
        continue
      fi

      echo "ERROR: ${PID_TO_TARGET[$pid]} exited non-zero. See $SESSION_DIR/${PID_TO_TARGET[$pid]}.log" >&2
      unset "RUNNING[$pid]"
      for other_pid in "${!RUNNING[@]}"; do
        kill "$other_pid" 2>/dev/null || true
      done
      for other_pid in "${!RUNNING[@]}"; do
        wait "$other_pid" 2>/dev/null || true
      done
      return 1
    done
    sleep 2
  done

  return 0
}

trap 'cleanup $?' EXIT INT TERM

echo "session_dir=$SESSION_DIR"
echo "targets=$(join_by ' ' "${TARGETS[@]}")"

if [[ "$BUILD_FIRST" -eq 1 ]]; then
  : > "$SESSION_DIR/build.log"
  for target in "${TARGETS[@]}"; do
    build_target "$target"
  done
fi

if [[ "$MODE" == "parallel" ]]; then
  for target in "${TARGETS[@]}"; do
    run_parallel_target "$target"
  done

  status=0
  if ! wait_for_parallel_targets; then
    status=1
  fi
  trap - INT TERM EXIT
  exit "$status"
fi

if [[ "$MAX_TOTAL_TIME" -eq 0 ]]; then
  echo "ERROR: sequential mode requires --max-total-time > 0" >&2
  exit 1
fi

for target in "${TARGETS[@]}"; do
  run_sequential_target "$target"
done

trap - INT TERM EXIT

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
SCRIPT_PATH="$ROOT_DIR/scripts/$(basename "${BASH_SOURCE[0]}")"
RUN_LOCAL_FUZZ="$ROOT_DIR/scripts/run_local_fuzz.sh"
FUZZ_ROOT="$ROOT_DIR/wasm-neovm/fuzz"
STATE_ROOT="$ROOT_DIR/build/fuzz-long"

DEFAULT_TARGETS_CSV="fuzz_rust_contract,fuzz_rust_contract_differential"
DEFAULT_ITERATION_SECONDS=86400
DEFAULT_SNAPSHOT_INTERVAL=300
DEFAULT_RESTART_DELAY=15
DEFAULT_TIMEOUT_SECONDS=120
DEFAULT_RSS_LIMIT_MB=0

CURRENT_RUNNER_PID=""
CURRENT_MONITOR_PID=""
SESSION_DIR=""

timestamp_utc() {
  date -u +%Y-%m-%dT%H:%M:%SZ
}

usage() {
  cat <<'USAGE'
Usage:
  scripts/run_long_fuzz.sh start [options]
  scripts/run_long_fuzz.sh status [--session-dir <path>]
  scripts/run_long_fuzz.sh stop [--session-dir <path>]

Long-running fuzz supervisor for multi-day or multi-week sessions.
It launches `run_local_fuzz.sh` in bounded iterations, rotates each iteration into
its own run directory, and writes periodic status snapshots under `build/fuzz-long/`.

Start options:
  --targets <csv>              Target list (default: fuzz_rust_contract,fuzz_rust_contract_differential)
  --iteration-seconds <sec>    Seconds per runner iteration before clean restart (default: 86400)
  --snapshot-interval <sec>    Seconds between status snapshots (default: 300)
  --restart-delay <sec>        Delay before restarting the next iteration (default: 15)
  --timeout <sec>              libFuzzer timeout passed through to run_local_fuzz.sh (default: 120)
  --rss-limit-mb <mb>          libFuzzer RSS limit passed through (default: 0)
  --max-len <bytes>            Optional libFuzzer max_len
  --no-build                   Skip the initial build step
  --replace                    Stop the current latest long-run session before starting a new one

Examples:
  scripts/run_long_fuzz.sh start --replace
  scripts/run_long_fuzz.sh start --targets fuzz_rust_contract,fuzz_rust_contract_differential --iteration-seconds 43200
  scripts/run_long_fuzz.sh status
  scripts/run_long_fuzz.sh stop
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
  local value
  for value in "$@"; do
    if [[ "$first" -eq 1 ]]; then
      printf "%s" "$value"
      first=0
    else
      printf "%s%s" "$delimiter" "$value"
    fi
  done
}

is_pid_alive() {
  local pid="${1:-}"
  [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null
}

latest_session_dir() {
  if [[ -e "$STATE_ROOT/latest" ]]; then
    readlink -f "$STATE_ROOT/latest"
  fi
}

resolve_session_dir() {
  local requested="${1:-}"
  if [[ -n "$requested" ]]; then
    readlink -f "$requested"
    return
  fi
  latest_session_dir
}

latest_iteration_dir() {
  local session_dir="$1"
  local run_root="$session_dir/runs"
  if [[ -e "$run_root/latest" ]]; then
    readlink -f "$run_root/latest"
    return
  fi
  if [[ -d "$run_root" ]]; then
    find "$run_root" -mindepth 1 -maxdepth 1 -type d | sort | tail -n 1
  fi
}

last_nonempty_line() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    return
  fi
  awk 'NF { line = $0 } END { if (line != "") print line }' "$path"
}

count_matching_files() {
  local dir="$1"
  local pattern="$2"
  local since="${3:-}"
  if [[ ! -d "$dir" ]]; then
    echo 0
    return
  fi
  if [[ -n "$since" ]]; then
    find "$dir" -type f -name "$pattern" -newermt "$since" | wc -l | tr -d ' '
  else
    find "$dir" -type f -name "$pattern" | wc -l | tr -d ' '
  fi
}

load_session_env() {
  local session_dir="$1"
  local env_path="$session_dir/session.env"
  if [[ ! -f "$env_path" ]]; then
    echo "ERROR: missing session metadata: $env_path" >&2
    exit 1
  fi
  # shellcheck disable=SC1090
  source "$env_path"
}

render_status_report() {
  local session_dir="$1"
  load_session_env "$session_dir"

  local supervisor_pid=""
  if [[ -f "$session_dir/supervisor.pid" ]]; then
    supervisor_pid=$(<"$session_dir/supervisor.pid")
  fi

  local supervisor_state="dead"
  if is_pid_alive "$supervisor_pid"; then
    supervisor_state="alive"
  fi

  local run_dir
  run_dir=$(latest_iteration_dir "$session_dir")
  local run_started="n/a"
  local runner_pid=""
  if [[ -n "$run_dir" && -f "$run_dir/session.txt" ]]; then
    run_started=$(sed -n 's/^started_at=//p' "$run_dir/session.txt")
    runner_pid=$(sed -n 's/^runner_pid=//p' "$run_dir/session.txt")
  fi

  local disk_usage="n/a"
  if [[ -d "$session_dir" ]]; then
    disk_usage=$(du -sh "$session_dir" 2>/dev/null | awk '{print $1}')
  fi

  IFS=',' read -r -a targets <<< "$TARGETS_CSV"

  printf 'Session:      %s\n' "$session_dir"
  printf 'Started:      %s\n' "$STARTED_AT"
  printf 'Targets:      %s\n' "$TARGETS_CSV"
  printf 'Supervisor:   %s (%s)\n' "${supervisor_pid:-none}" "$supervisor_state"
  printf 'Latest Run:   %s\n' "${run_dir:-none}"
  printf 'Run Started:  %s\n' "$run_started"
  printf 'Runner PID:   %s\n' "${runner_pid:-none}"
  printf 'Disk Usage:   %s\n' "$disk_usage"
  printf 'Iteration:    %ss\n' "$ITERATION_SECONDS"
  printf 'Snapshots:    every %ss\n' "$SNAPSHOT_INTERVAL"
  printf 'Restart Wait: %ss\n' "$RESTART_DELAY"
  printf 'Snapshot At:  %s\n' "$(timestamp_utc)"

  local target
  for target in "${targets[@]}"; do
    local target_pid=""
    local target_alive="no"
    local target_log=""
    if [[ -n "$run_dir" && -f "$run_dir/${target}.pid" ]]; then
      target_pid=$(<"$run_dir/${target}.pid")
      if is_pid_alive "$target_pid"; then
        target_alive="yes"
      fi
    fi
    if [[ -n "$run_dir" ]]; then
      target_log="$run_dir/${target}.log"
    fi

    local corpus_dir="$FUZZ_ROOT/corpus/$target"
    local artifact_dir="$FUZZ_ROOT/artifacts/$target"
    local corpus_count=0
    local crash_count=0
    local timeout_count=0
    local oom_count=0
    local new_crash_count=0
    local new_timeout_count=0
    local new_oom_count=0
    if [[ -d "$corpus_dir" ]]; then
      corpus_count=$(find "$corpus_dir" -type f | wc -l | tr -d ' ')
    fi
    if [[ -d "$artifact_dir" ]]; then
      crash_count=$(count_matching_files "$artifact_dir" 'crash-*')
      timeout_count=$(count_matching_files "$artifact_dir" 'timeout-*')
      oom_count=$(count_matching_files "$artifact_dir" 'oom-*')
      new_crash_count=$(count_matching_files "$artifact_dir" 'crash-*' "$STARTED_AT")
      new_timeout_count=$(count_matching_files "$artifact_dir" 'timeout-*' "$STARTED_AT")
      new_oom_count=$(count_matching_files "$artifact_dir" 'oom-*' "$STARTED_AT")
    fi

    local last_line="(no log activity yet)"
    if [[ -n "$target_log" && -f "$target_log" ]]; then
      local extracted
      extracted=$(last_nonempty_line "$target_log")
      if [[ -n "$extracted" ]]; then
        last_line="$extracted"
      fi
    fi

    printf '\n[%s]\n' "$target"
    printf '  PID:       %s\n' "${target_pid:-none}"
    printf '  Alive:     %s\n' "$target_alive"
    printf '  Corpus:    %s\n' "$corpus_count"
    printf '  Crashes:   %s new / %s total\n' "$new_crash_count" "$crash_count"
    printf '  Timeouts:  %s new / %s total\n' "$new_timeout_count" "$timeout_count"
    printf '  OOMs:      %s new / %s total\n' "$new_oom_count" "$oom_count"
    printf '  Last:      %s\n' "$last_line"
  done
}

append_status_history() {
  local session_dir="$1"
  load_session_env "$session_dir"
  local run_dir
  run_dir=$(latest_iteration_dir "$session_dir")
  local snapshot_at
  snapshot_at=$(timestamp_utc)
  IFS=',' read -r -a targets <<< "$TARGETS_CSV"

  local target
  for target in "${targets[@]}"; do
    local target_pid=""
    local target_alive="no"
    if [[ -n "$run_dir" && -f "$run_dir/${target}.pid" ]]; then
      target_pid=$(<"$run_dir/${target}.pid")
      if is_pid_alive "$target_pid"; then
        target_alive="yes"
      fi
    fi

    local corpus_dir="$FUZZ_ROOT/corpus/$target"
    local artifact_dir="$FUZZ_ROOT/artifacts/$target"
    local corpus_count=0
    local crash_count=0
    local timeout_count=0
    local oom_count=0
    local new_crash_count=0
    local new_timeout_count=0
    local new_oom_count=0
    if [[ -d "$corpus_dir" ]]; then
      corpus_count=$(find "$corpus_dir" -type f | wc -l | tr -d ' ')
    fi
    if [[ -d "$artifact_dir" ]]; then
      crash_count=$(count_matching_files "$artifact_dir" 'crash-*')
      timeout_count=$(count_matching_files "$artifact_dir" 'timeout-*')
      oom_count=$(count_matching_files "$artifact_dir" 'oom-*')
      new_crash_count=$(count_matching_files "$artifact_dir" 'crash-*' "$STARTED_AT")
      new_timeout_count=$(count_matching_files "$artifact_dir" 'timeout-*' "$STARTED_AT")
      new_oom_count=$(count_matching_files "$artifact_dir" 'oom-*' "$STARTED_AT")
    fi

    local target_log=""
    if [[ -n "$run_dir" ]]; then
      target_log="$run_dir/${target}.log"
    fi
    local last_line=""
    if [[ -n "$target_log" && -f "$target_log" ]]; then
      last_line=$(last_nonempty_line "$target_log")
      last_line=$(printf '%s' "$last_line" | tr '\t' ' ' | sed 's/  */ /g')
    fi

    {
      printf '%s ' "$snapshot_at"
      printf 'target=%s ' "$target"
      printf 'alive=%s ' "$target_alive"
      printf 'pid=%s ' "${target_pid:-none}"
      printf 'corpus=%s ' "$corpus_count"
      printf 'crashes=%s ' "$crash_count"
      printf 'crashes_new=%s ' "$new_crash_count"
      printf 'timeouts=%s ' "$timeout_count"
      printf 'timeouts_new=%s ' "$new_timeout_count"
      printf 'ooms=%s ' "$oom_count"
      printf 'ooms_new=%s ' "$new_oom_count"
      printf 'latest_run=%s ' "${run_dir:-none}"
      printf 'last="%s"\n' "$last_line"
    } >> "$session_dir/status-history.log"
  done
}

write_status_files() {
  local session_dir="$1"
  local temp_status="$session_dir/status.txt.tmp"
  render_status_report "$session_dir" > "$temp_status"
  mv "$temp_status" "$session_dir/status.txt"
  append_status_history "$session_dir"
}

snapshot_loop() {
  local session_dir="$1"
  local child_pid="$2"
  local interval="$3"
  while is_pid_alive "$child_pid"; do
    write_status_files "$session_dir"
    sleep "$interval" || true
  done
  write_status_files "$session_dir"
}

cleanup_supervisor() {
  trap - EXIT INT TERM

  if is_pid_alive "$CURRENT_MONITOR_PID"; then
    kill "$CURRENT_MONITOR_PID" 2>/dev/null || true
    wait "$CURRENT_MONITOR_PID" 2>/dev/null || true
  fi

  if is_pid_alive "$CURRENT_RUNNER_PID"; then
    echo "[$(timestamp_utc)] stop requested; terminating runner pid=$CURRENT_RUNNER_PID"
    kill "$CURRENT_RUNNER_PID" 2>/dev/null || true
    wait "$CURRENT_RUNNER_PID" 2>/dev/null || true
  fi

  rm -f "$SESSION_DIR/current-runner.pid"
  write_status_files "$SESSION_DIR" || true
  echo "[$(timestamp_utc)] supervisor stopped"
}

stop_session() {
  local session_dir="$1"
  if [[ ! -d "$session_dir" ]]; then
    echo "ERROR: session directory not found: $session_dir" >&2
    exit 1
  fi

  local supervisor_pid=""
  if [[ -f "$session_dir/supervisor.pid" ]]; then
    supervisor_pid=$(<"$session_dir/supervisor.pid")
  fi

  if ! is_pid_alive "$supervisor_pid"; then
    echo "No live supervisor found for $session_dir"
    return 0
  fi

  echo "Stopping supervisor $supervisor_pid for $session_dir"
  kill -TERM -- "-$supervisor_pid" 2>/dev/null || kill -TERM "$supervisor_pid" 2>/dev/null || true

  local deadline=15
  while is_pid_alive "$supervisor_pid" && [[ "$deadline" -gt 0 ]]; do
    sleep 1
    deadline=$((deadline - 1))
  done

  if is_pid_alive "$supervisor_pid"; then
    echo "Supervisor did not exit cleanly; sending SIGKILL"
    kill -KILL -- "-$supervisor_pid" 2>/dev/null || kill -KILL "$supervisor_pid" 2>/dev/null || true
  fi

  write_status_files "$session_dir" || true
}

start_command() {
  local targets_csv="$DEFAULT_TARGETS_CSV"
  local iteration_seconds="$DEFAULT_ITERATION_SECONDS"
  local snapshot_interval="$DEFAULT_SNAPSHOT_INTERVAL"
  local restart_delay="$DEFAULT_RESTART_DELAY"
  local timeout_seconds="$DEFAULT_TIMEOUT_SECONDS"
  local rss_limit_mb="$DEFAULT_RSS_LIMIT_MB"
  local max_len=""
  local skip_build=0
  local replace_existing=0

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --targets)
        targets_csv="$2"
        shift 2
        ;;
      --iteration-seconds)
        iteration_seconds="$2"
        shift 2
        ;;
      --snapshot-interval)
        snapshot_interval="$2"
        shift 2
        ;;
      --restart-delay)
        restart_delay="$2"
        shift 2
        ;;
      --timeout)
        timeout_seconds="$2"
        shift 2
        ;;
      --rss-limit-mb)
        rss_limit_mb="$2"
        shift 2
        ;;
      --max-len)
        max_len="$2"
        shift 2
        ;;
      --no-build)
        skip_build=1
        shift
        ;;
      --replace)
        replace_existing=1
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

  require_command setsid "Install util-linux (setsid) and retry."

  mkdir -p "$STATE_ROOT"
  if [[ ! -x "$RUN_LOCAL_FUZZ" ]]; then
    echo "ERROR: missing executable runner: $RUN_LOCAL_FUZZ" >&2
    exit 1
  fi

  local existing_session=""
  existing_session=$(latest_session_dir || true)
  if [[ -n "$existing_session" && -f "$existing_session/supervisor.pid" ]]; then
    local existing_pid
    existing_pid=$(<"$existing_session/supervisor.pid")
    if is_pid_alive "$existing_pid"; then
      if [[ "$replace_existing" -eq 1 ]]; then
        stop_session "$existing_session"
      else
        echo "ERROR: latest long-run session is still active: $existing_session" >&2
        echo "Use --replace to stop it before starting a new one." >&2
        exit 1
      fi
    fi
  fi

  local session_dir="$STATE_ROOT/$(date -u +%Y%m%dT%H%M%SZ)"
  mkdir -p "$session_dir"
  mkdir -p "$session_dir/runs"
  touch "$session_dir/status-history.log"

  cat > "$session_dir/session.env" <<EOF
ROOT_DIR='$ROOT_DIR'
STATE_ROOT='$STATE_ROOT'
SESSION_DIR='$session_dir'
TARGETS_CSV='$targets_csv'
ITERATION_SECONDS='$iteration_seconds'
SNAPSHOT_INTERVAL='$snapshot_interval'
RESTART_DELAY='$restart_delay'
TIMEOUT_SECONDS='$timeout_seconds'
RSS_LIMIT_MB='$rss_limit_mb'
MAX_LEN='$max_len'
INITIAL_NO_BUILD='$skip_build'
STARTED_AT='$(timestamp_utc)'
EOF

  ln -sfn "$session_dir" "$STATE_ROOT/latest"

  local supervisor_log="$session_dir/supervisor.log"
  local cmd=(
    "$SCRIPT_PATH"
    "__supervise"
    --session-dir "$session_dir"
    --targets "$targets_csv"
    --iteration-seconds "$iteration_seconds"
    --snapshot-interval "$snapshot_interval"
    --restart-delay "$restart_delay"
    --timeout "$timeout_seconds"
    --rss-limit-mb "$rss_limit_mb"
  )
  if [[ -n "$max_len" ]]; then
    cmd+=(--max-len "$max_len")
  fi
  if [[ "$skip_build" -eq 1 ]]; then
    cmd+=(--no-build)
  fi

  setsid "${cmd[@]}" > "$supervisor_log" 2>&1 < /dev/null &
  local supervisor_pid=$!
  echo "$supervisor_pid" > "$session_dir/supervisor.pid"
  sleep 3

  if ! is_pid_alive "$supervisor_pid"; then
    echo "ERROR: long-run supervisor failed to stay alive. See $supervisor_log" >&2
    sed -n '1,80p' "$supervisor_log" >&2 || true
    exit 1
  fi

  write_status_files "$session_dir"

  echo "session_dir=$session_dir"
  echo "supervisor_pid=$supervisor_pid"
  echo "status_file=$session_dir/status.txt"
  echo "history_file=$session_dir/status-history.log"
  echo "supervisor_log=$supervisor_log"
}

status_command() {
  local requested_session=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --session-dir)
        requested_session="$2"
        shift 2
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

  local session_dir=""
  session_dir=$(resolve_session_dir "$requested_session")
  if [[ -z "$session_dir" || ! -d "$session_dir" ]]; then
    echo "ERROR: no long-run fuzz session found" >&2
    exit 1
  fi

  render_status_report "$session_dir"
}

stop_command() {
  local requested_session=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --session-dir)
        requested_session="$2"
        shift 2
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

  local session_dir=""
  session_dir=$(resolve_session_dir "$requested_session")
  if [[ -z "$session_dir" || ! -d "$session_dir" ]]; then
    echo "ERROR: no long-run fuzz session found" >&2
    exit 1
  fi

  stop_session "$session_dir"
  echo "stopped_session=$session_dir"
}

supervise_command() {
  local targets_csv="$DEFAULT_TARGETS_CSV"
  local iteration_seconds="$DEFAULT_ITERATION_SECONDS"
  local snapshot_interval="$DEFAULT_SNAPSHOT_INTERVAL"
  local restart_delay="$DEFAULT_RESTART_DELAY"
  local timeout_seconds="$DEFAULT_TIMEOUT_SECONDS"
  local rss_limit_mb="$DEFAULT_RSS_LIMIT_MB"
  local max_len=""
  local skip_build=0

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --session-dir)
        SESSION_DIR="$2"
        shift 2
        ;;
      --targets)
        targets_csv="$2"
        shift 2
        ;;
      --iteration-seconds)
        iteration_seconds="$2"
        shift 2
        ;;
      --snapshot-interval)
        snapshot_interval="$2"
        shift 2
        ;;
      --restart-delay)
        restart_delay="$2"
        shift 2
        ;;
      --timeout)
        timeout_seconds="$2"
        shift 2
        ;;
      --rss-limit-mb)
        rss_limit_mb="$2"
        shift 2
        ;;
      --max-len)
        max_len="$2"
        shift 2
        ;;
      --no-build)
        skip_build=1
        shift
        ;;
      *)
        echo "ERROR: unknown supervisor option '$1'" >&2
        exit 1
        ;;
    esac
  done

  if [[ -z "$SESSION_DIR" ]]; then
    echo "ERROR: --session-dir is required for supervisor mode" >&2
    exit 1
  fi

  mkdir -p "$SESSION_DIR/runs"
  echo "$$" > "$SESSION_DIR/supervisor.pid"
  trap cleanup_supervisor EXIT INT TERM

  echo "[$(timestamp_utc)] supervisor started pid=$$ targets=$targets_csv"
  local iteration=0
  local disable_build_after_first="$skip_build"

  while true; do
    iteration=$((iteration + 1))
    local cmd=(
      "$RUN_LOCAL_FUZZ"
      --log-root "$SESSION_DIR/runs"
      --targets "$targets_csv"
      --mode parallel
      --max-total-time "$iteration_seconds"
      --timeout "$timeout_seconds"
      --rss-limit-mb "$rss_limit_mb"
    )
    if [[ -n "$max_len" ]]; then
      cmd+=(--max-len "$max_len")
    fi
    if [[ "$disable_build_after_first" -eq 1 ]]; then
      cmd+=(--no-build)
    fi

    echo "[$(timestamp_utc)] iteration=$iteration starting"
    CURRENT_RUNNER_PID=""
    CURRENT_MONITOR_PID=""

    "${cmd[@]}" &
    CURRENT_RUNNER_PID=$!
    echo "$CURRENT_RUNNER_PID" > "$SESSION_DIR/current-runner.pid"

    snapshot_loop "$SESSION_DIR" "$CURRENT_RUNNER_PID" "$snapshot_interval" &
    CURRENT_MONITOR_PID=$!

    local run_status=0
    if wait "$CURRENT_RUNNER_PID"; then
      run_status=0
    else
      run_status=$?
    fi

    if is_pid_alive "$CURRENT_MONITOR_PID"; then
      kill "$CURRENT_MONITOR_PID" 2>/dev/null || true
      wait "$CURRENT_MONITOR_PID" 2>/dev/null || true
    fi
    CURRENT_MONITOR_PID=""
    rm -f "$SESSION_DIR/current-runner.pid"
    write_status_files "$SESSION_DIR"

    if [[ "$run_status" -eq 0 ]]; then
      echo "[$(timestamp_utc)] iteration=$iteration completed cleanly"
    else
      echo "[$(timestamp_utc)] iteration=$iteration exited non-zero status=$run_status; restarting"
    fi

    disable_build_after_first=1
    CURRENT_RUNNER_PID=""
    sleep "$restart_delay"
  done
}

main() {
  local command="${1:-start}"
  shift || true

  case "$command" in
    start)
      start_command "$@"
      ;;
    status)
      status_command "$@"
      ;;
    stop)
      stop_command "$@"
      ;;
    __supervise)
      supervise_command "$@"
      ;;
    -h|--help|help)
      usage
      ;;
    *)
      echo "ERROR: unknown command '$command'" >&2
      usage >&2
      exit 1
      ;;
  esac
}

main "$@"

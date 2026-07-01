#!/usr/bin/env bash
# Copyright (c) 2025-2026 R3E Network. SPDX-License-Identifier: MIT
#
# Continuous semantic differential fuzzer: each round regenerates a fresh batch
# of random op-composition expressions (gen_exprs) AND fresh random inputs (the
# seed), then runs every op — fixed + generated — on a real NeoVM and compares
# to native Rust. Stops and saves a repro on the first mismatch.
#
#   scripts/run_long_differential.sh                 # forever from seed 1000
#   START=5000 ROUNDS=50 RANDOM_PAIRS=300 scripts/run_long_differential.sh
set -uo pipefail
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
ORACLE="${NEO_VM_ORACLE:-/tmp/neo-validate/oracle}"
LOG="${LOG:-/tmp/neo-validate/longdiff.log}"
START="${START:-1000}"
ROUNDS="${ROUNDS:-0}"           # 0 = run forever (until a mismatch or kill)
RANDOM_PAIRS="${RANDOM_PAIRS:-200}"
GEN="$ROOT/contracts/fuzz-ops"
GENFILE="$GEN/src/generated_ops.rs"
REPRO_DIR="${REPRO_DIR:-/tmp/neo-validate}"

mkdir -p "$REPRO_DIR"
: > "$LOG"
echo "long differential started $(date) — seed from $START, +$RANDOM_PAIRS random pairs/op" | tee -a "$LOG"

# Build prerequisites once.
( cd "$GEN" && cargo build --release --bin gen_exprs >/dev/null 2>&1 ) || { echo "gen_exprs build failed" | tee -a "$LOG"; exit 2; }
cargo build -p wasm-neovm --release >/dev/null 2>&1 || { echo "wasm-neovm build failed" | tee -a "$LOG"; exit 2; }
if [ ! -x "$ORACLE" ]; then ( cd "$ROOT/conformance" && go build -o "$ORACLE" ./oracle ) || { echo "oracle build failed" | tee -a "$LOG"; exit 2; }; fi

# One-time trap-conformance gate (the trap_* methods are static, so checking
# once per session is enough). A failure is a serious abort-lowering bug.
if python3 "$ROOT/conformance/fuzz/trap_check.py" --root "$ROOT" --oracle "$ORACLE" \
        > "$REPRO_DIR/trapgate.log" 2>&1; then
    echo "trap-conformance gate: PASS — $(grep RESULT "$REPRO_DIR/trapgate.log" | tail -1)" | tee -a "$LOG"
else
    echo "!!! trap-conformance gate: FAIL — see $REPRO_DIR/trapgate.log" | tee -a "$LOG"
    tail -20 "$REPRO_DIR/trapgate.log" | tee -a "$LOG"
fi

# One-time storage-marshalling gate (the st_* round-trips are static). A failure
# is a storage syscall marshalling bug.
if python3 "$ROOT/conformance/fuzz/storage_fuzz.py" --root "$ROOT" --oracle "$ORACLE" \
        --seed "$START" --random 150 --quiet > "$REPRO_DIR/stgate.log" 2>&1; then
    echo "storage-conformance gate: PASS — $(grep RESULT "$REPRO_DIR/stgate.log" | tail -1)" | tee -a "$LOG"
else
    echo "!!! storage-conformance gate: FAIL — see $REPRO_DIR/stgate.log" | tee -a "$LOG"
    tail -20 "$REPRO_DIR/stgate.log" | tee -a "$LOG"
fi

i=0; seed=$START
trap 'echo "stopped $(date) after $i rounds" | tee -a "$LOG"; exit 0' INT TERM
while :; do
    i=$((i + 1))
    # Fresh expression batch for this seed (temp -> move so the lib never sees a
    # truncated generated_ops.rs).
    FUZZ_SEED=$seed "$GEN/target/release/gen_exprs" > "$REPRO_DIR/gen.$$.rs" 2>/dev/null \
        && mv "$REPRO_DIR/gen.$$.rs" "$GENFILE"
    out=$(python3 "$ROOT/conformance/fuzz/diff_fuzz.py" --root "$ROOT" --oracle "$ORACLE" \
            --seed "$seed" --random "$RANDOM_PAIRS" 2>&1)
    rc=$?
    res=$(printf '%s\n' "$out" | grep -E "RESULT|command failed|^SKIP" | head -1)
    if [ "$rc" -eq 3 ]; then
        # Oversized generated NEF (> neo-go's 131070 cap) — a generation-size
        # artifact, not a translator bug. Skip this seed and keep hunting.
        echo "[round $i seed $seed] SKIP — $res" | tee -a "$LOG"
        seed=$((seed + 1))
        continue
    fi
    echo "[round $i seed $seed] $res" | tee -a "$LOG"
    if [ "$rc" -ne 0 ]; then
        echo "!!! MISMATCH/FAILURE at seed $seed — saving repro to $REPRO_DIR/repro_seed_$seed.rs" | tee -a "$LOG"
        cp "$GENFILE" "$REPRO_DIR/repro_seed_$seed.rs"
        printf '%s\n' "$out" | tail -30 >> "$LOG"
        break
    fi
    [ "$ROUNDS" -ne 0 ] && [ "$i" -ge "$ROUNDS" ] && break
    seed=$((seed + 1))
done
echo "long differential ended $(date) after $i rounds" | tee -a "$LOG"

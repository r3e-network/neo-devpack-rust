#!/usr/bin/env python3
# Copyright (c) 2025-2026 R3E Network. SPDX-License-Identifier: MIT
"""Trap-conformance harness for the wasm->NeoVM translator.

The value-differential (diff_fuzz.py) can only assert oracle_top == native_value
for HALTing executions; it cannot express "this input MUST trap". That leaves the
translator's abort/unreachable lowering unchecked: a divide-by-zero, a MIN/-1
division overflow, or a Rust `unreachable!()` (panic=abort) must make the real
NeoVM FAULT, never return a silent wrong value.

This harness builds fuzz-ops, translates it, and invokes the deliberately
UNGUARDED `trap_*` methods (lib.rs, excluded from OP_NAMES) with both trapping
and safe inputs, asserting st=='FAULT' on the former and HALT+value on the latter.
"""
import json, os, subprocess, sys, argparse, tempfile

I64_MIN, I32_MIN = -(2**63), -(2**31)
FAULT = "FAULT"  # sentinel: expect the VM to abort, not return a value


def camel(op):
    p = op.split("_")
    return p[0] + "".join(x.capitalize() for x in p[1:])


def run(cmd, cwd=None):
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if r.returncode != 0:
        sys.stderr.write(f"command failed: {' '.join(cmd)}\n{r.stdout}\n{r.stderr}\n")
        sys.exit(2)
    return r.stdout


# (method, a, b, expected)  expected == FAULT means "the VM must abort".
CASES = [
    # signed i64 division: divide-by-zero and MIN/-1 overflow must trap.
    ("trap_div_s", 10, 2, 5),
    ("trap_div_s", -9, 2, -4),
    ("trap_div_s", 7, 0, FAULT),
    ("trap_div_s", I64_MIN, -1, FAULT),
    # signed i64 remainder: same trap inputs.
    ("trap_rem_s", 10, 3, 1),
    ("trap_rem_s", -10, 3, -1),
    ("trap_rem_s", 7, 0, FAULT),
    ("trap_rem_s", I64_MIN, -1, FAULT),
    # unsigned i64 division: only divide-by-zero traps (no MIN/-1 case).
    ("trap_div_u", 10, 3, 3),
    ("trap_div_u", -1, 2, 9223372036854775807),  # 0xFFFF.. as u64 / 2
    ("trap_div_u", 5, 0, FAULT),
    # i32 division: divide-by-zero and i32::MIN/-1 overflow must trap.
    ("trap_div_i32", 10, 2, 5),
    ("trap_div_i32", -7, 2, -3),
    ("trap_div_i32", 5, 0, FAULT),
    ("trap_div_i32", I32_MIN, -1, FAULT),
    # unreachable!(): the panic=abort -> wasm `unreachable` -> VM abort path.
    ("trap_unreachable", 5, 0, 5),
    ("trap_unreachable", 0, 0, FAULT),
]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--oracle", default="/tmp/neo-validate/oracle")
    ap.add_argument("--workdir", default="")
    a = ap.parse_args()

    root = os.path.abspath(a.root)
    fuzzdir = os.path.join(root, "contracts", "fuzz-ops")
    wk = a.workdir or tempfile.mkdtemp(prefix="trapcheck-")
    os.makedirs(wk, exist_ok=True)
    nef = os.path.join(wk, "fuzz-ops.nef")
    man = os.path.join(wk, "fuzz-ops.manifest.json")

    print("[1/4] build fuzz-ops wasm32")
    run(["cargo", "build", "--release", "--target", "wasm32-unknown-unknown", "--lib"], cwd=fuzzdir)
    wasm = os.path.join(fuzzdir, "target", "wasm32-unknown-unknown", "release", "fuzz_ops.wasm")

    print("[2/4] translate -> NEF")
    run([os.path.join(root, "target", "release", "wasm-neovm"),
         "--input", wasm, "--name", "FuzzOps", "--nef", nef, "--manifest", man])

    print("[3/4] oracle batch")
    batch_in = os.path.join(wk, "trap_in.jsonl")
    batch_out = os.path.join(wk, "trap_out.jsonl")
    with open(batch_in, "w") as f:
        for (m, x, y, _exp) in CASES:
            f.write(json.dumps({
                "nef_path": nef, "manifest_path": man, "method": camel(m),
                "arguments": [{"type": "integer", "value": str(x)},
                              {"type": "integer", "value": str(y)}],
                "signers": [], "initial_storage": [], "gas_limit": 2000000000,
            }) + "\n")
    run([a.oracle, "-batch", "-in", batch_in, "-out", batch_out])
    res = [json.loads(l) for l in open(batch_out) if l.strip()]
    assert len(res) == len(CASES), f"{len(res)} != {len(CASES)}"

    print("[4/4] check")
    fails = []
    for (m, x, y, exp), o in zip(CASES, res):
        st, top = o.get("st"), o.get("top")
        label = f"{m}({x},{y})"
        if exp is FAULT:
            ok = st != "HALT"
            got = st
        else:
            ok = st == "HALT" and top is not None and int(top) == exp
            got = f"{st}:{top}"
        tag = "ok " if ok else "FAIL"
        print(f"   [{tag}] {label:<28} expect={exp!s:<12} got={got}")
        if not ok:
            fails.append((label, exp, got))

    if fails:
        print(f"\nRESULT: {len(CASES)-len(fails)}/{len(CASES)} OK, {len(fails)} FAILURES")
        for (label, exp, got) in fails:
            print(f"  MISMATCH {label}: expected {exp}, got {got}")
        sys.exit(1)
    print(f"\nRESULT: {len(CASES)}/{len(CASES)} OK — all trap/halt expectations met")


if __name__ == "__main__":
    main()

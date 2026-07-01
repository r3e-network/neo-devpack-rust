#!/usr/bin/env python3
# Copyright (c) 2025-2026 R3E Network. SPDX-License-Identifier: MIT
"""Storage-marshalling differential for the wasm->NeoVM translator.

The value-differential (diff_fuzz.py) is pure (i64,i64)->i64 arithmetic and
cannot touch storage; the host/stateful storage syscalls (Put/Get/Delete/Has)
went unchecked on the real VM. This harness drives the SELF-CONTAINED storage
round-trip methods in fuzz-ops (lib.rs `st_*`): each writes and reads storage
within one invocation on a fresh VM, so the result is a pure function of (k, v)
that we know a priori — no refgen. Any oracle mismatch is a storage-syscall
marshalling bug (e.g. i64 key truncation, value round-trip loss, delete/has
inconsistency).
"""
import argparse, json, os, subprocess, sys, tempfile

MASK = (1 << 64) - 1


def i64(x):
    x &= MASK
    return x - (1 << 64) if x >= (1 << 63) else x


def camel(op):
    p = op.split("_")
    return p[0] + "".join(x.capitalize() for x in p[1:])


def run(cmd, cwd=None):
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if r.returncode != 0:
        sys.stderr.write(f"command failed: {' '.join(cmd)}\n{r.stdout}\n{r.stderr}\n")
        sys.exit(2)
    return r.stdout


# (method, expected(k, v))  — the a-priori semantics of each round-trip.
METHODS = [
    ("st_put_get", lambda k, v: v),          # put(k,v); get(k)          => v
    ("st_overwrite", lambda k, v: i64(~v)),  # put(k,v); put(k,!v); get  => !v
    ("st_two_keys", lambda k, v: v),         # distinct high-bit key must not alias
    ("st_del_absent", lambda k, v: 0),       # put; delete; get          => 0
    ("st_has_flags", lambda k, v: 2),        # has after put(=1)*2 + has after del(=0)
]

EDGES = [
    0, 1, -1, 2, -2, 7, 255, 256, -256,
    1 << 20, 1 << 32, (1 << 32) - 1, 1 << 40, 1 << 62, -(1 << 62),
    (1 << 63) - 1, -(1 << 63), 0x0123456789ABCDEF, -0x0123456789ABCDEF,
]


def gen_pairs(seed, n):
    pairs = []
    # structured: edges x a few edges — high-bit keys stress key encoding.
    for a in EDGES:
        for b in EDGES[:6]:
            pairs.append((a, b))
    # seeded random
    st = seed if seed else 0x9E3779B97F4A7C15
    def nxt():
        nonlocal st
        st ^= (st << 13) & MASK
        st ^= st >> 7
        st ^= (st << 17) & MASK
        return st
    for _ in range(n):
        pairs.append((i64(nxt()), i64(nxt())))
    return pairs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--oracle", default="/tmp/neo-validate/oracle")
    ap.add_argument("--workdir", default="")
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--random", type=int, default=200)
    ap.add_argument("--quiet", action="store_true")
    a = ap.parse_args()
    log = (lambda *x: None) if a.quiet else (lambda *x: print(*x))

    root = os.path.abspath(a.root)
    fuzzdir = os.path.join(root, "contracts", "fuzz-ops")
    wk = a.workdir or tempfile.mkdtemp(prefix="stfuzz-")
    os.makedirs(wk, exist_ok=True)
    nef = os.path.join(wk, "fuzz-ops.nef")
    man = os.path.join(wk, "fuzz-ops.manifest.json")

    log("[1/4] build fuzz-ops wasm32")
    run(["cargo", "build", "--release", "--target", "wasm32-unknown-unknown", "--lib"], cwd=fuzzdir)
    wasm = os.path.join(fuzzdir, "target", "wasm32-unknown-unknown", "release", "fuzz_ops.wasm")

    log("[2/4] translate -> NEF")
    run([os.path.join(root, "target", "release", "wasm-neovm"),
         "--input", wasm, "--name", "FuzzOps", "--nef", nef, "--manifest", man])

    log("[3/4] oracle batch")
    pairs = gen_pairs(a.seed, a.random)
    cases = []  # (method, k, v, expected)
    batch_in = os.path.join(wk, "st_in.jsonl")
    with open(batch_in, "w") as f:
        for (m, exp) in METHODS:
            for (k, v) in pairs:
                cases.append((m, k, v, exp(k, v)))
                f.write(json.dumps({
                    "nef_path": nef, "manifest_path": man, "method": camel(m),
                    "arguments": [{"type": "integer", "value": str(k)},
                                  {"type": "integer", "value": str(v)}],
                    "signers": [], "initial_storage": [], "gas_limit": 2000000000,
                }) + "\n")
    batch_out = os.path.join(wk, "st_out.jsonl")
    run([a.oracle, "-batch", "-in", batch_in, "-out", batch_out])
    res = [json.loads(l) for l in open(batch_out) if l.strip()]
    assert len(res) == len(cases), f"{len(res)} != {len(cases)}"

    log("[4/4] diff")
    from collections import defaultdict
    mism = defaultdict(list)
    ok = 0
    for (m, k, v, exp), o in zip(cases, res):
        if o["st"] != "HALT" or o["top"] is None:
            mism[m].append((k, v, exp, o["st"] if o["st"] != "HALT" else "NO_RET"))
            continue
        got = int(o["top"])
        if got == exp:
            ok += 1
        else:
            mism[m].append((k, v, exp, got))
    total_m = sum(len(x) for x in mism.values())
    print(f"RESULT: {ok}/{len(cases)} OK, {total_m} mismatches across {len(mism)} methods")
    for m in sorted(mism, key=lambda k: -len(mism[k])):
        print(f"  [{m}] {len(mism[m])} mismatches; samples:")
        for (k, v, exp, got) in mism[m][:5]:
            print(f"      k={k} v={v}  expected={exp}  got={got}")
    sys.exit(1 if total_m else 0)


if __name__ == "__main__":
    main()

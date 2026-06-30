#!/usr/bin/env python3
"""Self-contained differential fuzzer for the wasm->NeoVM translator.

Pipeline (all paths relative to --root, the repo working tree):
  1. build contracts/fuzz-ops to wasm32 (the single-op-per-method surface)
  2. translate it with <root>/target/release/wasm-neovm  -> NEF + manifest
  3. run the native `refgen` bin to produce ground-truth (op,a,b,expected)
     [native Rust == wasm semantics for these ops, so any mismatch is a
     translator lowering bug, not a reference artifact]
  4. feed every case to the NeoVM oracle in batch mode
  5. diff oracle top-of-stack vs expected, grouped by op

Usage:
  diff_fuzz.py --root /path/to/repo --oracle /tmp/neo-validate/oracle [--ops div_u,rem_u] [--quiet]

Exit code 0 = all match, 1 = mismatches found.
"""
import argparse, json, os, subprocess, sys, tempfile
from collections import defaultdict


def camel(op):
    p = op.split("_")
    return p[0] + "".join(x.capitalize() for x in p[1:])


def run(cmd, cwd=None, env=None):
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, env=env)
    if r.returncode != 0:
        sys.stderr.write(f"command failed: {' '.join(cmd)}\n{r.stdout}\n{r.stderr}\n")
        sys.exit(2)
    return r.stdout


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True, help="repo working tree")
    ap.add_argument("--oracle", default="/tmp/neo-validate/oracle", help="oracle binary")
    ap.add_argument("--ops", default="", help="comma-separated op filter (default: all)")
    ap.add_argument("--workdir", default="", help="scratch dir (default: temp)")
    ap.add_argument("--seed", type=int, default=None, help="random seed for continuous fuzzing (varies inputs each run)")
    ap.add_argument("--random", type=int, default=0, help="extra random input pairs per op")
    ap.add_argument("--quiet", action="store_true")
    a = ap.parse_args()

    root = os.path.abspath(a.root)
    fuzzdir = os.path.join(root, "contracts", "fuzz-ops")
    wk = a.workdir or tempfile.mkdtemp(prefix="difffuzz-")
    os.makedirs(wk, exist_ok=True)
    nef = os.path.join(wk, "fuzz-ops.nef")
    man = os.path.join(wk, "fuzz-ops.manifest.json")
    truth = os.path.join(wk, "truth.jsonl")
    log = (lambda *x: None) if a.quiet else (lambda *x: print(*x))

    log("[1/5] build fuzz-ops wasm32")
    run(["cargo", "build", "--release", "--target", "wasm32-unknown-unknown", "--lib"], cwd=fuzzdir)
    wasm = os.path.join(fuzzdir, "target", "wasm32-unknown-unknown", "release", "fuzz_ops.wasm")

    log("[2/5] translate -> NEF")
    run([os.path.join(root, "target", "release", "wasm-neovm"),
         "--input", wasm, "--name", "FuzzOps", "--nef", nef, "--manifest", man])

    log("[3/5] refgen ground truth")
    refgen_env = dict(os.environ)
    if a.seed is not None:
        refgen_env["FUZZ_SEED"] = str(a.seed)
    if a.random:
        refgen_env["FUZZ_RANDOM"] = str(a.random)
    out = run(["cargo", "run", "--release", "--bin", "refgen"], cwd=fuzzdir, env=refgen_env)
    open(truth, "w").write(out)
    cases = [json.loads(l) for l in out.splitlines() if l.strip()]
    op_filter = set(x for x in a.ops.split(",") if x)
    if op_filter:
        cases = [c for c in cases if c["op"] in op_filter]
    log(f"      {len(cases)} cases over {len(set(c['op'] for c in cases))} ops")

    log("[4/5] oracle batch")
    batch_in = os.path.join(wk, "batch_in.jsonl")
    batch_out = os.path.join(wk, "batch_out.jsonl")
    with open(batch_in, "w") as f:
        for c in cases:
            if c["op"] == "gen":
                method = "gen"
                argvals = [c["idx"], c["a"], c["b"]]
            else:
                method = camel(c["op"])
                argvals = [c["a"], c["b"]]
            f.write(json.dumps({
                "nef_path": nef, "manifest_path": man, "method": method,
                "arguments": [{"type": "integer", "value": str(x)} for x in argvals],
                "signers": [], "initial_storage": [], "gas_limit": 2000000000,
            }) + "\n")
    run([a.oracle, "-batch", "-in", batch_in, "-out", batch_out])
    res = [json.loads(l) for l in open(batch_out) if l.strip()]
    assert len(res) == len(cases), f"{len(res)} != {len(cases)}"

    log("[5/5] diff")
    mism = defaultdict(list)
    ok = 0
    for c, o in zip(cases, res):
        label = f"idx={c['idx']} a={c['a']} b={c['b']}" if c["op"] == "gen" else f"a={c['a']} b={c['b']}"
        if o["st"] != "HALT" or o["top"] is None:
            mism[c["op"]].append((label, c["e"], o["st"] if o["st"] != "HALT" else "NO_RET"))
            continue
        got = int(o["top"])
        if got == c["e"]:
            ok += 1
        else:
            mism[c["op"]].append((label, c["e"], got))
    total_m = sum(len(v) for v in mism.values())
    print(f"RESULT: {ok}/{len(cases)} OK, {total_m} mismatches across {len(mism)} ops")
    for op in sorted(mism, key=lambda k: -len(mism[k])):
        print(f"  [{op}] {len(mism[op])} mismatches; samples:")
        for (label, e, g) in mism[op][:5]:
            print(f"      {label}  expected={e}  got={g}")
    sys.exit(1 if total_m else 0)


if __name__ == "__main__":
    main()

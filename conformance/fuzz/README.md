<!-- Copyright (c) 2025-2026 R3E Network. SPDX-License-Identifier: MIT -->
# Differential fuzzer for the wasm→NeoVM translator

A differential tester that hunts translator lowering bugs by running the **same
Rust op code** two ways and comparing:

1. **native Rust** (`contracts/fuzz-ops` `refgen` bin) → ground truth — for the
   integer/bitwise/shift/conversion/BigInteger ops here, native Rust semantics
   equal WASM semantics, so any divergence is a translator bug, not a reference
   artifact.
2. **wasm → NeoVM** — the op compiled to wasm, translated by `wasm-neovm`, and
   executed on the real NeoVM (the neo-go conformance oracle, batch mode).

## Run

```
python3 conformance/fuzz/diff_fuzz.py --root "$PWD" --oracle /tmp/neo-validate/oracle [--ops div_u,rem_u] [--quiet]
```

It builds `contracts/fuzz-ops` to wasm, translates it, generates ground truth via
`refgen`, runs the oracle in batch over an edge-case + seeded-random input set
(78 ops × 396 inputs ≈ 30.9k cases), and reports mismatches grouped by op.
Exit 0 = all match.

The op surface (`contracts/fuzz-ops/src/lib.rs`) is one `#[neo_method]` per op,
each forwarding to a shared `ops::*` fn that the native `refgen` also calls — so
the export path and the reference can never drift. It covers i64/i32 arithmetic,
unsigned div/rem, bitwise/shift/rotate, clz/ctz/popcnt, all comparisons,
conversions, the `NeoInteger` (256-bit) operators, and chained/boolean-operand
comparisons.

## Oracle modes (conformance/oracle)

The oracle gained three modes used here and for triage:
- `-batch -in <jsonl> -out <jsonl>` — load a contract once, run many invocations.
- `-disasm <nef> -from <IP> -count <N>` — print `IP: OPCODE param`.
- `-trace -in <req> -from <IP> -to <IP>` — single-step, print each instr + estack.

## Bugs found + fixed

| Symptom (op) | Root cause | Fix |
|---|---|---|
| `div_u`/`rem_u` always returned the zero-divisor sentinel; `BigInteger >> n` off-by-one for negatives | `i32/i64.eqz`→`PUSH0; EQUAL` and `eq`/`ne`→`EQUAL`/`NOTEQUAL`. NeoVM `EQUAL` is **type-strict**: `Boolean(false) == Integer(0)` is false. LLVM chains comparisons (`if x==0` → `(x==0)==0`), feeding a Boolean into the next `eqz`/`eq`, which always misfired → guards fell through. | `eqz`→`NOT`; `eq`→`NUMEQUAL`; `ne`→`NUMNOTEQUAL` (numeric, coerce Boolean→0/1). |
| (class) `br_table` over a comparison-derived selector | selector compared to case labels with type-strict `EQUAL` | `br_table` selector compare → `NUMEQUAL` |

Both bugs traced to one root cause and affected **every contract with a chained
comparison / `if x == 0` / `match` / boolean logic** — they produced valid NEFs
but executed incorrectly on chain. After the fix: 30,888/30,888 op cases and a
separate 90/90 memory-lowering differential all match native Rust.

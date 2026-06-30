<!-- Copyright (c) 2025-2026 R3E Network. SPDX-License-Identifier: MIT -->
# Neo N3 Rust dev tech-stack validation

A point-in-time validation of the complete Rust-on-Neo development toolkit. Re-run
any row from the repo root; the whole sweep is green.

## Toolkit components (Solana/Move parity)

| Capability | Tool | Command |
|---|---|---|
| Build (Rust → NEF) | `wasm-neovm` translator | `cargo run -p wasm-neovm -- --input x.wasm --nef …` |
| Unit / logic tests (mock runtime) | `neo-test` | `cargo test -p neo-test` |
| Real-VM bytecode e2e (pure **and** stateful) | `neo-vm-test` | `make vm-test` |
| Full-chain e2e (deploy/multi-contract) | `integration-tests` + Neo Express | `make smoke-neoxp` |
| Debugging (disassemble + single-step trace) | oracle `-disasm`/`-trace`, `Contract::disassemble()`/`trace()` | — |
| Benchmarking | `#[neo_bench]` (criterion) | `cargo bench --features bench` |
| Coverage-guided fuzzing | 10 cargo-fuzz targets | `make fuzz-all` |
| Semantic fuzzing (native vs real VM) | `conformance/fuzz` differential | `make fuzz-differential` |
| Conformance gate | matrix checker | `make verify-neo-n3-conformance` |
| Cross-chain front-ends | `move-neovm`, `solana-compat` | `make test-cross-chain` |

## Validation results

| Check | Result |
|---|---|
| Full workspace test suite (`cargo test --workspace`) | **907 passed, 0 failed** |
| Cross-chain tests (Move + Solana → NeoVM) | **23 passed, 0 failed** |
| Neo N3 conformance matrix gate | **PASS** (rows match the syscall/native registry) |
| Contract sweep (build wasm32 → translate → validate NEF+manifest) | **32 / 32 VALID** |
| Real-VM e2e harness (`neo-vm-test`: pure + stateful + debug) | **9 passed, 0 failed** |
| Coverage-guided fuzz targets (compiler + devpack) | **9 / 9 clean** (translate, translate_config, structured_pipeline, nef, numeric, devpack_codec, syscall_surface, rust_contract, rust_contract_differential) |
| Devpack-types fuzz (`fuzz_devpack_types`) | **clean** (6.9M execs, NeoInteger/ByteString/Array vs references) |
| Semantic differential (translated op surface vs native Rust, real VM) | **77,910 / 77,910 OK** (fresh seed + 400 random pairs/op) |

## Fuzz coverage map

The fuzz system covers the full stack:

- **Compiler robustness** — `fuzz_translate`, `fuzz_translate_config` (arbitrary bytes never panic), `fuzz_structured_pipeline` (structured programs → translate + manifest + NEF invariants), `fuzz_nef` (NEF serialization), `fuzz_numeric` (varint/int/string encoders).
- **Devpack framework** — `fuzz_devpack_codec` (serde roundtrips), `fuzz_devpack_types` (NeoInteger/ByteString/Array semantics vs i128/std/Vec references), `fuzz_syscall_surface` (syscall alias/hash parity), `fuzz_rust_contract` (generate Rust contracts → compile → translate invariants), `fuzz_rust_contract_differential` (deterministic output).
- **Semantic correctness** — `conformance/fuzz/diff_fuzz.py` (+ `contracts/fuzz-ops`, 126 op forms) runs every translated op on a real NeoVM and compares to native Rust. Continuous via `make fuzz-differential SEED=$RANDOM RANDOM_PAIRS=400`.

`make fuzz-everything` runs all coverage-guided targets plus the semantic differential.

## Bugs found + fixed by this toolkit (this work)

The fuzzer/audit/e2e tooling surfaced and fixed several translator correctness bugs:
type-strict `EQUAL` for `eqz`/`eq`/`ne`/`br_table`; popcount/clz/ctz arbitrary-precision
wraparound; `handle_branch` dropping branch operands; chunked-memory marshalling of
heap-built syscall byte args; and a brittle conformance-gate script. All fixed and
regression-guarded.

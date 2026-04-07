# wasm-neovm Fuzzing

This directory contains the `cargo-fuzz` harnesses for the Neo Wasm -> NeoVM
translator and the surrounding Rust developer tooling.

## Prerequisites

- Rust nightly toolchain
- `cargo-fuzz`

```bash
rustup toolchain install nightly
cargo install cargo-fuzz
```

## Targets

- `fuzz_translate`
  - Feeds arbitrary bytes into the translator and checks that it never panics.
- `fuzz_translate_config`
  - Varies translation configuration knobs while translating arbitrary byte input.
- `fuzz_structured_pipeline`
  - Builds structured WAT/WASM contract templates and checks end-to-end invariants:
    translation stats, manifest metadata extraction, and NEF writing.
- `fuzz_nef`
  - Exercises NEF serialization with arbitrary scripts and metadata.
- `fuzz_numeric`
  - Exercises public integer/varint/string/byte encoding helpers.
- `fuzz_devpack_codec`
  - Exercises `neo-devpack` codec roundtrips and malformed decode inputs.
- `fuzz_syscall_surface`
  - Checks translator/devpack syscall alias + hash parity under arbitrary names and hashes.
- `fuzz_rust_contract`
  - Generates structured Rust `neo-devpack` contracts, compiles them to `wasm32-unknown-unknown`,
    then checks translation, manifest, and NEF invariants.
- `fuzz_rust_contract_differential`
  - Compiles the same generated Rust contract twice and checks deterministic Wasm, script,
    manifest, method token, and NEF output.

## Running

From `/home/neo/git/neo-llvm/wasm-neovm`:

```bash
cargo +nightly fuzz run fuzz_structured_pipeline -- -max_total_time=300
```

Or from the repository root:

```bash
make fuzz
make fuzz-compiler
make fuzz-all
```

For a long local fuzz session without CI, from the repository root:

```bash
scripts/run_local_fuzz.sh
```

To focus on the Rust compiler/devpack surfaces specifically, use a higher timeout because the
first generated contract build may need to warm the cargo target dir:

```bash
scripts/run_local_fuzz.sh --targets fuzz_rust_contract,fuzz_rust_contract_differential --timeout 120
```

That script:

- builds the fuzz binaries first
- starts all targets in parallel by default
- keeps running until interrupted
- writes logs and pid files under `build/fuzz-local/latest/`

To stop it, interrupt the foreground process or kill the recorded pids.

For a detached multi-day Rust contract fuzz session with periodic status snapshots
and per-iteration log rotation, use the long-run supervisor instead:

```bash
scripts/run_long_fuzz.sh start --replace --targets fuzz_rust_contract,fuzz_rust_contract_differential --timeout 120
scripts/run_long_fuzz.sh status
scripts/run_long_fuzz.sh stop
```

That supervisor:

- runs detached via `setsid`, so it survives the launching shell
- rolls each iteration into `build/fuzz-long/<session>/runs/<timestamp>/`
- refreshes `build/fuzz-long/<session>/status.txt`
- appends compact snapshot lines to `build/fuzz-long/<session>/status-history.log`
- restarts the bounded runner automatically after each iteration

Use `make fuzz-rust-long`, `make fuzz-long-status`, and `make fuzz-long-stop`
for the same workflow from the repository root.

`fuzz_rust_contract_differential` now belongs in the long-run rotation as well:
it checks Rust -> Wasm determinism and translator parity on the same structured
contract corpus, so it complements the crash-oriented `fuzz_rust_contract`
target instead of duplicating it.

## Design Notes

- The structured pipeline target is the highest-value harness for compiler
  behaviour because it exercises real contract shapes rather than only random
  bytes.
- The Rust contract targets mirror the `neo-solidity` approach: they prefer
  valid structured programs and determinism checks over blind string fuzzing, so
  they reach deeper `neo-devpack` macro/runtime integration paths and the actual
  Rust -> Wasm -> NeoVM compilation pipeline.
- The devpack codec and syscall surface targets extend fuzz coverage beyond the
  translator crate into the developer-facing runtime/tooling layer.
- Successful translations are checked for postconditions, not only panic
  freedom. This makes the harnesses more useful for catching silent corruption
  or drift between manifest/metadata/NEF outputs.

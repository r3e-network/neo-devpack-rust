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

That script:

- builds the fuzz binaries first
- starts all targets in parallel by default
- keeps running until interrupted
- writes logs and pid files under `build/fuzz-local/latest/`

To stop it, interrupt the foreground process or kill the recorded pids.

## Design Notes

- The structured pipeline target is the highest-value harness for compiler
  behaviour because it exercises real contract shapes rather than only random
  bytes.
- The devpack codec and syscall surface targets extend fuzz coverage beyond the
  translator crate into the developer-facing runtime/tooling layer.
- Successful translations are checked for postconditions, not only panic
  freedom. This makes the harnesses more useful for catching silent corruption
  or drift between manifest/metadata/NEF outputs.

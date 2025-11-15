# C Sample Contract

This directory demonstrates how to author a minimal NeoVM contract in plain C
and compile it into a NEF + manifest pair using the `wasm-neovm` translator.

## Entry Points

- `sum(a: i64, b: i64) -> i64` – returns the sum of the inputs.
- `version() -> i32` – exposes a simple numeric constant.
- `clamp_add(value: i64, delta: i64, max: i64) -> i64` – adds `delta` to
  `value` but clamps the result so it never exceeds `max`.

All functions are exported via `__attribute__((export_name("...")))` so the
compiler retains them when optimising with `-O3`.

## Building

The repository provides a helper script that wraps clang and the translator:

```bash
scripts/build_c_contract.sh contracts/c-hello
```

Outputs are written to `contracts/c-hello/build/`:

- `c_hello.wasm` – intermediate Wasm artefact.
- `c_hello.nef` – NeoVM bytecode.
- `c_hello.manifest.json` – merged manifest (base manifest + overlay metadata).

Exported methods and their parameter metadata live in
`manifest.overlay.json`. Update this file whenever the C exports change to keep
the manifest in sync—overlays are now validated against the actual Wasm exports,
so introducing or removing method names in the overlay triggers an error instead
of silently drifting out of sync.

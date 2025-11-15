# C → NeoVM Quickstart

This guide walks through authoring a minimal smart contract in plain C, compiling
it to WebAssembly with clang, and translating the resulting module into a NeoVM
NEF + manifest pair using the `wasm-neovm` toolchain.

## 1. Prerequisites

- Install **clang** ≥ 15 with WebAssembly support (`--target=wasm32-unknown-unknown`).
- Ensure Rust tooling is available (the translator is a Rust binary).
- Fetch translator dependencies:
  ```bash
  cargo fetch --manifest-path wasm-neovm/Cargo.toml
  ```

## 2. Structure Your Contract

Author C sources inside a dedicated directory, for example
`contracts/c-hello/contract.c`. Exported entry points must be annotated with
`__attribute__((export_name("...")))` so the optimiser keeps them alive:

```c
#include <stdint.h>

__attribute__((export_name("sum")))
int64_t sum(int64_t a, int64_t b) {
    return a + b;
}

__attribute__((export_name("version")))
int32_t version(void) {
    return 1;
}
```

Add a `manifest.overlay.json` next to the sources to describe ABI metadata that
cannot be inferred automatically:

```json
{
  "name": "CExample",
  "abi": {
    "methods": [
      {
        "name": "sum",
        "parameters": [
          { "name": "a", "type": "Integer" },
          { "name": "b", "type": "Integer" }
        ],
        "returntype": "Integer",
        "safe": true
      },
      {
        "name": "version",
        "parameters": [],
        "returntype": "Integer",
        "safe": true
      },
      {
        "name": "clamp_add",
        "parameters": [
          { "name": "value", "type": "Integer" },
          { "name": "delta", "type": "Integer" },
          { "name": "max", "type": "Integer" }
        ],
        "returntype": "Integer",
        "safe": true
      }
    ]
  },
  "supportedstandards": ["NEP-17"]
}
```

Overlay entries must correspond to real exports—if the JSON introduces a method
that is not present in the Wasm module (or omits one that is), the translator
fails so your manifest always mirrors the NEF script. For more overlay tips,
see [`docs/manifest-overlay-guide.md`](../manifest-overlay-guide.md).

## 3. Compile C → Wasm

Use the helper script provided by the repository:

```bash
scripts/build_c_contract.sh contracts/c-hello
```

Under the hood the script executes:

```bash
clang --target=wasm32-unknown-unknown \
  -O3 -nostdlib -fno-builtin -ffreestanding \
  -Wl,--no-entry -Wl,--export-all \
  contracts/c-hello/contract.c \
  -o contracts/c-hello/build/c_hello.wasm
```

The default flags produce a freestanding Wasm module without pulling in libc.
This avoids `env::` imports such as `memcpy`/`memset`. The translator bridges
those common shims automatically, but other host imports remain unsupported, so
prefer inlining or Neo syscall equivalents when bringing in additional runtime
helpers.
Pass additional clang flags after the first `--` if needed:

```bash
scripts/build_c_contract.sh contracts/c-hello CExample -- -DWASM_DEBUG
```

## 4. Translate Wasm → NEF + Manifest

The script automatically invokes the translator once compilation succeeds:

```text
==> Translating Wasm to NeoVM
Generated contracts/c-hello/build/c_hello.nef and contracts/c-hello/build/c_hello.manifest.json
```

Supply extra translator options (for example `--source-url`) after a second
`--`:

```bash
scripts/build_c_contract.sh contracts/c-hello CExample -- \
  -DWASM_DEBUG -- \
  --source-url https://example.com/c-example
```

## 5. Updating the Manifest

The base translator manifest contains the core ABI structure. The overlay JSON
is merged at translation time so it remains the single source of truth for
parameter names, safe flags, permissions, and supported standards. Update the
overlay whenever:

- A new function is exported or removed.
- Parameter types change.
- Additional metadata (permissions, trusts, events) is required.

## 6. Tips for C Authors

- Stick to integer-only logic. Floating-point, SIMD, atomics, and threads are
  currently rejected by the translator.
- Use `-fno-builtin` or provide local implementations of functions such as
  `memcpy`, `memmove`, or `memset`. The translator recognises these shims when
  imported from `env`, but avoiding additional host dependencies keeps modules
  self-contained.
- For reusable helpers, place their prototypes and inline implementations in a
  header inside the contract directory and include it from `contract.c`.
- Keep contract state in Neo storage (via syscalls) rather than relying on
  static/global variables. The translator supports Wasm memories and globals,
  but persistent state should be managed explicitly through the Neo APIs.

After translation the generated NEF/manifest pair can be deployed and exercised
exactly like the Rust-based examples in this repository.

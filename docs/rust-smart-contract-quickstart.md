# Rust → NeoVM Quickstart

This guide shows how to compile a Rust smart contract into a NeoVM NEF script and manifest using the `neo-devpack` SDK and the `wasm-neovm` translator in this repository.

## 1. Prerequisites

- Install the nightly toolchain or stable Rust ≥ 1.83.
- Add the Wasm build target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- Ensure the repo submodules have been checked out and the translator dependencies have been fetched:
  ```bash
  cargo fetch --manifest-path wasm-neovm/Cargo.toml
  cargo fetch --manifest-path rust-devpack/Cargo.toml
  ```

## 2. Sample Contract

The repository includes a ready-to-build example contract in `contracts/hello-world`. The manifest overlay describes the exported ABI, and a single Wasm export returns a constant so you can observe the full toolchain without additional runtime plumbing:

```rust
use neo_devpack::prelude::*;

neo_manifest_overlay!(r#"{
    "name": "HelloWorld",
    "features": { "storage": false }
}"#);

#[neo_safe]
#[no_mangle]
pub extern "C" fn hello() -> i64 {
    42
}
```

The `#[neo_safe]` attribute marks the exported function safe in the manifest so it can be invoked by other contracts without additional CLI arguments.
Any overlay fragments (from `neo_manifest_overlay!` or external files) must reference real exports—if a fragment introduces a method name the Wasm module does not actually export, the translator now fails so the manifest always matches the NEF script. Refer to [`docs/manifest-overlay-guide.md`](manifest-overlay-guide.md) for a full template and CLI tooling notes.

## 3. Build the Wasm Artifact

From the repository root run:

```bash
cargo build --manifest-path contracts/hello-world/Cargo.toml \
  --release --target wasm32-unknown-unknown
```

The compiled module will be written to:

```
contracts/hello-world/target/wasm32-unknown-unknown/release/hello_world_neo.wasm
```

## 4. Translate Wasm → NEF + Manifest

Invoke the translator with the Wasm payload, choosing the desired output locations:

```bash
cargo run --manifest-path wasm-neovm/Cargo.toml -- \
  --input contracts/hello-world/target/wasm32-unknown-unknown/release/hello_world_neo.wasm \
  --nef build/HelloWorld.nef \
  --manifest build/HelloWorld.manifest.json \
  --name HelloWorld
```

- `--nef` and `--manifest` control the output paths.
- `--name` sets the contract name within the manifest.
- Safe methods are declared inside the contract via the `#[neo_safe]` attribute
  (see the sample code above) so no additional CLI flags are required.

The command emits two files:

```
build/HelloWorld.nef
build/HelloWorld.manifest.json
```

Both artifacts bake in the metadata gathered from the manifest overlay.

## 5. Batch Build via Makefile

The repository ships with a Makefile that drives the entire pipeline. To build every sample contract (Wasm → NEF + manifest) run:

```bash
make examples
```

The generated artefacts are placed in the `build/` directory. Run `make clean` to remove the Wasm targets and translator outputs.

## 6. Additional Examples

Two richer samples are provided under the `contracts/` directory:

1. **NEP-17 micro token (`contracts/nep17-token`)** – a storage-backed token that enforces witness checks, emits transfer events, and persists balances. It exposes `init`, `totalSupply`, `balanceOf`, and `transfer` exports. View methods mark themselves safe via the `#[neo_safe]` attribute. Build and translate it the same way:

   ```bash
   make nep17-token
   ```

2. **Constant-product AMM (`contracts/constant-product`)** – a Uniswap-style swapper that keeps reserves in storage, charges a 0.3% swap fee, and validates the caller via `check_witness`. `getReserves` returns a packed 64-bit integer where the high 32-bits represent the X reserve and the low 32-bits represent the Y reserve. Query methods (`getReserves`, `quote`) carry `#[neo_safe]` metadata.

   ```bash
   make constant-product
   ```

3. **NEP-11 NFT (`contracts/nep11-nft`)** – illustrates minting, ownership tracking, balances, and transfers for individual tokens. The view methods (`totalSupply`, `balanceOf`, `ownerOf`) are safe by virtue of `#[neo_safe]`.

   ```bash
   make nep11-nft
   ```

These examples avoid complex reference types so they translate cleanly with the current Wasm → NeoVM feature set.

## 7. Next Steps

- Modify `contracts/hello-world/src/lib.rs` to add more methods, events, or manifest overlays.
- Use the generated NEF/manifest pair with the Neo CLI, Neo Express, or any compatible deployment tool.
- Add unit tests to the contract crate with standard Rust `#[test]`s; they will run natively before translating.
- Run `make test-contracts` from the repository root to execute contract tests across the full sample suite.

The same workflow applies to any `neo-devpack` contract crate—just point the translator at the compiled Wasm module.

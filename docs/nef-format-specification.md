# NEF (Neo Executable Format)

This note documents the container produced by the Wasm → NeoVM pipeline. The `wasm-neovm` crate now owns NEF emission end-to-end; there is no LLVM `AsmPrinter` in the tree anymore.

## Binary Layout

The emitted NEF matches the canonical N3 layout implemented by `Neo.SmartContract.NefFile`. All multi-byte integers use little-endian encoding.

1. **Magic** (`uint32`) – constant value `0x3346454E` (`NEF3`).
2. **Compiler** (`byte[64]`) – ASCII identifier for the toolchain, padded with zeros.
3. **Source** (`varstring`) – optional URL describing where the artefact originated. The CLI accepts `--source-url` and manifests may supply `extra.nefSource`.
4. **Reserved** (`byte`) – must be zero.
5. **Method tokens** (`vararray`) – static invoke metadata. Each token stores a 20-byte contract hash, a method name (`varstring`, max 32 bytes), a `u16` parameter count, a boolean return flag, and a `u8` call flag bitmask. Tokens can be provided through manifest overlays (`extra.nefMethodTokens`).
6. **Reserved** (`uint16`) – must be zero.
7. **Script** (`varbytes`) – length-prefixed NeoVM bytecode payload.
8. **Checksum** (`uint32`) – first four bytes of `hash256` (double SHA-256) over the preceding bytes.

The `varstring`, `vararray`, and `varbytes` encodings follow the Neo convention: a compact integer prefix (1/3/5/9 bytes) describing the length, followed by the payload.

## Writer API

`wasm-neovm/src/nef.rs` exposes two helpers:

```rust
pub fn write_nef<P: AsRef<Path>>(script: &[u8], output_path: P) -> anyhow::Result<()>

pub fn write_nef_with_metadata<P: AsRef<Path>>(
    script: &[u8],
    source: Option<&str>,
    method_tokens: &[MethodToken],
    output_path: P,
) -> anyhow::Result<()>
```

`write_nef` is a convenience wrapper that emits an empty source string and no method tokens. `write_nef_with_metadata` assembles the full header, script, and checksum. Empty scripts are rejected early so incorrect translation output is surfaced with a useful error.

## Supplying Metadata

`wasm-neovm` extracts NEF metadata from the manifest:

- **Source URL** – read from the top-level `"source"` field or `manifest["extra"]["nefSource"]`.
- **Method tokens** – read from `manifest["extra"]["nefMethodTokens"]`, which must be an array of objects with `hash`, `method`, `paramcount`, `hasreturnvalue`, and `callflags` fields. Hashes are specified as 40 hexadecimal characters (optionally prefixed with `0x`).

The helper `wasm_neovm::extract_nef_metadata` returns both fields in a single struct so callers can keep manifest and NEF in sync.
The translator also inspects the generated script for literal `System.Contract.Call` invocations; when it detects constant contract hashes, method names, and argument arrays, it synthesises method tokens automatically and merges them into the manifest before emission.

`neo_devpack` users can embed the metadata via custom sections (e.g. `neo_manifest_overlay!`) alongside ABI fragments.

## Example Usage

```rust
use wasm_neovm::{
    extract_nef_metadata,
    manifest::merge_manifest,
    translate_module,
    write_nef_with_metadata,
};

let wasm = std::fs::read("contract.wasm")?;
let translation = translate_module(&wasm, "HelloWorld")?;
let mut manifest = translation.manifest.value.clone();

let overlay: serde_json::Value = serde_json::from_str(r#"{
    "supportedstandards": ["NEP-17"],
    "extra": {
        "nefSource": "ipfs://hello-world",
        "nefMethodTokens": [{
            "hash": "0102030405060708090a0b0c0d0e0f1011121314",
            "method": "balanceOf",
            "paramcount": 2,
            "hasreturnvalue": true,
            "callflags": 3
        }]
    }
}"#)?;
merge_manifest(&mut manifest, &overlay);

let nef_metadata = extract_nef_metadata(&manifest)?;
let manifest_string = serde_json::to_string_pretty(&manifest)?;
write_nef_with_metadata(
    &translation.script,
    nef_metadata.source.as_deref(),
    &nef_metadata.method_tokens,
    "HelloWorld.nef",
)?;
std::fs::write("HelloWorld.manifest.json", manifest_string)?;
```

The translator guarantees that the manifest and script stay in sync (method offsets, ABI metadata), so it is safe to persist them together even though they live in separate files.

## Validation

- **Structural checks** – confirm the magic/compiler/source/token fields using a hex editor or `neo-cli`'s `show contract` command.
- **Checksum verification** – recompute `hash256` locally when debugging issues; a mismatched value will cause the NeoVM loader to reject the artefact.
- **Runtime validation** – execute the NEF on the NeoVM reference VM or `neo-cli` to ensure the manifest delivers the expected entry points.

The pipeline's tests cover NEF emission end-to-end (see `wasm-neovm/tests/basic.rs`), ensuring the emitted containers remain aligned with downstream tooling expectations.

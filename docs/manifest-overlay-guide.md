# Manifest Overlay Guide

This guide explains how to describe contract metadata via JSON overlays (either
embedded using DevPack macros or supplied as standalone files) so the
`wasm-neovm` translator can emit a correct manifest. It applies to Rust and C
contracts alike.

## Overlay Sources

1. **Rust macros** – `neo_manifest_overlay! { ... }` embeds JSON fragments in
   the Wasm module. The translator extracts and merges each fragment.
2. **External files** – pass `--manifest-overlay <file>` to the CLI (or use the
   per-contract `manifest.overlay.json`). The new `translate_with_config` API
   accepts overlays programmatically via `ManifestOverlay`.

All overlays are merged before safe flags or method tokens are propagated, so
they must reference real exports—otherwise the translator aborts to keep the
manifest aligned with the NEF script.

## Template

Any valid manifest field can be specified. Common structure:

```json
{
  "name": "MyContract",
  "abi": {
    "methods": [
      {
        "name": "balanceOf",
        "parameters": [
          { "name": "owner", "type": "Integer" }
        ],
        "returntype": "Integer",
        "safe": true
      }
    ],
    "events": [
      {
        "name": "Transfer",
        "parameters": [
          { "name": "from", "type": "Integer" },
          { "name": "to", "type": "Integer" },
          { "name": "amount", "type": "Integer" }
        ]
      }
    ]
  },
  "permissions": [
    { "contract": "0xff", "methods": ["balanceOf"] }
  ],
  "supportedstandards": ["NEP-17"]
}
```

### Tips

- **Safe methods** – set `"safe": true` per method; the translator propagates
  these flags so duplicates are harmless.
- **Method tokens** – if your overlay includes `extra.nefMethodTokens`, ensure
  the contract hash/method entries match the actual syscalls you emit.
- **Events** – `#[neo_event]` automatically emits `abi.events` entries (with canonical parameter types) so manual JSON is rarely needed. Only declare events explicitly if you are not using the macro or need to override metadata.
- **Permissions/trusts** – overlays deduplicate `permissions`, `supportedstandards`,
  and `trusts`, so additional entries are merged automatically. You generally no longer
  need to toggle the `storage` or `payable` flags manually—using `System.Storage.*`
  syscalls flips `features.storage`, and exporting `onPayment`/`onNEP17Payment`/
  `onNEP11Payment` enables `features.payable`. Permission `methods` may be either
  `"*"` or an array; wildcard values are preserved, and extra permission fields are
  retained when duplicate contract entries are merged.

## CLI Usage

```bash
wasm-neovm --input <path/to.wasm> \
  --nef build/MyContract.nef \
  --manifest build/MyContract.manifest.json \
  --manifest-overlay contracts/my-contract/manifest.overlay.json
```

The CLI now relies on `ManifestBuilder`, so overlays are validated against the
exported ABI before NEF/manifest files are written.

## Programmatic Usage

When embedding the translator in tooling:

```rust
use wasm_neovm::{translate_with_config, TranslationConfig, ManifestOverlay};
use serde_json::json;

let wasm = std::fs::read("contract.wasm")?;
let overlay = ManifestOverlay {
    value: json!({ "abi": { "methods": [ /* ... */ ] } }),
    label: Some("tooling overlay".into()),
};
let config = TranslationConfig::new("MyContract").with_manifest_overlay(overlay);
let translation = translate_with_config(&wasm, config)?;
```

Errors include the overlay label so mismatched ABI entries are easy to trace.

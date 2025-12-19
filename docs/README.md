# Documentation

This repository focuses on the Wasm → NeoVM translator, the Rust DevPack, and the cross-chain compatibility layers (Solana/Move). The following pages cover the current design, roadmap, and usage.

## Index

- **[wasm-pipeline.md](wasm-pipeline.md)** – Design notes and roadmap for the Wasm → NeoVM workflow.
- **[wasm-neovm-status.md](wasm-neovm-status.md)** – Current feature coverage for the translator.
- **[wasm-memory-design.md](wasm-memory-design.md)** – Deep dive into the linear memory helper architecture.
- **[wasm-table-design.md](wasm-table-design.md)** – Table runtime design and helper behaviour.
- **[cross-chain-compilation.md](cross-chain-compilation.md)** – Practical guide for Solana/Move contract compilation.
- **[CROSS_CHAIN_SPEC.md](CROSS_CHAIN_SPEC.md)** – Full cross-chain compilation specification.
- **[nef-format-specification.md](nef-format-specification.md)** – Reference for the NEF container format.
- **[c-smart-contract-quickstart.md](c-smart-contract-quickstart.md)** – Step-by-step guide for compiling C contracts to NEF.
- **[manifest-overlay-guide.md](manifest-overlay-guide.md)** – Shared reference for authoring manifest overlays (Rust macros, external JSON, `translate_with_config`).

These documents evolve with the translator and DevPack; please keep them in sync with code changes.

# Refactor Roadmap for wasm-neovm & DevPack

This document captures an incremental plan to reduce duplication, standardise
layering, and make heavier use of Rust traits/macros so the Wasm → NeoVM
pipeline stays maintainable as new features land.

## Goals

1. **Clear layering** – isolate Wasm ingestion, translation, and NeoVM emission
   into explicit modules so each layer can evolve independently.
2. **Shared abstractions** – deduplicate runtime helper generation, manifest
   merging, and metadata extraction via traits/macros.
3. **Testable units** – keep translator internals small enough that unit tests
   can target individual behaviours instead of end-to-end fixtures only.
4. **Documentation-first** – record design intent as we refactor so future
   contributors can follow the same conventions.

## Current Snapshot

- The original monolithic `wasm-neovm/src/translator/translation.rs` and
  `wasm-neovm/src/translator/runtime.rs` have been decomposed into focused
  submodules under `wasm-neovm/src/translator/translation/` and
  `wasm-neovm/src/translator/runtime/` (driver, per-op lowering, runtime helper
  families, and final NEF/manifest assembly).
- Manifest overlay merging/validation now lives under `wasm-neovm/src/manifest/`
  via `ManifestBuilder`; the CLI loads overlay JSON and forwards it via
  `TranslationConfig` instead of re-implementing merge logic.
- Runtime helper emission is grouped by domain (memory, tables, globals,
  records), but some helper builders still share similar patterns that could be
  standardised via traits/macros.
- The contract examples/scripts embed their own manifest overlays, but no
  shared schema exists to validate them outside of the translator runtime.

## Proposed Layered Design

```
┌──────────────────────────┐
│        CLI / API         │  (arg parsing, build orchestration)
└────────────┬─────────────┘
             │
┌────────────▼─────────────┐
│    Wasm Frontend Layer   │  (module parsing, validation, IR building)
│  - ModuleLoader trait    │
│  - WasmContext struct    │
└────────────┬─────────────┘
             │
┌────────────▼─────────────┐
│   Translation Mid-layer  │  (IR → NeoVM instructions)
│  - FunctionTranslator    │ (trait)
│  - RuntimeHelperBuilder  │ (trait + macro for repetitive helpers)
└────────────┬─────────────┘
             │
┌────────────▼─────────────┐
│ NeoVM Backend & Manifest │
│  - ScriptWriter          │ (opcode emission abstractions)
│  - ManifestBuilder       │ (deduped overlay merge/validation)
└──────────────────────────┘
```

### Key Abstractions

- **`ModuleLoader` trait** – wraps `wasmparser` traversal, producing a typed
  `ModuleIR` (functions, tables, globals, custom sections). This lets us unit
  test import/export handling without touching NeoVM code.
- **`FunctionTranslator` trait** – consumes `ModuleIR` + function bodies and
  emits a `NeoVMFunction` structure (script bytes + metadata). Concrete
  implementations can be swapped/tested individually.
- **`RuntimeHelperBuilder` trait** – provides reusable helpers for memory,
  table, and segment ops. Procedural macros (or `macro_rules!`) can generate the
  repetitive boilerplate currently copy-pasted across helper kinds.
- **`ManifestBuilder` struct** – owns merging, safe-flag propagation, overlay
  validation, and method-token integration. It becomes the single entry point
  for both the CLI and translator core.

## Refactor Roadmap

### Phase 1: Structural Cleanup

1. Extract `ModuleIR` types into `translator/ir.rs`, encapsulating imports,
   exports, tables, and segments.
2. Move Wasm parsing logic into `translator/frontend.rs` that implements
   `ModuleLoader` and outputs the IR.
3. Introduce `ScriptWriter` helper (wrapping opcode lookup + stack tracking) to
   replace ad-hoc `emit_*` functions scattered throughout `translation.rs`.

### Phase 2: Runtime Helper Deduplication

1. Define `RuntimeHelperBuilder` trait with methods like
   `emit_memory_helper(kind)` / `emit_table_helper(kind)`.
2. Implement a macro (e.g., `helper_kind!`) that generates boilerplate for each
   helper case, reducing the switch statements in `runtime.rs`.
3. Separate memory/table helper configuration data (slot IDs, bounds) from the
   emission logic so the builder simply iterates over a declarative structure.

### Phase 3: Manifest & Metadata Layer

1. Introduce `ManifestBuilder` with:
   - `fn from_methods(name, methods) -> Self`
   - `fn apply_overlay(&mut self, overlay_json, source_label)`
   - `fn finalize(&mut self, method_tokens, source_url) -> RenderedManifest`
2. Update both the CLI and translator to call `ManifestBuilder`, removing the
   duplicate overlay merge logic.
3. Add schema validation hooks (optional) so overlays can be linted before
   translation time if desired.

### Phase 4: CLI / API Surface

1. Expose a library entry point (`translate_with_config`) that accepts the new
   layered components, making unit tests and future integrations simpler.
2. Update `scripts/build_*.sh` to call the new API where possible and document
   how overlays are validated.
3. Consider providing `wasm-neovm` as a library crate consumed by cargo plugins
   or build scripts, leveraging the layered design.

### Phase 5: Example & Doc Alignment

Status: **In Progress**

- [ ] Consolidate manifest overlay snippets into reusable macros or JSON
  templates and document them (e.g., `docs/manifest-overlay-guide.md`).
- [ ] Add integration tests that consume the refactored APIs (`translate_with_config`)
  to guard against regressions.
- [ ] Update developer documentation to explain the new layering, traits, and
  macros so contributors know where to add features (touch `docs/wasm-pipeline.md`
  and relevant contract README files).

Notes:
- Overlay samples currently live per-contract; they should point to the shared
  guide once created.
- Integration tests can build atop `integration-tests/` or a lightweight harness
  that runs `translate_with_config` directly.

## Implementation Guidelines

- **Incremental commits** – tackle each phase in small PRs so the existing test
  suite can catch regressions early.
- **Trait-based contracts** – prefer traits for cross-cutting behaviour (loader,
  translator, helper builder) so unit tests can inject fakes/mocks.
- **Macro hygiene** – keep macros narrow in scope (e.g., generating match arms
  for helpers) and document them thoroughly.
- **Documentation-first** – update `docs/wasm-pipeline.md` and related guides
  whenever a layer’s behaviour changes.

## Next Steps

1. Land Phase 1 (IR extraction + frontend separation) with focused tests.
2. Schedule subsequent phases after confirming no performance regressions.
3. Track progress in this document, marking completed milestones and capturing
   follow-up tasks as the architecture settles.

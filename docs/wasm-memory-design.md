# Wasm Linear Memory & Bit Operation Lowering

This document captures the plan for supporting WebAssembly integer bit-counting
instructions and linear-memory operations in the NeoVM translator.

## Goals

- Provide a faithful implementation of Wasm linear memory semantics (single
  memory, byte-addressed, little-endian access) using the primitives available
  in NeoVM.
- Lower Wasm `load*`, `store*`, `memory.size`, and `memory.grow` instructions to
  sequences of NeoVM opcodes that manage memory through a hidden runtime buffer.
- Implement integer bit counting operators (`clz`, `ctz`, `popcnt` for `i32` and
  `i64`) using NeoVM arithmetic and control-flow instructions.
- Maintain constant folding and stack metadata tracking already present in the
  translator.
- Ensure correctness via targeted integration tests.

## High-Level Strategy

### Memory Representation

- **Static Slot Buffer**: On first entry into any exported function, initialise
  a static slot (index `0`) with a `Buffer` that represents linear memory.
- **Initial Contents**: The buffer is zero-initialised. Wasm data segments will
  be applied by emitting `MEMCPY` operations after allocation (future work).
- **Capacity**: Track both `current_size_bytes` and `current_pages` (64 KiB
  units) via dedicated static slots (`1` and `2`).
- **Access Discipline**: Loads/stores always perform explicit bounds checks
  before touching the buffer. Indexing is little-endian.
- **Growth**: `memory.grow` allocates a new buffer (`NEWBUFFER`), copies the old
  contents, zeros the new region, updates bookkeeping slots, and returns the
  previous page count (or `-1` on failure when exceeding a configurable limit).

### Runtime Helper Stubs

To keep generated code short, synthesize small helper routines that operate on
the linear memory. Each helper will be emitted once per translation and
referenced from load/store lowering sites.

Helpers to generate:

1. `neo_wasm_mem_init` – ensures static slots are ready (idempotent).
2. `neo_wasm_load_u8` / `neo_wasm_load_u16` / `neo_wasm_load_u32` /
   `neo_wasm_load_u64` (signed variants reuse the unsigned version + sign
   extension).
3. `neo_wasm_store8` / `store16` / `store32` / `store64`.
4. `neo_wasm_memory_size` – returns current pages.
5. `neo_wasm_memory_grow` – implements the growth logic.

Helpers will live at the end of each generated script. Calls are lowered via
`CALL_L` with computed offsets.

### Bit Counting Operators

For lack of native opcodes, use arithmetic sequences:

- **CLZ**: For `i32`/`i64`, repeatedly check high halves using binary search
  technique.
- **CTZ**: Mirror CLZ by scanning low halves (or compute by reversing bits). Use
  loops with `JMPIF_L`.
- **POPCNT**: Use HAKMEM-style parallel bit counting (mask and add) within
  32/64-bit masks.

Where operands are constant, leverage existing literal propagation to fold the
result and drop emission.

## Implementation Tasks

1. **Translator Refactor**
   - Add a `RuntimeBuilder` to manage helper emission and static slot indices.
   - Extend `Translation` to record helper offsets for manifest linking.

2. **Memory Ops Lowering**
   - `memory.size` → call helper.
   - `memory.grow` → call helper; ensure result is left on stack.
   - Loads/stores → ensure operand order is `(address, value?)`, mask offsets,
     call appropriate helper, then apply sign extension if needed.

3. **Bit Ops Lowering**
   - Introduce new lowering functions `emit_clz`, `emit_ctz`, `emit_popcnt`.
   - Support constants via compile-time evaluation.

4. **Testing**
   - Add fixtures covering loads/stores across byte widths, out-of-bounds
     checks, `memory.grow`, and bit operations with representative values.

5. **Documentation Updates**
   - Update `README.md` and `docs/wasm-pipeline.md` to describe the new helper
     runtime and operator coverage.

## Open Questions / Future Work

- Support for data segments and passive segments.
- Multiple memories (requires extending slot layout).
- Interaction with host syscalls for persistent storage.

The current scope focuses on single-memory Wasm modules and numeric bit ops.

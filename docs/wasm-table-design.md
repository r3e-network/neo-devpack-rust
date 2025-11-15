# Wasm Table Runtime & Reference Type Plan

This document captures the design for extending the Wasm→NeoVM translator with
complete support for reference-type instructions over funcref tables.

## Goals

- Represent Wasm tables (currently restricted to `funcref`) inside NeoVM so
  regular and bulk table instructions (`table.get/set/size/grow/fill/copy/init`
  and `elem.drop`) execute with the same semantics as the Wasm reference
  interpreter.
- Lower Wasm reference opcodes (`ref.null`, `ref.func`, `ref.is_null`, `ref.eq`,
  `ref.as_non_null`) to NeoVM stack operations while preserving interaction with
  `call_indirect`.
- Keep the runtime contract-friendly by piggybacking on the existing static
  slot initialisation routine so globals, memory, and tables are wired up in a
  single helper.
- Surface descriptive failures when Wasm modules attempt to use unsupported
  reference features (e.g., externref).

## Representation

- **Funcref Values**: Represented as 32-bit signed integers where `-1`
  denotes `ref.null` and non-negative values encode the translated function
  index. This matches the current `call_indirect` dispatcher, which already
  operates on function indices, and keeps comparisons (`ref.is_null`) trivial.
- **Tables**: Stored in per-table static slots as NeoVM arrays. Each slot holds
  the live array, while an optional companion slot stores the declared maximum
  (or `-1` if unbounded). Array elements use the same sentinel encoding as
  funcref values, so null entries and indirect calls share the same code path.
- **Element Segments**: Passive segment contents are cached in static slots as
  arrays, plus a drop-flag slot mirroring the data-segment handling that exists
  for linear memory. Active segments record their offset and bytes to be
  applied during runtime initialisation.

## Helper Routines

New helpers emitted once per translation keep generated code compact and reuse
NeoVM loops instead of re-synthesising them at every call site:

| Helper | Purpose |
| --- | --- |
| `neo_wasm_table_init` | Allocates arrays for each declared table, applies active element segments, and initialises passive segment caches + drop flags. |
| `neo_wasm_table_get_<n>` | (One per table) Bounds-checks an index and returns the funcref stored at position `n`. |
| `neo_wasm_table_set_<n>` | Bounds-checks then writes a funcref into table `n`. |
| `neo_wasm_table_size_<n>` | Returns current length of table `n` (thin wrapper around `SIZE`). |
| `neo_wasm_table_grow_<n>` | Implements the Wasm grow semantics with optional maximum checks, appending `delta` copies of the provided funcref and returning the previous length (or `-1` on failure). |
| `neo_wasm_table_fill_<n>` | Fills a range with the provided funcref after validating indices. |
| `neo_wasm_table_copy_<dst>_<src>` | Copies elements between tables with overlap handling and bounds checks. |
| `neo_wasm_table_init_from_passive_<segment>` | Copies from a passive element segment into a table and marks the segment as dropped. |
| `neo_wasm_elem_drop_<segment>` | Clears the cached passive segment and sets its drop flag. |

The helpers accept operands directly from the stack (index, length, value)
using NeoVM primitives (`SIZE`, `PICKITEM`, `SETITEM`, `APPEND`, `MEMCPY`, and
small loops built with `JMP*_L`). Where possible, helpers specialised per table
or segment let us bake static-slot loads as immediates, avoiding indirect slot
lookups at runtime.

## Translation Strategy

- `ref.null` → emit `PUSHM1` (the sentinel) and track the literal.
- `ref.is_null` → compare operand against `-1`, producing an `i32` boolean.
- `ref.func` → emit the function index literal; the validator ensures the
  referenced function exists.
- `call_indirect` → switch to table helpers: duplicate the selector, invoke
  `neo_wasm_table_get_<table>` to retrieve the target funcref, trap on `-1`,
  and then dispatch to the appropriate import or internal function based on the
  retrieved slot. This keeps table mutations visible to subsequent
  `call_indirect` executions.
- Table instructions (`get/set/size/grow/fill/copy/init`, `elem.drop`) lower to
  helper calls after requesting runtime initialisation. Each helper is patched
  once during `RuntimeHelpers::finalize` alongside the existing memory/data
  machinery.

## Implementation Tasks

1. **Runtime Metadata** – extend `RuntimeHelpers` with table descriptors, slot
   allocation, and passive element tracking. Update the initialisation helper to
   allocate arrays and apply active segments.
2. **Reference Opcodes** – lower `ref.null`, `ref.func`, `ref.is_null`,
   `ref.eq`, and `ref.as_non_null`, and adjust literal tracking so the sentinel
   participates in constant folding.
3. **Table Helpers** – synthesise helper scripts for each table/segment as
   described above, wiring them into the helper registry and ensuring they are
   emitted exactly once.
4. **Translator Lowering** – switch `call_indirect` to use the helper-based
   dispatch and add cases for all table instructions, including bounds checks
   and drop semantics.
5. **Tests & Docs** – expand integration tests covering reference operators,
   table mutation scenarios, and failure paths (out-of-range access, grow
   beyond maximum, double `elem.drop`, etc.), and update public docs to reflect
   the new coverage.

## Open Questions

- Supporting `externref` or tables of other reference types will require a
  richer representation than a simple integer sentinel; that can be layered on
  later by tagging values.
- The current plan specialises helpers per table to keep static-slot accesses
  fast. If module counts explode this may inflate bytecode; we can revisit with
  multi-parameter helpers if needed.
- Future `call_ref` and typed function references (from Wasm GC proposals) are
  out of scope for this milestone.

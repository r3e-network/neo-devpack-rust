# Translator limitations catalogue (L3)

> **Status**: catalogue produced 2026-06-27 from the
> `wasm-neovm/src/` tree. Re-runnable: `rg "bail!\(|unimplemented!\(|todo!\(" wasm-neovm/src/ | wc -l`.
> **Last count**: 186 `bail!` / `unimplemented!` / `todo!` sites across
> 45 files. Of these, **6 are real bugs** (L3.BUG-1..6) that need
> fixing; the rest are **intentional design limits** with a clear
> reason documented per category.

The translator's `bail!` / `unimplemented!` / `todo!` calls fall into
five categories:

1. **INTENTIONAL — explicit design limit** (e.g. f32/f64 unsupported,
   memory64/table64 unsupported, multi-value returns unsupported).
   These are documented here for newcomers. Calling contracts with
   any of these patterns gets a clear, actionable error message.

2. **INTENTIONAL — runtime safety check** (e.g. negative immediate,
   out-of-bounds script offset, unclosed block at function end).
   These fire when the translator produced invalid bytecode and the
   post-emit validator catches it. Real bugs in the translator
   upstream would surface as these, so they double as integration
   tests for the emit path.

3. **INTENTIONAL — wasm-opt lenient branch handling** (e.g. branch
   to a block with fewer values than the abstract stack says it
   needs; PUSH0 placeholders synthesised). Documented in
   `control.rs:122-130`.

4. **BUG — translator regression** (L3.BUG-1..6 below). These are
   cases where the translator should be able to handle the
   pattern but currently bails. Each gets a TDD test in
   `wasm-neovm/tests/`.

5. **UNKNOWN — needs runtime exercise** to determine if it's a
   real gap (in which case promote to BUG) or if the bail site is
   never reached for any test wasm we have. These are listed in
   §UNKNOWN below for L6 conformance-oracle exercise.

## INTENTIONAL design limits (the documented, intentional ones)

These are well-known NeoVM-vs-Wasm boundaries. The translator bails
with a clear, actionable message so contract authors know what to
change.

| Category | Where | Workaround for the user |
|---|---|---|
| f32/f64 (float) values anywhere | `numeric.rs:11/21/31`, `function.rs:70/101/170`, `control.rs:79` (multiple sites) | Use integer or BigInt arithmetic. NeoVM has no native float. |
| v128 (SIMD) | `numeric.rs`, `function.rs`, `control.rs` (multiple sites) | Not supported on-chain. |
| Reference types (ref.func, ref.null except FUNC) | `op_refs.rs:55-58` | Only funcref handles are modelled. |
| Multi-value returns (wasm MVP blocktypes) | `function.rs:77`, `control.rs:74`, `op_calls.rs:222` | Split into multiple single-value functions. |
| Block parameters (multi-value block inputs) | `control.rs:58-62` | Not on-chain. |
| memory64 / table64 / shared memory / shared tables | `parser/sections.rs:71/74/77/85/96/160/163/168/179` | NeoVM is 32-bit. |
| Static field slot > 255 | `helpers/statics.rs:23/51` | NeoVM hard limit. Reduce the number of static locals. |
| Too many locals/params (>255) | `function.rs:117/124/213/310/321` | INITSLOT operand is a byte. Split into smaller functions. |
| Try/catch in unsupported forms | `helpers/try_instructions.rs:39/63`, `op_control.rs:90/93/109/128/145` | Wasm try/catch is post-MVP. Use a single TRY block. |
| function-table call_indirect (the table itself is supported, but multi-value or non-i32/i64 params in the callee are not) | `op_calls.rs:215-222` | Restrict the funcref signature to (params=[i32/i64...], results=0 or 1). |
| br_table with too many cases | `control.rs:134/143` | Limit to ~256 jump targets per switch. |
| PICK / SWAP / DROP underflow | `ops/shift.rs:224/238/248` | Stack underflow; user code bug. |
| Active element segments (only passive supported) | `helpers_impl/tables.rs:43/45/190/205` | Passive segments + explicit table.init. |
| Active data segments (only passive supported) | `helpers_impl/memory.rs:133/149/181/190/219` | Same — passive data segments. |
| Memory init / data.drop on active segments | `helpers_impl/memory.rs:133/149` | Same. |
| RefAsNonNull trap on null funcref | `op_refs.rs:30` (ABORT emit) | Runtime trap; intentional. |
| Branch to function frame with fewer values | `control.rs:142` | Function return must have the result on stack. |
| Loop continue with wrong stack height | `control.rs:133` | Wasm spec requires exact match. |

## INTENTIONAL runtime safety checks (post-emit validation)

These fire when the translator has produced a script that the
post-emit validator rejects. Real translator bugs surface as these,
so they double as integration tests for the emit path.

| File | Why it's here |
|---|---|
| `helpers/validate.rs:25/39/62/71/81/103/120/129/138/178/207` | Script validator: opcode out of bounds, prefix out of bounds, target not on instruction boundary, etc. |
| `translation/imports/opcode.rs:60/73/82/100/111/135/142/160/169/178/182` | Lowering fails when CONVERT/SYSCALL opcode metadata is wrong (an internal invariant). |
| `translation/imports/syscall.rs:43/50/60/73/81/88/100/156/163/178/209/216/233/281/301/312/355/367/389/396/410/417/431/438/452/459/486/498/518/532/563` | Lowering validation: wrong import signature, missing opcode, missing syscall hash. These are caught at translate time, not post-emit, but they protect against a class of contract-author bugs. |
| `translation/imports/env.rs:22/29/40/48/78` | Env-style imports: signature mismatches. |
| `translation/imports/dispatch.rs:62/69` | Unsupported import module/name. |
| `runtime/storage.rs:44/59/74/86/97/183` | Storage helper lowering: bad slot/out-of-range. |
| `translation/imports/opcode.rs:13/21/53/61/82/99/111/135` | Opcode import validation. |
| `translation/driver/parser/*` | Wasm parser: bad section, bad type. |
| `runtime/init.rs:27` | Too many static slots (>255). |
| `runtime/memory/*` | Memory setup: out-of-bounds, missing section. |
| `translation/function.rs:50/101/170/213` | Misc translate-time guards. |

## BUG-1..6 (the real bugs found during this catalogue)

These are translator patterns where the `bail!` is reachable for
**valid wasm that a real contract would produce**, but the bail
message is misleading or the bail is wrong. Each gets a TDD test
in the next batch.

- **L3.BUG-1 — `control.rs:74` "blocks with multi-value results are not supported" fires for legitimate wasm-opt output.**
  wasm-opt commonly folds single-value block results through phi
  nodes that temporarily have arity 1. The bail should check whether
  the block actually produces a single value at the lowered
  NeoVM-script level. **TDD test**: build a wasm with
  `(block (result i32) ... end)` after wasm-opt, translate, assert
  no bail.

- **L3.BUG-2 — `op_calls.rs:222` "call_indirect returning multiple values is not supported" misses the empty-results case.**
  `(call_indirect (type $t) (i32.const 0))` where `$t` returns nothing
  is also rejected. **TDD test**: build a wasm with a 0-result
  indirect call, translate, assert no bail.

- **L3.BUG-3 — `function.rs:117/124` "function has too many parameters/locals" message could be friendlier.**
  The error says "for NeoVM INITSLOT" but doesn't say what the limit
  is or how to refactor. **Improvement**: include the actual count
  and a concrete suggestion (e.g. "split into smaller functions or
  use a struct to pack state into a single local").

- **L3.BUG-4 — `helpers/try_instructions.rs:39/63` try/catch handler lowering may mis-handle nested exceptions.**
  C# NeoVM exception handling has subtle CFI rules around `ENDTRY`
  in catch blocks. The translator's try lowering doesn't trace
  unreachable-paths through catch (so a `return` in catch leaves
  the control flow in a state the `ENDTRY` patcher doesn't expect).
  **TDD test**: wasm with `try ... catch ... return` then
  `(drop (call $f))` after the try, translate, assert the script
  validator passes.

- **L3.BUG-5 — `runtime/helpers_impl/memory.rs:181` "data segment N defined multiple times" is reachable from passive data segments emitted by the translator itself.**
  When the wasm module has two passive data segments with
  overlapping slot indices (a wasm-opt optimisation), the
  translator's bookkeeping sees a "redefined" segment. **TDD test**:
  wasm with two `(data ...) (data ...)` passive segments, translate,
  assert no bail.

- **L3.BUG-6 — `op_refs.rs:69-78` `ref.func` for an imported function index can emit a constant that the rest of the pipeline doesn't track.**
  The lowering emits `PUSH <index>` and registers it with
  `register_ref_func_constant`, but the *runtime* (exec harness)
  doesn't see this list, so `call_indirect` candidates that
  reference an imported function silently fail at runtime.
  **TDD test**: wasm with `(ref.func $import)` then
  `(call_indirect ...)`, run on the exec harness, assert the
  indirect call resolves.

Each of these gets a TDD test in `wasm-neovm/tests/`, a fix in
the source file, and a regression test in
`docs/translator-limitations.md#BUG-1..6`.

## UNKNOWN — needs L6 conformance exercise

These bail sites fire only for wasm patterns the L1 contracts in
`contracts/` don't produce. They need the L6 conformance oracle
(L1 + L2 + L6 together) to exercise. After L6 we revisit the
catalogue and reclassify:

- `runtime/helpers_impl/tables.rs:139-155` — `call_indirect` over
  a table that has both initial entries and an active element
  segment (wasm-opt merges them).
- `translation/imports/syscall.rs:281-563` (15 sites) — the syscall
  lowerings for Storage/Notify/GetExecutingScriptHash/etc. that
  L1 doesn't exercise; these may produce wrong-bytecode (rather
  than just bail) for some valid input.
- `runtime/storage.rs:44/59/74` — out-of-range storage slot; the
  validator may have a bug where the slot is computed before
  globals are registered.

## Maintenance

- Re-run the catalogue any time the translator grows:
  ```bash
  rg "bail!\(|unimplemented!\(|todo!\(" wasm-neovm/src/ | wc -l
  rg "bail!\(|unimplemented!\(|todo!\(" wasm-neovm/src/ -l | wc -l
  ```
- Any new `bail!` site must add a row to one of the categories
  above; if it doesn't fit, it's a candidate for INTENTIONAL
  (with reason) or BUG (with TDD test).
- Any new "TODO" or "FIXME" in the translator (not in
  `bail!` form) gets a follow-up issue and an entry in the
  audit doc.

*Catalogue produced 2026-06-27. Re-classify on every BUG fix.*

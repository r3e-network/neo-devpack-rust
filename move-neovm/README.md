# move-neovm

⚠️ **EXPERIMENTAL — not production-ready.** This crate is a minimal Move
bytecode → WASM translator that feeds the `wasm-neovm` pipeline:

```text
Move Source → Move Compiler → Move Bytecode → move-neovm → WASM → wasm-neovm → NEF
```

The lowering does not cover full Move semantics. Anything it cannot translate
*faithfully* is rejected with an explicit error instead of being silently
mis-compiled — the tables below enumerate exactly what is accepted, what is
rejected, and what error you will see. Do not use for real assets.

## Binary-format version support

`parse_move_bytecode` reads the version field after the Move magic
(`a1 1c eb 0b`) and rejects anything outside the range the parser has been
tested against:

| Version | Status | Error |
|---|---|---|
| 6 | Accepted (the only version the test fixtures exercise) | — |
| anything else | Rejected | `Move bytecode version {v} is unsupported; supported: 6..=6` |

The accepted range is exported as `bytecode::SUPPORTED_VERSION_MIN` /
`bytecode::SUPPORTED_VERSION_MAX`.

Note that the parser itself is minimal: it reads the table headers (module
handles, struct handles, function handles, identifiers, struct defs, function
defs — other table kinds are skipped), recovers struct/function *names* only
(struct fields, abilities, signatures and per-function code are not decoded
from the tables), and — when the module declares no functions — treats the
trailing bytes as a single `main` entry function using the simplified opcode
encoding below. It is not a full Move binary-format parser; real compiler
output is generally driven through the `MoveModule` API instead.

## Implemented instruction subset

These opcodes are lowered to real WASM by `translate_to_wasm` (values are
modelled as WASM `i32`/`i64`; Move integers are treated as unsigned 64-bit):

| Group | Opcodes | Notes |
|---|---|---|
| Constants | `LdU8`, `LdU64`, `LdTrue`, `LdFalse` | |
| Constants | `LdConst` | Pushes the constant-pool *index* as an `i64` — the constant pool itself is not parsed |
| Locals | `CopyLoc`, `MoveLoc`, `StLoc` | `CopyLoc` of a struct without the `copy` ability errors: `copy of resource {name} is not allowed` |
| Locals | `ImmBorrowLoc`, `MutBorrowLoc` | Lowered as a plain local read — borrows are not real references |
| Arithmetic | `Add`, `Sub`, `Mul`, `Div`, `Mod` | Unsigned; `Add`/`Sub`/`Mul` emit Move's abort-on-overflow checks (WASM `unreachable`); `Div`/`Mod` trap on divide-by-zero |
| Comparison | `Lt`, `Gt`, `Le`, `Ge`, `Eq`, `Neq` | Unsigned 64-bit |
| Logical | `And`, `Or`, `Not` | |
| Control flow | `Branch`, `BrTrue`, `BrFalse`, `Ret`, `Abort`, `Call`, `Nop`, `Pop` | Branches go through a pc-dispatch loop; `Abort` lowers to `unreachable`; `Call` targets in-module functions only |
| Casts | `CastU8`, `CastU64` | `CastU8` masks to the low 8 bits; `CastU64` is a no-op |

Public and entry functions are exported by name. A stack analysis pass
(`translator/analysis.rs`) type-checks every function up front, so malformed
stack usage fails translation with `stack underflow in ...` /
`type mismatch in ...` / `stack mismatch at target ...`.

## Explicitly-rejected features

`translate_to_wasm` validates the whole module first
(`validate_supported_module`) and fails fast on anything the flat 64-bit value
model cannot represent:

| Feature | Trigger | Error you will see |
|---|---|---|
| u128 / u256 beyond i64 | `TypeTag::U128` / `TypeTag::U256` in a parameter, return, or local | `unsupported Move type in {context}: u128 requires multi-word lowering and cannot be translated losslessly` (same for u256) |
| u128 opcodes | `LdU128`, `CastU128` | `unsupported Move opcode {op} at pc {pc} in function {name}: u128 values would be truncated to i64` |
| Structs with fields | `Pack`, `Unpack`, `BorrowField`, `MutBorrowField` | `unsupported Move opcode {op} ...: struct materialization and field access are not implemented` — struct *definitions* and opaque struct-typed values are fine |
| Global storage ops | `MoveTo`, `MoveFrom`, `Exists`, `BorrowGlobal`, `MutBorrowGlobal` | `unsupported Move opcode {op} ...: global resource operations are not implemented losslessly` (a storage-backed lowering exists in `lowering/instructions.rs` but is gated off behind this check until it is faithful) |
| Vector ops | `VecPack`, `VecLen`, `VecImmBorrow`, `VecMutBorrow`, `VecPushBack`, `VecPopBack` | `unsupported Move opcode {op} ...: vector operations are not implemented` |
| Generics | generic instruction bytes (e.g. `CallGeneric`, `PackGeneric`) | Rejected at parse time: `unsupported Move opcode 0x{byte}` — they are not in the simplified opcode table |
| Resource copy | `CopyLoc` of a struct lacking the `copy` ability | `copy of resource {name} is not allowed` |
| Keyless global access | resource op on a struct without the `key` ability | `struct {name} does not have the 'key' ability required for global operations` |

## Usage

```rust,ignore
use move_neovm::{parse_move_bytecode, translate_to_wasm, translate_move_to_wasm};

// Parse + translate in one step (returns WASM bytes + metadata)...
let translation = translate_move_to_wasm(&bytecode, "my-module")?;

// ...or drive the two stages yourself:
let module = parse_move_bytecode(&bytecode)?;
let wasm = translate_to_wasm(&module)?;
// Then use wasm-neovm to generate a NEF.
```

# Neo N3 LLVM Backend Design

## Goals
- Produce NeoVM bytecode scripts from LLVM IR.
- Support the full Neo N3 opcode set and syscall interface.
- Expose a stable target triple for downstream toolchains (e.g. `neos-wasm-neo3` placeholder).
- Integrate with Rust's `rustc_codegen_llvm` to compile Rust smart contracts end-to-end.

## Target Overview
The NeoVM is a stack-based virtual machine operating on 256-bit integers, byte arrays, and interop references. Key characteristics:

- **Stacks**: primary evaluation stack, alternate stack, return stack.
- **Type System**: dynamic VM types (Boolean, Integer, ByteString, Buffer, Array, Map, Struct, InteropInterface).
- **Control Flow**: structured around absolute instruction offsets; jump targets resolved at assembly phase.
- **Syscalls**: named with dot-separated identifiers, hashed to 4-byte IDs. Cover runtime, blockchains, native contracts (Oracle, Ledger, ContractManagement, etc.).

LLVM backend must bridge SSA form to stack-based execution.

## Target Triple and Data Layout
- **Target triple**: `neovm-unknown-neo3` (architecture `neovm`).
- **Pointer size**: 64-bit logical pointers, but physical representation is a reference handle (managed by VM). Use 64-bit pointer type with custom address spaces for interop.
- **Data layout**:
  - Little endian.
  - `i1`, `i8`, `i16`, `i32`, `i64`, `i128`, `i256` legal where `i256` maps to VM integer.
  - Aggregates serialized to stacks using VM array/map operations.

## Backend Components

### Target Information (TargetDesc)
- `NeoVMTargetInfo`: register target, set default features (full opcode set).
- `NeoVMTargetMachine`: config options for script assembly (e.g., deterministic asset ID, contract hash).
- `NeoVMSubtarget`: toggles for features like ABI revisions, contract manifest versioning.

### Register & Stack Model
- Represent the evaluation stack via virtual registers grouped as stack slots.
- Use pseudo registers `S0..Sn` to model top-of-stack values.
- Introduce custom `NeoVMRegisterInfo` and `NeoVMFrameLowering` to map LLVM stack objects to VM `PUSH/SETITEM` sequences.

### Instruction Selection
- Leverage GlobalISel to map LLVM IR opcodes to stack machine instructions.
- Each machine instruction emits stack behaviour metadata (push/pop counts).
- Provide legalization hooks for unsupported operations (e.g., `switch` lowered to `JMPIF` cascades).

### Machine Instruction Representation
- Define `.td` TableGen descriptions for all NeoVM opcodes:
  - **Stack manipulation**: `PUSHINT`, `PUSHDATA`, `DUP`, `SWAP`, `ROT`, `DROP`, `NIP`, `XDROP`, `XSWAP`, `XTUCK`, `DEPTH`, `ROLL`, `REVERSE`.
  - **Control flow**: `JMP`, `JMPIF`, `JMPIFNOT`, `CALL`, `TAILCALL`, `RET`, `THROW`, `ABORT`, `ASSERT`, `TRY`, `ENDTRY`, `ENDFINALLY`, `PUSHT`/`PUSHF` for booleans.
  - **Arithmetic**: `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `POW`, `SQRT`, `SHL`, `SHR`, `BOOLAND`, `BOOLOR`, `BOOLEOR`.
  - **Bitwise**: `INC`, `DEC`, `SIGN`, `ABS`, `NEGATE`, `BITAND`, `BITOR`, `BITXOR`, `NOT`, `COMPARE`, `EQUAL`, `NOTEQUAL`, `LT`, `LE`, `GT`, `GE`.
  - **Numeric conversion**: `ISNULL`, `ISNUMERIC`, `ISARRAY`, `ISSTRING`, `ISSTRUCT`, `ISBOOL` etc.
  - **Memory**: `PACK`, `UNPACK`, `NEWARRAY`, `NEWARRAYT`, `NEWSTRUCT`, `NEWSTRUCT0`, `APPEND`, `SETITEM`, `REMOVE`, `REVERSEITEMS`, `HASKEY`, `KEYS`, `VALUES`.
  - **Buffers/strings**: `CAT`, `SUBSTR`, `LEFT`, `RIGHT`, `SIZE`, `BINNOT`, `BINAND`, `BINOR` (alias to bitwise), `MEMCPY`, `MEMSET` (sequence expansions).
  - **Interop**: `SYSCALL`, `ABORTMSG`, `ASSERTMSG`, `NOP`, `RET` wrappers.
  - **Other**: `FROMALTSTACK`, `TOALTSTACK`, `ISTYPE` pseudo combinations.

### MC Layer
- `NeoVMAsmBackend`: responsible for assembling machine instructions into NeoVM bytecode.
- `NeoVMELFObjectWriter` equivalent replaced by `NeoVMScriptWriter` packaging script + manifest metadata.
- Implement custom `NeoVMFixupKinds` for relative jump offsets, syscall ID relocation, data literal sizes.
- Provide disassembler for debugging.

### Calling Convention & ABI
- Define `CC_NeoVM` handling entry point arguments (VM serializes call data as array on evaluation stack). Primary contract function signature: `(ExecutionEngine engine)` optional, but for Rust we expect explicit parameters typed.
- Lowering strategy: each exported method expects parameters serialized to stack (reverse order). Return values left on stack.
- Support multiple entry points via manifest `abi` (e.g., `_deploy`, `main`, public methods via dispatcher).

### Runtime Support
- Provide `librustneovm_rt` runtime with helpers for serialization, storage access, iterators, events.
- Map high-level APIs to syscalls (e.g., `System.Runtime.*`, `Neo.ContractManagement.*`).
- Provide panic handler lowering to `ABORTMSG` or `ASSERT`.

### Syscall Mapping
- Maintain TableGen/JSON mapping for all Neo N3 syscalls: name, hash, parameter schema, return type.
- Expose as LLVM intrinsic declarations (e.g., `declare neo_syscall.system_runtime_gettime()`).
- InstCombine pass rewrites runtime library calls to intrinsics, then to `SYSCALL` MIs.

### Metadata & Debug Info
- Encode sequence point metadata to support `--debug` script output (NeoVM debugger expects opcode offsets -> source mapping).
- Use `DIExpression` to annotate stack slots.

### Testing Strategy
- Unit tests using LLVM `lit` for instruction selection, MC encoding, and IR intrinsics.
- Integration tests compile sample Rust -> LLVM IR -> NeoVM to validate script behaviour via Neo VM CLI.

## Implementation Roadmap
1. Bootstrap target skeleton (`llvm/lib/Target/NeoVM`) with TableGen definitions.
2. Implement instruction lowering for arithmetic/control flow.
3. Add stack discipline verification pass to ensure stack height correctness.
4. Integrate MC layer to emit deterministic bytecode.
5. Implement intrinsic -> syscall lowering and runtime support library.
6. Wire Rust target via `rustc` `codegen-backends` plugin.
7. Provide contract packaging tool to emit `.nef` + `.manifest.json`.

### Initial Target Skeleton
Create the baseline file layout before filling in implementations. Each file contains complete implementations as documented in this specification.

- `llvm/lib/Target/NeoVM/NeoVMTargetMachine.cpp` — wire up target registration, pass pipeline hooks.
- `llvm/lib/Target/NeoVM/NeoVMISelLowering.cpp` — placeholder GlobalISel hooks, stack model comments.
- `llvm/lib/Target/NeoVM/NeoVMFrameLowering.cpp` — stub methods for frame layout (will emit stack discipline).
- `llvm/lib/Target/NeoVM/NeoVMInstrInfo.cpp` — instantiate instruction metadata; include autogenerated TableGen.
- `llvm/lib/Target/NeoVM/NeoVMRegisterInfo.cpp` — describe pseudo stack registers.
- `llvm/lib/Target/NeoVM/NeoVMAsmPrinter.cpp` — emit bytecode stream via MC layer.
- `llvm/lib/Target/NeoVM/NeoVMSubtarget.cpp` — feature parsing.
- `llvm/include/llvm/Target/NeoVM` directory with matching headers for each component.
- `llvm/lib/Target/NeoVM/NeoVM.td` — root TableGen file including opcode/MC definitions (empty lists initially).
- `llvm/lib/Target/NeoVM/CMakeLists.txt` + `llvm/include/llvm/Target/NeoVM/CMakeLists.txt` — register build targets.
- `llvm/test/CodeGen/NeoVM/` — placeholder `README.txt` describing test expectations.
- `llvm/lib/Target/NeoVM/NeoVMTargetInfo.cpp` — target registration hooks.

Document any deviations from this skeleton before committing code.

### NeoVMTargetMachine Responsibilities
- Instantiate `NeoVMSubtarget` and expose hooks for feature parsing (`CPU`, `FS` strings from target triple).
- Provide `createPassConfig` that schedules:
  1. Instruction selection (GlobalISel) with custom pipeline.
  2. Stack height verification pass post-isel.
  3. MC lowering to NeoVM bytecode emitter.
- Override `addPassesToEmitFile` to route output through `NeoVMAsmPrinter` producing `.nef` scripts and optional manifest sidecar.
- Register target via `LLVMInitializeNeoVMTarget`, `LLVMInitializeNeoVMTargetInfo`, `LLVMInitializeNeoVMTargetMC`, linking `Target` instance to triple `neovm`.
- Enforce unsupported features at creation time (e.g., no JIT, no object file emission) by returning errors.
- Surface per-module options: syscall table reference, debug flag, deterministic ordering mode.

### GlobalISel & Stack Verification Pipeline
- Use `RegBankSelect` tuned for stack pseudo registers; map virtual registers to stack slots with metadata describing push/pop effects.
- Implement custom `NeoVMInstructionSelector` extending `InstructionSelector` to emit stack-aware pseudos.
- Insert `NeoVMStackifyPass` after instruction selection to translate pseudos into concrete stack manipulations (`PUSH`, `SWAP`, etc.).
- Run `NeoVMStackHeightVerifier` to assert stack depth never negative and final height matches ABI contract.
- Append `NeoVMIntrinsicLoweringPass` prior to MC lowering to rewrite syscall intrinsics into `SYSCALL` machine ops with hashed IDs.
- Ensure pipeline executes with `-O0` and `-O1+` variants; disable unsupported optimizations (e.g., tail duplication that breaks stack ordering) via pass config toggles.

### NeoVMSubtarget Responsibilities
- Capture ABI revision (e.g., `neo3-mainnet`, `neo3-testnet`, future forks) affecting syscall availability and gas tables.
- Store feature bits controlling optional opcode sets (`hasTryCatch`, `hasManifestHints`, `hasBigIntegers`).
- Provide hooks for data layout adjustments (e.g., enabling 128-bit ints) depending on feature strings.
- Expose references to shared target resources: `NeoVMInstrInfo`, `NeoVMFrameLowering`, `NeoVMRegisterInfo` singletons tied to subtarget.
- Offer helper to fetch syscall metadata tables for intrinsic lowering.
- Coordinate with `TargetLoweringObjectFile` equivalent to flag non-object emission (script only).

### NeoVMInstrInfo & NeoVMRegisterInfo Responsibilities
- `NeoVMInstrInfo`
  - Wrap autogenerated opcode descriptors from TableGen (`NeoVMGenInstrInfo.inc`).
  - Provide helpers to reason about stack effects: `getPushCount`, `getPopCount`, `isPureStackOp`.
  - Support pseudo expansion hooks for high-level operations (e.g., `PUSHIMM256` -> `PUSHDATA1` sequence).
  - Expose canonical instruction forms for branch lowering (`getCondBranchOpcode`, `getUncondBranchOpcode`).
- `NeoVMRegisterInfo`
  - Model evaluation stack as virtual register bank (`S0`..`S31`) with unlimited allocation via stackifier.
  - Describe call-preserved “registers” as none (stack fully consumed per call).
  - Manage frame index elimination by mapping to VM storage operations (delegate to frame lowering pass).
  - Provide pointer register class metadata for address-space-qualified pointers (interop references).

### TableGen Opcode & Stack Metadata
- Root file `NeoVM.td` includes:
  - `def NeoVMTarget` deriving from `Target` with `InstructionSet` set to `NeoVM`.
  - `def NeoVMInstrInfo`/`NeoVMRegInfo` records referencing generated classes.
- Create `NeoVMBase.td` to hold common definitions:
  - Enum definitions for operand types (`NeoVMImm`, `NeoVMStackSlot`, `NeoVMSyscall`).
  - `class NeoVMInst` capturing opcode name, encoding byte, stack push/pop counts, intrinsic gas cost, feature predicate.
- Maintain stack effect table via TableGen `TSFlags` (bits 0-7 push count, 8-15 pop count, 16-23 gas cost) so generated code exposes helpers consumed by `NeoVMInstrInfo::getPushCount`/`getPopCount`.
- Introduce pseudo op definitions grouped by categories (stack manipulation, control flow, arithmetic, syscalls) with `isPseudo = 1`.
- Emit additional `defm` macros for convenience (e.g., `defm PUSHINT` family covering immediate sizes).
- Generate enumerations for syscalls referencing `neo_syscalls.json` through a preprocessing step (CMake custom command) that outputs `NeoVMSyscalls.td`.
- Provide script to validate opcode number uniqueness vs official Neo N3 specification; add lit tests comparing encoding tables.
- Minimum instruction selection MVP targets integer arithmetic (`ADD`, `SUB`, `MUL`, `DIV`), comparisons (`EQUAL`, `NOTEQUAL`, `LT`, `LE`, `GT`, `GE`), and control flow (`JMP`, `JMPIF`, `JMPIFNOT`, `RET`).
  - `G_BR` lowers to `JMP` with block labels resolved at MC time.
  - `G_BRCOND` lowers to `JMPIF`/`JMPIFNOT` depending on branch sense; stackify ensures condition on top.
  - `G_ICMP` provides predicate for boolean results, tagged with `NeoVMTypeHint::Boolean`.

### Full Opcode Coverage Plan
- Divide NeoVM opcodes into categories with dedicated `.td` fragments:
  - `NeoVMStackOps.td`, `NeoVMControl.td`, `NeoVMArithmetic.td`, `NeoVMBinary.td`, `NeoVMInterop.td`.
- Encode each opcode's byte value, stack deltas, gas cost, and required feature bits.
- Include all extension opcodes (e.g., `PUSHT`, `PUSHF`, `TRY`, `ENDTRY`, `ASSERTMSG`).
- For each category, document unusual semantics (e.g., `TRY` pushes catch/ finally offsets) for later lowering passes.
- Maintain sync script comparing TableGen definitions against upstream Neo reference JSON to prevent drift.

### Stack Mapping Format for Stackify Pass
- Virtual registers produced by GlobalISel map to abstract stack slots annotated via `MachineInstr` metadata `!neovm.stack` with fields:
  - `push` (integer): number of values pushed by instruction.
  - `pop` (integer): number of values consumed prior to execution.
  - `types` (array): optional NeoVM type hints for runtime checks.
- `NeoVMStackifyPass` consumes this metadata to emit explicit stack ops using helper pseudos (`NEOVM_PUSH_SLOT`, `NEOVM_SWAP_SLOT`, etc.).
- Stack slots identified by index relative to top-of-stack after pops; pass ensures ordering aligns with operand requirements.
- After stackification, instructions replaced with real opcodes defined in TableGen; pseudos removed before MC emission.

### Stackify Algorithm Outline
1. **Metadata Harvest**: For each `MachineInstr`, read `push`/`pop` counts from `MCInstrDesc::TSFlags` and reconcile with `!neovm.stack` annotations; emit diagnostics if inconsistent.
2. **Virtual Stack Tracker**: Maintain vector representing current stack slots, mapping virtual registers to positions.
3. **Operand Ordering**: Before emitting an instruction, ensure required operands are on top of stack by inserting pseudos:
   - `NEOVM_STACK_LOAD $slot` to bring a value from deeper in stack to top (expands to series of `SWAP`/`ROT`).
   - `NEOVM_STACK_PERMUTE` for reordering multiple values at once.
4. **Instruction Emission**: Output underlying NeoVM opcode; update tracker by popping consumed operands and pushing results (with new virtual register IDs).
5. **Spill Handling**: When stack depth exceeds threshold, delegate to `NeoVMFrameLowering::materializeFrameObject` to store values in VM arrays.
6. **Cleanup**: Remove pseudo instructions, annotate final stack depth for verifier.

Pseudos required in TableGen (all `isPseudo=1`):
- `NEOVM_STACK_SWAP_SLOT` — operands: (`depth`: immediate index from top, `width`: number of contiguous values)
- `NEOVM_STACK_PUSH_SLOT` — operands: (`slot`: frame index or metadata handle) used when spilling to VM storage.
- `NEOVM_STACK_POP_SLOT` — operands mirror PUSH; restores value from storage to evaluation stack.
- `NEOVM_STACK_SYNC_META` — no operands; attaches updated `!neovm.stack` metadata for verifier.

Operand reordering strategy inside `ensureOperandsOnStack`:
- Identify operands required by the current instruction and locate their positions in the simulated stack tracker.
- For each operand starting from the deepest, emit `NEOVM_STACK_SWAP_SLOT depth=idx width=1` to bubble it to the top while updating the tracker to reflect swaps.
- When multiple contiguous operands are needed, use a single swap with `width` equal to the block size to reduce instruction count.
- After reordering, operands at the top match the instruction’s expected order; tracker pops them before pushing outputs.
- If an operand is missing (not tracked), fall back to metadata-derived expectations and log a diagnostic for the verifier pass.
- Initial implementation handles width=1 swaps only, emitting a sequence of `NEOVM_STACK_SWAP_SLOT` instructions for each operand while updating the stack tracker.
- Future optimization: detect contiguous operand groups and replace repeated depth-1 swaps with a single `width > 1` swap once validated.

### Metadata Emission in Instruction Selection
- Extend `NeoVMInstructionSelector` (GlobalISel) to attach `!neovm.stack` metadata during selection:
  - Compute expected pop count from operand usage; push count from result types.
  - Attach metadata node `{i32 push, i32 pop, [type hints...]}` to resulting `MachineInstr`.
- For legacy SelectionDAG paths, provide helper in `NeoVMISelLowering` that sets `MI.addOperand(MachineOperand::CreateMetadata(...))` after lowering.
- Intrinsic lowering should propagate metadata when replacing high-level calls with pseudos.
- Stackify pass validates metadata presence and falls back to TSFlags when absent, emitting warnings in debug builds.
- Instruction selector must invoke `annotateDefaultStackInfo` for all lowered ops and set type hints to `Integer` for arithmetic results and operands where applicable.
- Memory operations:
  - `G_LOAD` lowers to `NEOVM_LOAD` pseudo expanding to `PICKITEM`/`SETITEM` sequences depending on address space.
  - `G_STORE` lowers to `NEOVM_STORE` pseudo that consumes value and address references.
  - Stack metadata: loads push one value (hint inferred), stores pop two.
- Addressing model relies on encoded NeoVM arrays/maps; frame lowering manages stack slots for spills.

### Metadata Helpers
- Define constant metadata kind IDs:
  - `int NeoVMStackMDKind` obtained via `LLVMContext::getMDKindID("neovm.stack")`.
  - `int NeoVMStackSyncMDKind` for verifier synchronization checkpoints (`neovm.stack.sync`).
- Provide utility functions in `NeoVMMetadata.h/.cpp`:
  - `MachineInstr *emitStackSync(...)` inserts `NEOVM_STACK_SYNC_META` with optional stack snapshot.
  - `Optional<NeoVMStackInfo> getStackInfo(const MachineInstr &)` returning push/pop/type hints.
- Add `void setStackInfo(MachineInstr &, const NeoVMStackInfo &)` so instruction selection can annotate newly created instructions with stack metadata consistently.
- Instruction selection for arithmetic/logic should call `setStackInfo` immediately after building the MachineInstr, using push/pop values derived from operand counts; type hints map to NeoVM value categories (0=int, 1=bool, 2=buffer, etc.).
- Type hint codes:
  - `0` — Unknown / Any
  - `1` — Integer
  - `2` — Boolean
  - `3` — ByteString / Buffer
  - `4` — Array / Struct
  - `5` — Map
  - `6` — Interop handle
- All passes (stackify, verifier) use these helpers to avoid duplicated string lookups and to keep metadata consistent.

### Testing Plan
- Add `llvm/test/CodeGen/NeoVM/stackify-basic.ll` covering:
  - Simple function with add/sub operations ensuring stack metadata attaches (checked via `CHECK: NEOVM_STACK_SYNC_META`).
  - Validate push/pop counts in remarks once implementation complete (use `llvm-mc` disassembly in future update).
- Provide `RUN: llc -march=neovm -debug-only=neovm-stackify` invocation to confirm stack analysis logs (skip until pass emits output; mark test `XFAIL: *` initially).
- Add JSON validation test `neo-syscalls-json.test` once generator exists, ensuring `neo_syscalls.json` matches manifest schema.
### Initial Opcode Set (MVP)
- **Literals & Stack Ops**: `PUSH0`, `PUSH1`..`PUSH16`, `PUSHINT8/16/32/64/128/256`, `PUSHDATA1/2/4`, `DROP`, `DUP`, `SWAP`, `ROT`, `OVER`, `TUCK`.
- **Control Flow**: `JMP`, `JMPIF`, `JMPIFNOT`, `CALL`, `RET`.
- **Arithmetic**: `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `NEGATE`, `ABS`, `INC`, `DEC`.
- **Comparison/Logic**: `EQUAL`, `NOTEQUAL`, `LT`, `LE`, `GT`, `GE`, `BOOLAND`, `BOOLOR`, `NOT`.
- **Memory & Aggregates**: `PACK`, `UNPACK`, `NEWARRAY`, `SETITEM`, `REMOVE`, `APPEND`.
- **Syscall Pseudo**: `SYSCALL` with operand referencing generated syscall enum.
- Each opcode entry includes:
  - Encoding byte (`0x00` etc.).
  - Stack push/pop counts.
  - Feature predicate (default `HasBigIntegers` true).
  - Gas cost per Neo specification.

### Build Integration
- Update `llvm/lib/Target/NeoVM/CMakeLists.txt` to:
  - Add TableGen invocation via `tablegen(LLVM NeoVMGenInstrInfo.inc -gen-instr-info)` and related outputs (`NeoVMGenRegisterInfo.inc`, `NeoVMGenSubtargetInfo.inc`).
  - Register target library `LLVMNeoVMCodeGen` with sources (`NeoVMTargetMachine.cpp`, `NeoVMInstrInfo.cpp`, `NeoVMRegisterInfo.cpp`, `NeoVMFrameLowering.cpp`, `NeoVMAsmPrinter.cpp`, `NeoVMStackify.cpp`).
  - Link against common LLVM components (`LLVMCodeGen`, `LLVMTarget`, `LLVMMC`, etc.).
- Ensure `llvm/include/llvm/Target/NeoVM/CMakeLists.txt` installs generated headers and static headers (`NeoVMInstrInfo.h`, `NeoVMRegisterInfo.h`, ...).
- Add initialization functions to `NeoVMTargetInfo.cpp` (to be created) and include it in the build; call `RegisterTargetMachine` and `RegisterAsmPrinter` macros.
- Extend top-level `llvm/lib/Target/CMakeLists.txt` to add `add_subdirectory(NeoVM)`.

### NeoVMFrameLowering & Stack Discipline Passes
- `NeoVMFrameLowering`
  - Define entry/exit stack adjustments (mostly no-op, but responsible for initializing manifest context when needed).
  - Provide `emitPrologue`/`emitEpilogue` stubs that ensure stack height zero for exported entry points.
  - Translate frame objects into VM storage operations (e.g., using `NEWARRAY`/`SETITEM` for spilled locals).
  - Coordinate with `NeoVMRegisterInfo::eliminateFrameIndex` via helper methods to serialize frame slots.
- `NeoVMStackHeightVerifier`
  - Analyze `MachineFunction` after pseudo expansion to ensure stack depth invariants.
  - Track gas hints for each instruction for optional reporting.
  - Emit diagnostics referencing source locations when stack underflow/overflow detected.
  - Implementation reads `!neovm.stack` metadata when present, otherwise falls back to opcode TSFlags.
  - Maintains running depth per basic block, emitting warnings if depth becomes negative or diverges from expected metadata.
- `NeoVMStackifyPass`
  - Convert virtual register SSA form into explicit stack manipulation sequences.
  - Insert `SWAP`, `ROT`, etc., to satisfy instruction operand ordering.
  - Annotate instructions with stack height metadata for verifier consumption.
  - Trigger spills when stack depth exceeds configurable limit (default 512 elements) by invoking frame lowering helpers.

### MC Emission Workflow (AsmPrinter & ScriptWriter)
- `NeoVMAsmPrinter`
  - Receive `MachineFunction` stream, translate `MachineInstr` into NeoVM opcodes with serialized operands.
  - Maintain bytecode buffer tracking instruction offsets for jump fixups.
  - Emit supplementary manifest metadata section if requested via target options.
- `NeoVMScriptWriter` (MC layer helper)
  - Handle fixups for control-flow offsets, syscall hashes, and data payload lengths.
  - Produce `.nef` binary with deterministic ordering, append CRC checksum per Neo spec.
  - Optionally output `.nefdbgnfo` containing address→source mappings from debug info.
- `NeoVMAsmBackend`
  - Integrate with LLVM MC `MCAssembler` to finalize fixups and write the script.
  - Supply per-opcode gas cost table for optional instrumentation pass.

### Syscall Intrinsic Lowering
- Maintain registry file `neo_syscalls.json` describing name, hash, parameter/return schema, feature flags.
- Auto-generate LLVM IR intrinsic declarations (`llvm.neo.syscall.*`) via TableGen or custom generator.
- `NeoVMIntrinsicLoweringPass` maps high-level runtime calls to these intrinsics based on annotations from Rust runtime crate.
- During machine instruction selection, intrinsics lower to pseudo `NEOVM_SYSCALL` nodes carrying resolved hash and argument packing strategy.
- Provide diagnostic when a syscall requires features disabled in current subtarget (`hasTryCatch`, etc.).
- Embed syscall metadata in manifest for reflection and debugging.


## Toolchain Integration
- Provide `llc -march=neovm` to assemble `.ll` to `.nef` script.
- Provide `clang --target=neovm-unknown-neo3` for C-level experiments.
- Add `rustc --target neovm-unknown-neo3.json` target spec referencing custom codegen backend.

## Open Questions
- Stack depth analysis heuristics for loops/recursion.
- Gas price metadata integration (Neo 3 uses per-instruction costs) — embed table, allow instrumentation pass.
- Contract storage schema DSL integration with Rust attribute macros.

## Completion Roadmap
1. **Opcode & TableGen Coverage**
   - Expand `NeoVMBase.td` into category-specific fragments with the full Neo N3 opcode list, encoding bytes, stack deltas, gas costs, and feature predicates.
   - Generate register, subtarget, and intrinsic tables via TableGen; add validation script comparing encodings to the official Neo reference data.
2. **Instruction Selection Integration**
   - Implement `NeoVMInstructionSelector` (GlobalISel) to lower arithmetic/control/memory ops and attach stack metadata via `annotateDefaultStackInfo`/`setNeoVMStackInfo` with accurate type hints.
   - Provide SelectionDAG fallbacks or disable where unsupported.
3. **Stackify Implementation**
   - Complete operand reordering logic using documented pseudos.
   - Implement spill/restore via `NeoVMFrameLowering::materializeFrameObject` and storage helpers.
   - Finalize stack height verifier to enforce invariants and surface diagnostics.
4. **MC Layer & Emission**
   - Implement `NeoVMAsmPrinter`, `NeoVMScriptWriter`, and MC backend to emit `.nef` scripts, manifests, and debug info.
   - Add disassembler support for debugging.
5. **Syscall Integration**
   - Populate `neo_syscalls.json`, generate `NeoVMSyscalls.td`, wire intrinsic lowering, and add runtime helpers.
6. **Toolchain Plumbing**
   - Integrate target registration, pass pipelines, llc/clang/rustc target specs, and add `cargo neo-*` tooling per rust integration plan.
7. **Testing & Validation**
   - Build a comprehensive `lit` test suite covering instruction selection, stackify transformations, MC emission, and syscall lowering.
   - Add integration tests running compiled scripts on NeoVM interpreter.
8. **Rust Frontend & SDK**
   - Implement `rustc_codegen_neovm`, runtime crates, macros, and cargo commands as described in `docs/rust-integration.md` and `docs/rust-framework.md`.
9. **Performance & QA**
   - Optimize stack transforms, add gas accounting instrumentation, and set up CI pipelines.

Each milestone builds on previous sections; work should track designs and update documentation prior to code changes.

# Design: Neo N3 Platform Support (100%) — 2026-06-27

> **Status**: design proposal awaiting your review.
> **Prior art**: `audit-2026-06-27-neo-n3-platform-support.md` (in this
> same directory) and the systematic-audit design from 2026-06-24.

## Goal

Move from "internal-correctness done, platform support partial" to "100% Neo
N3 mainnet platform support": every system syscall reachable from a
contract, every native contract method callable, every emitted
opcode verified against the C# reference, with regression tests that
run against the C# NeoVM and catch regressions before merge.

## Non-goals (this design)

- A pure-Rust NeoVM re-implementation (we keep the Phase 1 exec harness as
  a fast feedback loop; reference validation is the C# VM).
- A new macro framework or breaking change to the existing `#[neo_method]`
  decorator (the macro/export redesign stays as documented follow-up).
- Token-economics work (oracle pricing, fee policies).
- Coverage of pre-N3 (Neo Legacy) or post-N3 (Neo X / R3E-only features).

## Approach

Six independent layers, each self-contained, each TDD'd, each with a
clear "done" criterion verifiable without the C# VM. **Layer 6 (the
C#-VM cross-compile test) is the conformance oracle** that the other
five layers are validated against.

```
+-----------------------------------------------------+
| L1: syscall wasm32 path matrix (33 syscalls)        |  ~15
|     (route everything through extern "C" shims)     |
+-----------------------------------------------------+
| L2: native contract routing (11 native contracts)   |  ~12
|     (port Phase 1 C1 pattern to all natives)        |
+-----------------------------------------------------+
| L3: translator coverage: 176 bail sites → catalog   |  ~3 days
|     + fix the ~5 real gaps (Q1-Q5)                  |
+-----------------------------------------------------+
| L4: devpack type/iterator quality (B5-B22 fixes)   |  ~3 days
|     (numeric, array, map, iterator ergonomics)      |
+-----------------------------------------------------+
| L5: doc + standard library macros (NEP-X)           |  ~2 days
|     (nep17!, nep11!, manifest extras, hardfork note)|
+-----------------------------------------------------+
| L6: C#-VM conformance harness (the oracle)          |  ~5 days
|     (compile NEF on real VM, run, diff events)      |
+-----------------------------------------------------+
```

Total estimate: **~4–5 weeks** of single-developer work, but mostly
mechanical. Many of the fixes (L1, L2) are 5–20 line changes once
the pattern is established.

## Layer 1: syscall wasm32 path matrix

### Goal
33 system syscalls: 9 already have a wasm32 path, 24 don't. **All 33 must
have a working wasm32 path before this design ships.** Behaviour must
match the C# reference exactly.

### Mechanism
For each syscall:
1. Read C# `ApplicationEngine.<X>.cs` to determine the function signature
   and return semantics.
2. Decide the on-chain ABI: the C# `Execute(...)` method
   takes `args` and `evaluationStack`/`ReferenceCounter` context. The
   wasm32 shim signature is a single function that takes a pointer
   to a serialised args buffer and writes its return into an output
   buffer (or a fixed-width return for primitives).
3. Add an `extern "C"` shim to `wrapper.rs` for the new path.
4. Add a host-mode path that the exec harness (Phase 1) calls.
5. The wasm32 path delegates to the host-mode path on native build
   (test-only); the actual wasm32 path is implemented as a libc
   symbol exported by the Neo VM (this is exactly how the existing
   `neo_runtime_get_time` etc. are linked). For each new shim,
   we add a corresponding entry in
   `wasm-neovm/src/host_bridge.h` (or similar) so the
   contract-emit side knows the symbol name.
6. **Tests**: TDD, one per syscall, asserting wasm32 path
   returns correct values for canonical inputs (use the
   exec harness for the host path; use a mock extern for the
   wasm32 path test — the test asserts the function calls
   the expected symbol with the expected args).

### Definition of done
- Every `NeoVMSyscall::*` method on wasm32 has a body that calls
  the new extern (no `Ok(NeoValue::Null)` / `default_value_for` fallback).
- A new test module `rust-devpack/neo-syscalls/tests/wasm32_syscalls.rs`
  enumerates every syscall and asserts its wasm32 path
  calls the expected extern with the expected args.
- The exec harness (host) covers every syscall too: integration test
  `cargo test --features exec -p wasm-neovm --test full_syscall_coverage`
  (added in this layer).

### TIER-1 fixes covered (B1–B4)
- B1: `get_executing_script_hash` / `get_calling_script_hash` /
  `get_entry_script_hash` (ByteString form) get three new externs.
- B2: `notify` extern gains a state-pointer arg, wrapper serialises
  the args array.
- B3: `storage_get` extern returns length; wrapper handles 0-length
  as missing.
- B4: `contract_call` / `load_script` / `contract_call_native` get
  a new extern that returns serialised result. Initial impl may
  panic with "cross-contract call not yet implemented" — that's
  still strictly better than silently returning Null (L1 should
  not silently lie).

## Layer 2: native contract routing

### Goal
11 native contracts: 1 (CryptoLib) is routed; 10 are not. **All 11 must
have a descriptor + canonical hash + per-method dispatch.**

### Mechanism
For each native contract:
1. Read C# `Native/<X>.cs` to enumerate the `Register("Neo.<X>.<method>")`
   or `Register("<X>.<method>")` calls.
2. Add an entry to `wasm-neovm/src/native_contracts.rs`:
   ```rust
   pub fn ledger_descriptor() -> NativeContractDescriptor {
       NativeContractDescriptor {
           hash: 0x7a86bc4a...little_endian_20_bytes,
           name: "Neo.Ledger",
           methods: &[
               ("getBlock", &[("index", Integer)]),
               ("getBlockByHash", &[("hash", Hash256)]),
               ("currentIndex", &[]),
               // ... etc
           ],
       }
   }
   ```
3. Add a helper in `native_contracts.rs` to resolve any
   `Neo.<X>.<method>` pattern.
4. Hook into the existing `C1` mechanism (auto-wildcard permissions
   from Phase 1) so the manifest picks up the right method tokens.
5. **Tests**: TDD, one per native contract: `cargo test
   -p wasm-neovm --test native_contract_routing` verifies
   the descriptor's hash matches the C# mainnet hash
   (looked up at compile time via a `const` literal verified
   against the published mainnet value).

### TIER-1 fixes covered (P2–P12)
All 10 unrouted native contracts get descriptors. The C# canonical
hash for each is the small numeric value embedded in the C# file's
`public static readonly UInt160 ...` or class attribute. We'll
source them from a `const` table checked against the C# source
in CI (new test in L6).

## Layer 3: translator coverage

### Goal
176 `bail!`/`unimplemented!` sites in `wasm-neovm/src/translator/`.
Some are intentional design limits (good); some are real bugs (bad).
Produce a public catalogue of all sites; fix the real bugs.

### Mechanism
1. New file `docs/translator-limitations.md` (or a section in the
   design) listing every `bail!` site, classified as
   `INTENTIONAL` (with reason and work-around for the user)
   or `BUG` (with link to a TDD test).
2. For each `BUG`: TDD, fix, add to the regression suite.
3. Mechanical grep: `rg "bail!\(|unimplemented!\(" wasm-neovm/src/translator/`
   → 1-by-1 review.

### Estimated real bugs
- `wasm_utils.rs:44` — `bail!("unsupported type {:?}")` for typed
  select results; probably intentional but untyped. Document.
- `control.rs:58` — IF block in single-value position. Probably
  intentional.
- `control.rs:134/143` — `br_table` jump table overflow. Verify
  the bytecode size and add a graceful error.
- `helpers/try_instructions.rs:39/63` — try-catch CFI. Verify
  the lowering matches C# NeoVM's exception handling.
- `helpers/statics.rs:23/51` — static field > 255. Intentional
  (NeoVM hard limit); improve the error message.
- `function.rs:310/321` — local allocation. Verify.

### TIER-1 fixes covered (Q1–Q5)
Q1: catalogue. Q2–Q5: fix the actual bugs in the catalogue.

## Layer 4: devpack type/iterator quality

### Goal
The audit found 22 type/iterator quality issues (B5–B22 + Q6–Q10).
This layer fixes the ones that surface in real contracts.

### Mechanism
For each finding:
- TDD test that asserts the correct behaviour.
- Fix.
- Update existing contract code (in `contracts/*`) to use the new
  ergonomic API where the old one was problematic.

### Prioritized subset
- Q6: `NeoByteString::Deref<Target = [u8]>` (efficiency + ergonomics).
- Q7: `NeoInteger::to_bigint()` / `from_bigint()`.
- Q8: `NeoIterator::next` / `value` modelled correctly with `NeoIteratorId`.
- Q9: `NeoContract::call_typed<T>` via `IInteroperable`.
- Q10: `NeoMap::remove_strict`.
- B18: `Cow<[u8]>` for `NeoByteString` (performance).
- B19: `NeoInteger::try_from` overflow check audit.
- B20: `NeoMap` iteration order doc.
- B21: `NeoArray::try_push` and `MAX_SIZE = 1024` enforcement.

## Layer 5: documentation + NEP standard library

### Goal
Make the SDK approachable to newcomers and the limitations
visible to integrators.

### Mechanism
- Update `README.md`: add a "production readiness matrix" — 1 row
  per syscall + 1 row per native contract, with ✅ / ⚠️ / ❌ status.
- Update `docs/translator-limitations.md` (from L3) — linked from README.
- Update all docs to mention the HF_Echidna hardfork (remove
  `NeoToken`/`GasToken` references).
- Add `nep17!` / `nep11!` macros in `neo-macros` that emit the standard
  boilerplate (name, symbol, decimals, total supply, balanceOf, transfer,
  Transfer event, etc.). This is a thin wrapper around the existing
  `#[neo_method]` infrastructure, not a redesign.
- Add the `groups` / `permissions` / `trusts` manifest field support
  (already partially emitted; document + test).

## Layer 6: C# NeoVM conformance oracle

### Goal
A real, automated test that compiles a contract to NEF, loads it in the
C# NeoVM, and compares events / storage / return values to expected
output. This is the only way to *guarantee* 100% N3 platform support.

### Mechanism
1. **CI integration**: add a `cs-vm` job that runs only on
   PR-merge-to-master (not every commit, to keep PR CI fast).
   It uses a small C# project under `tests/conformance/` that
   references the C# `neo` source from the
   `neo-project/neo` submodule (added in this design) and runs
   the contract.
2. **Test harness**: a `tests/conformance/Program.cs` that:
   - Loads the NEF + manifest emitted by the Rust devpack.
   - Builds a `TriggerType.Application` engine with mock signers +
     mock storage.
   - Invokes a method.
   - Asserts events, return, and storage match the expected JSON.
3. **Submodule**: `git submodule add https://github.com/neo-project/neo tests/conformance/neo-vm`
   at a pinned commit. We update the pin monthly.
4. **Build a reference contract set**: for every category, we have
   one C# reference contract + the Rust equivalent + the JSON expected
   output (events, return, storage diff). Categories:
   - Minimal `hello` (no syscalls).
   - NEP-17 (transfer, balanceOf, total supply, symbol, decimals).
   - NEP-11 (mint, transfer, ownerOf, tokensOf).
   - Oracle consumer (request + finish).
   - Storage round-trip.
   - Iterator round-trip.
   - Native call (`Neo.ContractManagement.GetContract`,
     `Neo.Ledger.GetBlock`, `Neo.Oracle.Request`).
   - Error path: assert !CheckWitness returns false.
5. **Diff output**: when the C# VM and the Rust contract disagree, the
   test prints the diff (event name, payload, return, storage).
   Disagreements are either Rust bugs (fix) or C# docs (update
   the expected JSON with justification + new behaviour in CHANGELOG).
6. **Fuzzing hook**: `cargo-fuzz` target for the translator that
   feeds random wasm modules and asserts the result is a well-formed
   NEF (or a clean error message).

### Definition of done
- `cargo test --features conformance` runs the C# VM tests and they pass.
- Every release adds at least one new reference contract pair.
- The `wasm2wat`/load-and-run on the exec harness passes for every
  contract in `contracts/`.

## What "100% N3 platform support" means — the testable definition

This design is done when **all** of the following are true:

1. Every `NeoVMSyscall::*` method has a working wasm32 path
   (no `default_value_for` / `unimplemented!` / silent-`Null`).
2. Every N3 native contract (`ContractManagement`, `CryptoLib`,
   `Ledger`, `Oracle`, `Policy`, `RoleManagement`, `StdLib`,
   `Notary`, `Governance`, `TokenManagement`, `Treasury`) has a
   descriptor and canonical hash routed through
   `native_contracts.rs`.
3. Every emitted opcode in every `contracts/*` wasm passes the
   exec harness without panic or fallback.
4. Every emitted NEF+manifest matches a C# NeoVM run
   (conformance test).
5. The translator's `bail!` sites are catalogued in
   `docs/translator-limitations.md`; the real bugs (estimated
   ~5) are fixed; the intentional ones are documented.
6. The `README.md` has a production-readiness matrix that shows
   100% (all rows ✅ or ✅ with notes).
7. The Neo N3 hardfork history is documented (HF_Aspidochelone
   through HF_Echidna) and our docs reflect the modern state
   (no `NeoToken`/`GasToken` references in the latest contract
   templates).
8. The fuzz target has run for ≥ 1 hour with no panics.

## Process

1. **Now** (this design): you review this spec, approve, then I write
   the implementation plan.
2. After your approval, I write `docs/superpowers/plans/2026-06-27-neo-n3-platform-support-plan.md`
   (the bite-sized TDD plan, one task per commit, each with a verification
   step).
3. I execute the plan end-to-end, fully autonomously (per the
   "fully autonomous, one gate" preference you selected). Each
   commit:
   - Lands a TDD test.
   - Implements the fix.
   - Runs `cargo fmt`, `cargo clippy --all-targets --all-features`,
     `cargo test --workspace`, and the new syscall/wasm32 test.
   - Is atomic and well-titled.
4. **Bumps version** to 0.7.0 once L1 + L2 are done (the high-impact
   platform-support layer); 1.0.0 once L1–L6 are all done with the
   conformance oracle passing.
5. **End-of-plan summary** back to you, like the prior audit summary.

## Open questions (please weigh in)

1. **C# submodule size**: the `neo-project/neo` source tree is large
   (~500MB). We could add it as a submodule and check out only the
   `src/Neo.VM` and `src/Neo.SmartContract` directories via sparse
   checkout. Reasonable? Or use a pre-built `neo-vm-test-runner`
   Docker image instead?
2. **Bumping 0.7.0 vs 0.6.1**: I recommend 0.7.0 (notable new
   functionality — all 24 missing syscall paths + 10 new native
   contract descriptors). 0.6.1 would be appropriate if we treat
   L1 as "bug fixes" (which the T1 audit was framed as).
3. **Fuzzing infrastructure**: do you want `cargo-fuzz` set up now
   or after L6? (I'm inclined to set it up in L6 with the conformance
   harness, since the feedback loop matters more than the raw fuzzer.)
4. **The 9 syscalls that already work**: should I still add a
   regression test per existing one in L1's test module? Or only
   the new ones? (I'm inclined to **yes** — the existing tests are
   host-mode; the new module tests the wasm32-path / extern-resolution
   which is exactly the layer that the audit found most broken.)

## Files this design will create / modify

**New files**:
- `wasm-neovm/src/native_contracts.rs` (expand; + ~250 lines per
  contract descriptor × 10 contracts).
- `docs/translator-limitations.md` (the catalogue from L3).
- `rust-devpack/neo-syscalls/tests/wasm32_syscalls.rs` (the
  matrix test from L1).
- `rust-devpack/neo-syscalls/tests/native_contract_routing.rs`
  (the L2 test).
- `tests/conformance/` (L6 submodule + harness).
- `neo-macros/src/expand/nep17.rs`, `nep11.rs` (L5).
- `rust-devpack/neo-types/src/bigint.rs`, `cow_bytestring.rs`,
  `iterator.rs` (L4, ergonomic extensions).

**Modified files**:
- `rust-devpack/neo-syscalls/src/wrapper.rs` (L1: +24 externs +
  +24 wasm32 paths).
- `wasm-neovm/src/translator/**` (L3: ~5 bug fixes + 176-site catalogue).
- `rust-devpack/neo-runtime/src/{runtime,contract,storage}.rs`
  (L4: type ergonomics).
- `README.md`, `CHANGELOG.md`, `docs/` (L5: doc updates + matrix).
- `.github/workflows/ci.yml` (L6: new `conformance` job).
- `Cargo.toml` (workspace version bump 0.6.0 → 0.7.0).
- All `contracts/*/Cargo.toml` (`neo-devpack` version bump).

## Risks

- **C# submodule maintenance**: pinning to a `neo-project/neo` commit
  means tracking upstream. Mitigation: monthly pin refresh;
  acceptance test uses the pinned commit, not `master`.
- **wasm-canonical ABI drift**: the C# VM may change its
  serialisation format for the future. Mitigation: L6 catches
  regressions immediately.
- **fuzz target may find a translator bug that has no clean fix**.
  Mitigation: TDD the bug, decide to fix or document. Bug budget:
  any fuzz-discovered bug in T6 is a release-blocker for 1.0.0.
- **Bumping to 0.7.0 with the macro/export redesign still pending
  (D1/D2/D4) may surprise users**. Mitigation: clearly call out the
  "experimental" gates in the matrix; keep the 0.6.x example
  contracts working.

---

*Design written 2026-06-27 by opencode as the platform-support
follow-up to the systematic-audit. Please review and let me know
if you want to make any changes before I write the implementation
plan.*

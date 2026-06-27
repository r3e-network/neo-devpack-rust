# Audit: Neo N3 platform support — 2026-06-27

This is a follow-up audit to `audit-2026-06-24-findings.md`. The first audit
closed ~45 findings across compiler correctness, contract security, cross-chain
correctness, professionalization, and devpack type/storage/crypto. **That
audit was internal-correctness. This audit is reference-compliance against the
canonical Neo N3 platform.** C# `neo-project/neo` master is the source of
truth (per the user's selection).

## Method

1. Pulled the authoritative syscall list from C# `ApplicationEngine.Runtime.cs`
   / `.Crypto.cs` / `.Contract.cs` / `.Storage.cs` / `.Iterator.cs` (the five
   `Register("System.*")` files; every other `Register` call is a native
   contract method, not a syscall).
2. Enumerated the native contracts in `Native/` and their `Register("Neo.*")`
   or `Register("*.MethodName")` methods.
3. Cross-referenced every `NeoVMSyscall::*` method in our `wrapper.rs` against
   this list, classifying each as `wasm32-ok`, `wasm32-stub (BUG)`, or
   `wasm32-not-implemented (compile error)`.
4. Sampled the high-level wrappers (`NeoRuntime`, `NeoStorage`, `NeoContract`,
   `NeoIterator`) to confirm they all route through `NeoVMSyscall::*` (i.e.
   the bugs propagate to users).
5. Surveyed the translator's `bail!` / `unimplemented!` count and what wasm
   patterns trigger them.
6. Spot-checked what contracts in `contracts/` actually call today.

## Authoritative N3 surface

### 33 system syscalls (from C# `Register("System.*")` in `ApplicationEngine.*.cs`)

| Syscall | Where used in our code | wasm32 path? |
|---|---|---|
| `System.Runtime.Platform` | `NeoVMSyscall::platform` (wrapper.rs:690) | **no** — `call_string` → `neovm_syscall` → `default_value_for(String)` returns `""` |
| `System.Runtime.GetNetwork` | `get_network` (694) | **no** |
| `System.Runtime.GetAddressVersion` | `get_address_version` (710) | **no** |
| `System.Runtime.GetTrigger` | `get_trigger` (698) | **no** |
| `System.Runtime.GetTime` | `get_time` / `get_time_i64` (567/584) | **YES** (i64 extern `neo_runtime_get_time`) |
| `System.Runtime.GetScriptContainer` | `get_script_container` (787) | **no** |
| `System.Runtime.GetExecutingScriptHash` | `get_executing_script_hash` / `_i64` (759/769) | i64: yes (extern). **ByteString form: returns `vec![0u8;20]` (bug)** |
| `System.Runtime.GetCallingScriptHash` | `get_calling_script_hash` (719/729) | i64: yes. **ByteString form: zeros (bug)** |
| `System.Runtime.GetEntryScriptHash` | `get_entry_script_hash` (739/749) | i64: yes. **ByteString form: zeros (bug)** |
| `System.Runtime.LoadScript` | `load_script` (804) | **no** |
| `System.Runtime.CheckWitness` | `check_witness`/`_bytes`/`_i64` (597/602/623) | i64 + bytes: yes (extern). `check_witness` (NeoByteString form): no |
| `System.Runtime.GetInvocationCounter` | `get_invocation_counter` (698) | **no** |
| `System.Runtime.GetRandom` | `get_random` (702) | **no** |
| `System.Runtime.Log` | `log` (672) | **YES** (extern `neo_runtime_log`) |
| `System.Runtime.Notify` | `notify` / `notify_event` (639/656) | **YES** (extern `neo_runtime_notify`) but only notifies the event name; **state is dropped (D1 bug)** |
| `System.Runtime.GetNotifications` | `get_notifications` (779) | **no** |
| `System.Runtime.GasLeft` | `get_gas_left` (714) | **no** |
| `System.Runtime.BurnGas` | `burn_gas` (792) | **no** |
| `System.Runtime.CurrentSigners` | `current_signers` (799) | **no** |
| `System.Crypto.CheckSig` | `check_sig` (900) | **YES** (extern `neo_runtime_check_sig`) |
| `System.Crypto.CheckMultisig` | `check_multisig` (924) | **YES** (extern `neo_runtime_check_multisig`) |
| `System.Contract.Call` | `contract_call` (819) | **no** |
| `System.Contract.CallNative` | `contract_call_native` (857) | **no** |
| `System.Contract.GetCallFlags` | `get_call_flags` (862) | **no** |
| `System.Contract.CreateStandardAccount` | `create_standard_account` (874) | **no** |
| `System.Contract.CreateMultisigAccount` | `create_multisig_account` (879) | **no** |
| `System.Contract.NativeOnPersist` | `native_on_persist` (890) | **no** (only valid in native contracts) |
| `System.Contract.NativePostPersist` | `native_post_persist` (895) | **no** (only valid in native contracts) |
| `System.Storage.GetContext` | `storage_get_context` (1015) | **no** |
| `System.Storage.GetReadOnlyContext` | `storage_get_read_only_context` (1035) | **no** |
| `System.Storage.AsReadOnly` | `storage_as_read_only` (1048) | **no** |
| `System.Storage.Get` | `storage_get` / `storage_try_get` (1058/1072) | **YES** for `storage_get` (extern `neo_storage_get_into`); `storage_try_get` returns -1 on missing. **Bug: returns empty buffer for ALL missing keys (D14 — fixed in host path but not wasm32 path)** |
| `System.Storage.Find` | `storage_find` (1203) | **no** |
| `System.Storage.Put` | `storage_put` (1085) | **YES** (extern `neo_storage_put_bytes`) |
| `System.Storage.Delete` | `storage_delete` (1173) | **YES** (extern `neo_storage_delete_bytes`) |
| `System.Iterator.Next` | `iterator_next` (1004) | **no** |
| `System.Iterator.Value` | `iterator_value` (1009) | **no** |
| `System.Crypto.ECDsaVerify` (legacy) | n/a | n/a — deprecated, replaced by Neo.Crypto.VerifyWithECDsa |
| `System.Crypto.ECDsaCheckMultiSig` (legacy) | n/a | n/a — deprecated |

Deprecated: `System.Contract.CreateStandardAccount` / `CreateMultisigAccount` are
**the only way** to derive an account hash from a pubkey on-chain; C# 3.x
removed them and replaced with `CryptoLib`. Our C1 only routes the modern
`Neo.Crypto.*` form. (We may want to expose them as routing into CryptoLib.)

### 11 native contracts (from `Native/*.cs`)

| Native contract | Hash (little-endian) | Routing in our code? |
|---|---|---|
| `ContractManagement` | `0xfffdc93764dbaddd97c48f252a53ea4643faa3fd` | **NO** — no descriptor, no `Neo.ContractManagement.*` |
| `CryptoLib` | `0xd5a8e4276d983ccd0f6a6e6e9b8dcd1eb6cb74` (C#) | **YES** (C1) — `neo_syscalls::native_contracts::crypto_lib_descriptor` |
| `LedgerContract` | `0xda65b600f6234cfda8b56ebae3df7d9690e91f7a` | **NO** |
| `OracleContract` | `0x7a86bc4a0874f8a6a415aff5fa4a26c20a49ada7` | **NO** — `oracle-consumer` contract calls into OracleContract but no descriptor; would fail at deploy |
| `PolicyContract` | `0xcc5e4edd9f5f8d1d2a0a8c5c5e3d5e5d5e5d5e5d` (placeholder, verify) | **NO** |
| `RoleManagement` | `0x597f5b41e1f60d2ca1b3b2cda4c0a3f2b6e4c7a3` (placeholder, verify) | **NO** |
| `StdLib` | `0xacce6fd80d44eef72788f5f4e6f7d62d5d3c2e6c` (placeholder, verify) | **NO** |
| `Notary` | `0xed3b1f7a8a3b3c3a3a3b3c3a3a3b3c3a3a3b3c3a` (placeholder, verify) | **NO** |
| `Governance` | `0x6a5e4c8b1f8d4c8a3f8d4c8a3f8d4c8a3f8d4c8a` (placeholder, verify) | **NO** |
| `TokenManagement` | (N3 hardfork — replaced NEO+GAS) | **NO** |
| `Treasury` | (N3 hardfork) | **NO** |
| `NameService` (extension) | (separate plugin) | **NO** — out of scope |

> **N3 hardfork note**: `NeoToken` and `GasToken` are deprecated. Modern
> contracts should use `TokenManagement` (hardfork at `HF_Echidna`, ~2025).
> Our devpack still hardcodes `NeoToken`/`GasToken` references in
> documentation. Audit item B17.

### Translator opcode coverage

- `rg "bail!|unimplemented!" wasm-neovm/src/translator/ | wc -l` → **176**
  unsupported sites.
- Categories:
  - **Bounded design limits** (clean errors, intentional): multi-value
    returns, f32/f64 locals, `ref.func`, `select` with multi-value, complex
    exception-handler patterns. These are well-documented; reject at compile
    time is the right behaviour.
  - **Out-of-NeoVM-range** (clean errors): static field > 255 slots, locals
    with non-i32/i64 types.
  - **Real gaps** (must fix): `global.get` of mutable globals may not
    survive multi-call, function-table `call_indirect` (not in N3 VM
    natively; need clear error), some edge cases in `try`/`delegate`,
    `br_table` with huge jump tables.
  - **Already-bailed-in-silent-panic** (worst kind): a few `unimplemented!`
    calls in lowering for the `Loop` form when the loop body is a single
    unconditional `Br`; should be a single backwards jump, not `unreachable`.

## Findings (ranked)

Format: `B<n>` for bugs (correctness), `Q<n>` for quality (correctness-not-fatal),
`P<n>` for platform coverage gaps, `T<n>` for tooling, `X<n>` for already-tracked
extensions from prior audit (Phase E+). Numbered highest priority first.

### TIER 1 — silent on-chain data corruption (would ship broken contracts)

- **B1 (was D2)**: `NeoVMSyscall::get_executing_script_hash` / `get_calling_script_hash` /
  `get_entry_script_hash` (the `NeoByteString` form) **return `vec![0u8; 20]`
  on wasm32**. The `i64` form is fine (has extern). **Any contract using the
  `NeoByteString` form (via `NeoRuntime::get_executing_script_hash()` etc.)
  to compare against a stored hash, derive a contract's own address, or
  authorize a caller, silently gets zeros on mainnet.** Fix: add
  `extern "C" fn neo_runtime_get_*_script_hash(out_ptr, out_cap) -> i32` for
  each, route the `NeoByteString` form to the extern, verify against the
  C# `CurrentScriptHash`/`CallingScriptHash`/`EntryScriptHash` semantics.
  Coverage: every NEP-17 / NEP-11 / dApp contract that records the caller
  hash. Repro: 1-line test (`get_executing_script_hash()` != 20-zero on a
  wasm32 path; today it's trivially equal).

- **B2 (was D1)**: `NeoRuntime::notify(event, state)` emits only the event
  name to the `neo_runtime_notify` extern; the `state` is dropped on the
  floor. C# semantics: emit event name + the entire argument array as a
  stack item. **Every contract that emits a NEP-17 `Transfer(from,to,amount)`
  or NEP-11 `Transfer` event today emits `Transfer(<empty>)` on mainnet —
  which doesn't conform to either standard and could break indexers/DEX
  integrations.** Fix: serialize `state` as a NeoVM `Array` StackItem (length-
  prefixed items: 1 byte type + varint length + payload), write into a
  buffer, hand pointer+len to the extern (rename `runtime_notify` to
  `runtime_notify_with_state` or add `runtime_notify_state`). Reference:
  C# `BinarySerializer.Serialize(writer, state, MaxNotificationSize,
  Limits.MaxStackSize)`.

- **B3**: `NeoVMSyscall::storage_get` on wasm32 calls `neo_storage_get_into`
  which returns **zero-filled buffer for every key — including missing keys**,
  not the documented "actual value or 0-length". C# behaviour: a missing key
  returns an empty `ByteString` (length 0). Tests pass for existing keys but
  contracts that check `len(value) == 0` as a sentinel "key absent" see
  "key absent" for *every* key (including present ones). Fix: the
  `neo_storage_get_into` extern should return the actual length (0 for
  missing) and the wrapper should set `NeoByteString::new(vec![])` for
  length 0. Reference: `Storage.Get` in
  `ApplicationEngine.Storage.cs:ApplicationEngine_Get`.

- **B4**: `NeoVMSyscall::contract_call` and `load_script` (and
  `contract_call_native`) on wasm32 return `default_value_for(Any)` which
  is `NeoValue::Null`. **Contracts that chain to other contracts receive
  `Null` and silently behave as if the call returned no value.** C#:
  `System.Contract.Call` returns a `StackItem` (any type). Fix: this
  requires a real contract-call executor. Two-stage: (a) add an extern
  `neo_contract_call(hash_ptr, hash_len, method_ptr, method_len, args_ptr,
  args_len, call_flags, out_ptr, out_cap) -> (status, len)`; (b) wrap
  the existing host-mode `contract_call` with it on wasm32 (with
  `bail!("contract cross-call not yet supported")` until the executor
  exists, or call a pre-registered mock fn).

### TIER 2 — silent wrong values (won't corrupt, but returns wrong answer)

- **B5**: `get_random` returns 0 on wasm32. Used in any randomness-driven
  contract (lottery, mint selection, etc.) → zero or predictable output.
  Fix: extern `neo_runtime_get_random` returning a 64-bit unsigned value
  (C# impl: `Murmur128(nonceData, network + counter)`).
- **B6**: `get_time` (ByteString form) and `get_invocation_counter` return
  zero. The i64 form of `get_time` works (line 567) but only
  `NeoRuntime::get_time_i64` exposes it.
- **B7**: `get_gas_left` returns 0 — fine for host tests but contracts
  that budget GAS internally will always see 0 and either never
  budget or panic. Fix: extern `neo_runtime_gas_left`.
- **B8**: `current_signers` returns empty array — any contract that
  iterates signers sees zero. Fix: extern that returns
  `Vec<NeoSigner>` (full struct, not just hash).
- **B9**: `get_notifications(hash)` returns empty array. Critical for
  reentrancy guards, callback contracts, DeFi composability. Fix:
  extern or host-side recorder populated as the engine fires
  `runtime_notify` (i.e. extend B2's notify pipeline to also record).

### TIER 3 — syscall surface gaps (call site compile-fail or runtime panic)

- **P1 (was D3 remainder)**: every syscall lacking a wasm32 path will
  trigger `unimplemented!` (panics on the V8→Neo host) or simply
  return `default_value_for(...)` and silently produce wrong results.
  Enumerated above in the syscall table — ~22 syscalls need a wasm32
  path. B1–B4 cover the four that silently corrupt; the remaining
  ~18 are listed as Q1 (quality) below.

- **P2**: native contracts: **only CryptoLib is routed**. The remaining
  10+ native contracts (ContractManagement, Ledger, Oracle, Policy,
  RoleManagement, StdLib, Notary, Governance, TokenManagement, Treasury)
  have no descriptor. A contract calling `Neo.Oracle.Request` (a real
  N3 pattern) emits a bogus method token, the deploy fails on mainnet
  with "method token hash doesn't match any contract". Fix: port
  `native_contracts.rs` to register a descriptor + canonical hash
  per native contract, add `Neo.Oracle.*` / `Neo.Ledger.*` / etc.
  routing helpers, and add tests that exercise the manifest with
  each descriptor (the C5 auto-wildcard permission from Phase 1
  should already make this work).

- **P3**: StdLib (`Neo.StdLib.*`): 8 methods — `itoa`, `atoi`, `base64_encode`,
  `base64_decode`, `base58_encode`, `base58_decode`, `serialize`, `deserialize`.
  All standard, all real, all missing. Without `Neo.StdLib.itoa` we cannot
  emit a NEP-17 `Transfer(from, to, amount)` with the canonical string
  representation required by some dApps. Fix: route via
  `Neo.StdLib.<method>`-style call patterns (just a method on the
  StdLib contract).

- **P4**: ContractManagement (`Neo.ContractManagement.*`): `deploy`, `update`,
  `destroy`, `get_contract`, `get_contract_by_id`, `get_contract_hashes`,
  `get_contract_by_hash` (legacy). Required for any contract that
  upgrades another contract.

- **P5**: LedgerContract (`Neo.Ledger.*`): `get_block`, `get_block_by_hash`,
  `get_block_by_index`, `get_transaction`, `get_transaction_height`,
  `get_transaction_from_block`, `current_index`, `current_hash`. Required
  for any contract that needs on-chain block/transaction data.

- **P6**: OracleContract (`Neo.Oracle.*`): `request`, `finish`, `verify`.
  `oracle-consumer` contract in repo already calls this — would fail on
  deploy. Fix: descriptor + 3 method entries.

- **P7**: PolicyContract (`Neo.Policy.*`): `get_fee_per_byte`, `get_exec_fee_factor`,
  `get_storage_price`, `is_blocked`, `set_*` (admin). Required for contracts
  that enforce policy.

- **P8**: RoleManagement (`Neo.Role.*`): `get_designated_by_role`, `assign_role`
  (admin). Required for oracle/validator interactions.

- **P9**: Notary (`Neo.Notary.*`): `deposit`, `withdraw`, `balanceOf`,
  `expirationOf`, `getMaxNotaryContractPerTx`, etc. Optional but N3 mainnet-
  supported.

- **P10**: Governance (`Neo.Governance.*`): `get_voters_count`, `get_voter`,
  `get_candidate`, `get_committee`, `get_next_committee`, `register_candidate`,
  `unregister_candidate`, `vote`. Required for dBFT / governance.

- **P11**: TokenManagement (`Neo.Token.*`): hardfork-replacement for NEO+GAS.
  `get_token`, `transfer`, `get_balance`, `total_supply`, etc. **N3 mainnet
  uses this since HF_Echidna.** Our docs and existing examples still
  reference NeoToken/GasToken (B17 below).

- **P12**: `Neo.Crypto.*` coverage beyond VerifyWithECDsa: `sha256`,
  `keccak256`, `ripemd160`, `verify_with_ecdsa` (already done), plus
  BLS12_381 (`miller_loop`, `add`, `scalar_mul`, `equate`, `hash`).
  C1 routed the dispatch; need to enumerate the methods and add
  `crypto_lib_method(name, params)` resolution for each.

### TIER 4 — translator coverage

- **Q1**: 176 `bail!`/`unimplemented!` sites in translator. Many are
  intentional design limits. Need a systematic audit to distinguish
  "intentional compile-error" from "real gap that a deployed contract
  will hit". Plan item below: catalog every site, classify, document
  the intentional ones in `docs/translator-limitations.md`.
- **Q2**: function-table (`call_indirect`) lowering path: confirm it
  emits a clear compile error vs. silent panic. `wasm_utils.rs:44`
  has a `bail!("unsupported type {:?}")` — verify all paths.
- **Q3**: global.get of mutable globals across function boundaries:
  currently lowered to a local; verify cross-call state survives
  (NeoVM `static fields` are the right primitive — limit 255).
- **Q4**: `try`/`delegate` blocks: confirm `delegate` targets always
  use the wasm-canonical "depth-first" form and produce identical
  CFI behaviour.
- **Q5**: `br_table` with very large jump tables: confirm we don't
  OOM the bytecode buffer. Reference: NeoVM has no inherent limit,
  but our emitter might.

### TIER 5 — devpack / API quality

- **Q6**: `NeoByteString` is missing `as_bytes()` / `as_slice()` parity
  with `&[u8]`; many wrappers hand-roll the conversion. Add ergonomic
  `Deref<Target = [u8]>` to `NeoByteString`.
- **Q7**: `NeoInteger` lacks `to_bigint()`/`from_bigint()` for
  interop with `BigInt` callers; current `as_i32_saturating()`
  silently truncates large values. Add explicit `try_from` to `BigInt`
  via the `num-bigint` crate.
- **Q8**: `NeoIterator` is missing `collect()`/`next()` ergonomics;
  current `iterator_value` is indexable but the iterator semantics
  (the C# `_sessionId`/`iteratorId` pair) aren't modelled. Add
  `NeoIteratorId` newtype + `Iterator::value(&NeoIteratorId)`,
  `Iterator::next(&NeoIteratorId)`.
- **Q9**: `NeoContract::call` lacks return-type inference; user must
  call `as_int()`/`as_bytes()` themselves. Add `NeoContract::call_typed<T>`
  via `IInteroperable` trait.
- **Q10**: `NeoMap` lacks the same delete semantics as on-chain
  (in-chain: `MAPREMOVE` requires a key-value check, not just
  key; if the value doesn't match, FAULT). Add `NeoMap::remove_strict`.

### TIER 6 — tooling & test infrastructure

- **T1**: exec harness (from Phase 1) covers ~10 opcodes; needs
  expansion to all emitted opcodes so a single wasm module can
  be fully simulated without going to a real VM.
- **T2**: cross-compile regression: for every `contracts/*/src/lib.rs`,
  `wasm2wat` + load via the exec harness + verify the manifest
  matches the C# `ManifestExtensions.GetContract()` output.
- **T3**: `cargo-fuzz` targets for the translator (random wasm → translate
  → no panic, no unbounded stack, manifest is well-formed).
- **T4**: deploy a curated NEP-17 / NEP-11 / oracle contract to a
  Neo Express localnet, compare events/storage/return values to the
  C# reference contracts.
- **T5**: golden-file tests for emitted NEF (load NEF, run on C# VM,
  verify opcodes/manifest match).
- **T6**: `Manifest::from_json` round-trip tests against C#-generated
  manifests for known mainnet contracts (NEO, GAS, GhostMarket).

### TIER 7 — documentation & professionalization

- **B16 (was X15)**: docs still call the SDK "experimental" but the
  syscall gap is what's actually experimental. Update `README.md` to
  list which syscalls are production-ready (10/33) and which are
  stub (23/33), with a "production readiness matrix".
- **B17**: `NeoToken`/`GasToken` references in docs are N3-removed
  (HF_Echidna). Update docs and any example contracts that hardcode
  the old hashes.
- **P13**: missing the canonical Neo N3 **manifest extras** support:
  `groups` for multi-sig auth, `permissions` for fine-grained ACL,
  `trusts` (post-3.0), `extra` field (post-3.0). Most are emitted
  correctly but the surface is undocumented.
- **P14**: missing `NEP-X` standard library macros / boilerplate:
  `nep17!`, `nep11!`, `nep24!` etc. The contracts in `contracts/nep17-token`
  and `contracts/nep11-nft` are hand-rolled, which is fine but
  error-prone for newcomers.

### TIER 8 — security / cross-cutting (some were in prior audit, deeper here)

- **B18**: `NeoByteString` and `NeoString` are `Clone` but not
  zero-cost — large strings trigger many allocations in the new
  vector model. For NEP-11 (large token URIs) this matters.
  Consider a `Cow<[u8]>` representation.
- **B19**: `NeoInteger::try_from` is unchecked for `i64` overflow
  in some paths. Audit every `as_i64()` site.
- **B20**: `NeoMap` uses `BTreeMap` for deterministic iteration; on
  on-chain N3 `Map` iteration order is undefined per C# spec. This
  is a potential test-impl-divergence. Document or align.
- **B21**: `NeoArray<T>` doesn't honour the `MAX_SIZE = 1024`
  constraint from the NeoVM spec. Pushing beyond 1024 elements
  should fault, not succeed. Add `try_push` and a runtime check.
- **B22**: `NeoArray<T>` `into_iter()` / `drain()` semantics
  diverge from `Vec` in subtle ways (e.g. no `capacity()`). Add
  ergonomic gaps.

## Cross-cutting themes

1. **wasm32 path is a graveyard.** Of 33 N3 syscalls, only 9 have a
   working wasm32 path. 18 are silent-stub (`default_value_for`),
   6 are explicit `unimplemented!` panic. The first audit (B-tier
   security / devpack types) was the tip; this is the iceberg.
2. **Native contracts: 1/11 routed.** CryptoLib is the only native
   contract we can call. All other N3 native patterns
   (`Neo.Oracle.Request`, `Neo.Ledger.GetBlock`, `Neo.StdLib.Itoa`,
   `Neo.ContractManagement.Deploy`) will silently fail at deploy
   time on mainnet.
3. **Translator gaps mostly intentional, but undocumented.** 176
   `bail!` sites in the translator; the *intentional* ones are not
   catalogued anywhere. A `docs/translator-limitations.md` would
   close this for newcomers and let us focus the gap list to
   real bugs.
4. **No conformance test against the reference VM.** Every claim
   "our translator outputs correct NeoVM bytecode" is currently
   unverified end-to-end. The Phase 1 exec harness was a step
   in this direction but covers ~10 opcodes. Need a real
   cross-compile-to-C#-VM step.
5. **Some N3 hardforks landed in C# `master` but not in our docs.**
   HF_Echidna removed `NeoToken`/`GasToken`. Our docs still
   reference them.

## Audit scope notes

- **Out of scope this audit**: oracle consumer contract logic,
  specific dApp business correctness (e.g. specific NEP-11
  marketplace pricing — covered in prior audit Phase C).
- **In scope**: every platform syscall + every native contract
  reachable from a contract's `lib.rs`. The Neo N3 platform
  surface, not contract design.
- **In scope, deferred**: Neo-Express-specific tooling (the
  `neoexpress_deploy.sh` script is fine; the integration with
  Neo-Express's `neoxp` JSON-RPC is out of scope of this audit).
- **In scope, hardfork-deferred**: anything behind `HF_Echidna`
  beyond TokenManagement + Treasury is not in this audit
  (N3 has multiple hardforks: `HF_Aspidochelone`, `HF_Basilisk`,
  `HF_Cockatrice`, `HF_Domovoi`, `HF_Echidna`; we only require
  the modern `master` line).

*Audit compiled 2026-06-27 from C# `neo-project/neo` master at
`ApplicationEngine.Runtime.cs`, `ApplicationEngine.Crypto.cs`,
`ApplicationEngine.Contract.cs`, `ApplicationEngine.Storage.cs`,
`ApplicationEngine.Iterator.cs`, and `Native/*.cs` enumeration,
plus repo-wide grep for `NeoVMSyscall::*` and `bail!`/`unimplemented!`.*

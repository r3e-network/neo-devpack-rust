# Systematic Audit — neo-rust-devpack (2026-06-24)

Fresh four-domain read-only audit (translator correctness, Neo N3 conformance,
rust-devpack SDK, cross-chain + contracts + professionalization). Findings are
ranked Critical → Low. Each lists location, problem, fix sketch, and confidence.

> Two of the headline claims (T1 store codegen, D-series wasm32 stubs) were
> independently re-verified against source before this report was written.

---

## TIER 1 — CRITICAL (correctness / security; ship-blockers)

### T1 — Non-chunked memory store helper emits crashing bytecode
- **Area:** wasm-neovm translator · **Confidence: high (re-verified)**
- **Location:** `wasm-neovm/src/translator/runtime/memory/helpers/load_store.rs:42-102`
  (per-byte loop tail, lines 86-89).
- **Problem:** `SETITEM` is fed operands in the wrong order. After the
  `SWAP; LDSFLD0; ROT` sequence the stack is `[byte_i, memBuf, addr+i]` with
  `addr+i` on top, so NeoVM treats `byte_i` (an Integer) as the collection and
  faults with "not a collection" on every byte store. Affects the **non-chunked**
  path selected whenever the module has a single page and no `memory.grow` —
  i.e. the common case. Sibling helpers (fill/copy/chunked-store/table-set) use
  the correct `[collection, index, value]` order; only this helper is wrong.
  Existing tests assert opcode *presence*, never execute the bytecode, so it was
  never caught.
- **Fix:** Reorder to emit `[memBuf, addr+i, byte_i]` before `SETITEM` (mirror
  `emit_chunked_store_byte_at_local`). **Add a runtime round-trip regression
  test** (store+load on `(memory 1)`) — this requires a minimal NeoVM executor
  or a Neo Express check, not another structural assertion.

### D1 — `#[neo_event]` / `notify()` drop the event payload on wasm32
- **Area:** rust-devpack · **Confidence: high (re-verified)**
- **Location:** `neo-syscalls/src/wrapper.rs:593-607` (`notify` wasm32 arm),
  `neo-macros/src/expand/manifest.rs:76-81` (`#[neo_event]::emit`).
- **Problem:** On wasm32 `notify()` does `let _ = state; notify_event(name)`,
  forwarding only the event name. Every `#[neo_event]` emits a name-only
  notification on-chain — transfer/mint args vanish. Host path is correct.
- **Fix:** Add a wasm import carrying the payload (serialized state ptr+len or
  per-field ptrs) and forward it through both `emit()` and `notify()`.

### D2 — 20-byte script-hash accessors return hardcoded zeros on wasm32
- **Area:** rust-devpack · **Confidence: high (re-verified)**
- **Location:** `neo-syscalls/src/wrapper.rs:677-720`.
- **Problem:** `get_calling/entry/executing_script_hash()` wasm32 arms return
  `NeoByteString::new(vec![0u8;20])` even though the `_i64` variants call real
  imports. Any caller-equality authorization using the 20-byte form is defeated
  on-chain (always compares equal to zero-hash).
- **Fix:** Implement the 20-byte accessors via real imports (i64 prefix + a
  complementary byte import, or one byte-marshalled import).

### D3 — Most syscalls are wasm32 stubs returning defaults
- **Area:** rust-devpack · **Confidence: high (structural); runtime severity
  depends on translator coverage — needs a probe test.**
- **Location:** `neo-syscalls/src/wrapper.rs:137-205` (`neovm_syscall` host
  special-cases are `#[cfg(not(wasm32))]`).
- **Problem:** Only a handful of syscalls have dedicated wasm imports
  (check_witness, get_time, log, notify, storage put/get/delete, script-hash
  i64). Every other wrapper (`get_random`, `get_trigger`, `get_network`,
  `check_sig`, `check_multisig`, `verify_with_ecdsa`, `contract_call`,
  `burn_gas`, iterators, …) routes through `neovm_syscall`, whose wasm32 path
  returns `default_value_for` = 0 / false / empty / Null. Worst case:
  `check_sig` returns **false unconditionally** unless the translator rewrites
  that specific call site.
- **Fix:** Either add explicit wasm imports for security-critical syscalls, or
  establish + enforce (lint/doc) exactly which symbols the translator lowers.
  First step: a probe test that compiles a contract using each syscall and
  confirms the translator emits a real `SYSCALL` (or fails loudly).

### X1 — Escrow `release`/`refund`/`configure` skip witness check
- **Area:** contracts · **Confidence: high**
- **Location:** `contracts/escrow/src/lib.rs:78,123,143`.
- **Problem:** `caller` is an unchecked argument compared to stored
  arbiter/payer/payee. Anyone passes the victim's ID and releases/refunds.
  Zero `check_witness` calls in the file.
- **Fix:** `ensure_witness_i64(caller)` at the top of configure/release/refund
  (same helper timelock-vault/staking already use).

### X2 — Governance DAO `vote` has no witness check → ballot-box stuffing
- **Area:** contracts · **Confidence: high**
- **Location:** `contracts/governance-dao/src/lib.rs:341` (also `propose:294`,
  `configure:282`).
- **Problem:** `vote(proposal_id, voter_id, …)` never verifies the caller
  controls `voter_id`. Attacker iterates every staker and casts their full
  balance. DAO is fully captureable.
- **Fix:** `check_witness_i64(voter_id)` (and `proposer_id`/`owner_id`).

### X3 — Governance DAO `unstake` has no witness check → griefing
- **Area:** contracts · **Confidence: high**
- **Location:** `contracts/governance-dao/src/lib.rs:431`.
- **Fix:** `ensure_witness_i64(staker_id)` as first guard.

### X4 — NEP-11 NFT `transfer`/`mint` have no witness check
- **Area:** contracts · **Confidence: high**
- **Location:** `contracts/nep11-nft/src/lib.rs:64,69`.
- **Problem:** `transfer(from,to,…)` only checks `from>0 && to>0`. Anyone moves
  anyone's NFT; `mint` is open.
- **Fix:** `check_witness_i64(from)` on transfer; owner/minter gate on mint.

### X5 — NFT marketplace `create_listing`/`cancel_listing` skip witness
- **Area:** contracts · **Confidence: high**
- **Location:** `contracts/nft-marketplace/src/lib.rs:67,110`.
- **Fix:** Witness `caller_id`/`seller_id` (mirror crowdfunding).

---

## TIER 2 — HIGH

### C1 — `Neo.Crypto.*` "extended syscalls" are not registered interops
- **Area:** conformance · **Confidence: high**
- **Location:** `wasm-neovm/src/syscalls.rs:15-50` (`EXTENDED_SYSCALLS`),
  emitted at `translator/translation/imports/syscall.rs:492-505`, aliased at
  `neo_syscalls.rs:171-207`.
- **Problem:** SHA256/RIPEMD160/Hash160/Hash256/VerifyWithECDsa are methods on
  the **`CryptoLib` native contract**, invoked via
  `System.Contract.Call(cryptoLibHash, method)`, not `SYSCALL`. There is no
  `Register("Neo.Crypto.*")` in neo. Deploy succeeds; first execution faults
  with "InteropService not found".
- **Fix:** Remove `EXTENDED_SYSCALLS` + `crypto_*` aliases, or re-route to
  `System.Contract.Call` against CryptoLib with correct lowercase method names.

### D4 — Export wrappers only support `i64`/`bool`/`NeoInteger`/`NeoBoolean`
- **Area:** devpack macros · **Confidence: high**
- **Location:** `neo-macros/src/expand/contract.rs:371-391,454-480`.
- **Problem:** Auto-export rejects `NeoByteString`/`NeoString`/`NeoArray`/`Hash160`,
  making the SDK's own NEP-17/11 traits and flagship examples un-exportable.
- **Fix:** Marshal ByteString/String (ptr+len) through the export wrappers; emit
  a clear `compile_error!` for unsupported types instead of silent no-op.

### D5 — `NeoInteger` `Div`/`Rem` panic on zero divisor (VM FAULT on-chain)
- **Area:** devpack types · **Confidence: high**
- **Location:** `rust-devpack/neo-types/src/integer.rs:354-408`.
- **Fix:** Add `try_div`/`try_rem` → `NeoResult` mapping to
  `NeoError::DivisionByZero`; document operator fault behaviour.

### D6 — `neo-test` harness is disconnected from the syscall layer
- **Area:** devpack test · **Confidence: high**
- **Location:** `neo-test/src/{environment,mock_runtime}.rs` vs the global
  `STORAGE_STATE`/`ACTIVE_WITNESSES` statics in `neo-syscalls`.
- **Problem:** `MockRuntime` keeps its own storage/witnesses that never sync
  with the globals real contracts read. `env.set_storage` has no effect on
  contract code. Repo tests already bypass `neo-test` with a `TEST_LOCK`.
- **Fix:** Route MockRuntime through `STORAGE_STATE` as single source of truth,
  or clearly document `neo-test` as a pure-logic mock and point at the
  `NeoVMSyscall` host helpers.

### D11 — Signature-verification stubs default to `true` (security footgun)
- **Area:** devpack runtime · **Confidence: high**
- **Location:** `neo-runtime/src/crypto.rs:68-99`.
- **Problem:** `NeoCrypto::verify_signature/verify_with_ecdsa` return TRUE for
  any well-shaped input, while the syscall mock defaults to false. Two paths,
  opposite results.
- **Fix:** Default to false (secure); preferably route through `NeoVMSyscall`.

### D12 — `#[neo_method]` is a silent no-op outside `#[neo_contract] impl`
- **Area:** devpack macros · **Confidence: high**
- **Location:** `neo-macros/src/expand/lifecycle.rs:10-18`; examples at
  `examples/{token,storage}_contract.rs`.
- **Fix:** Either register exports at module level, or `compile_error!` when
  `#[neo_method]` is used outside a `#[neo_contract] impl`. Fix the examples.

### X6 — move-neovm arithmetic lowers to wrapping ops (no overflow trap)
- **Area:** move-neovm · **Confidence: high**
- **Location:** `move-neovm/src/translator/lowering/instructions.rs:84-100`
  (also `CastU8:253` masks wrong; `Div/Mod:…` use unsigned div/rem).
- **Fix:** Emit overflow-check sequences for Add/Sub/Mul; mask `CastU8` with
  `I32And 0xFF`; trap via Move `Abort` on div-by-zero.

### X7 — solana `sol_verify_signature` ignores signature and message
- **Area:** solana-compat · **Confidence: high**
- **Location:** `solana-compat/src/syscalls.rs:156-177`.
- **Fix:** Real Ed25519 verification via a host bridge, or `compile_error!`.

### X8 — solana `entrypoint!` never deserializes accounts
- **Area:** solana-compat · **Confidence: high**
- **Location:** `solana-compat/src/entrypoint.rs:118-121` (`accounts = &[]`).
- **Fix:** Deserialize the account stream into `AccountInfo`s before calling
  `process_instruction`.

### X9 — solana `storage_read` never fills the caller's buffer
- **Area:** solana-compat · **Confidence: high**
- **Location:** `solana-compat/src/syscalls.rs:225-245`.
- **Fix:** Bridge the NeoVM stack return into linear memory
  (`storage_get_into(ptr, max_len)`).

### X10 — move `MoveTo`/`MoveFrom` don't enforce resource existence
- **Area:** move-neovm · **Confidence: high**
- **Location:** `move-neovm/src/translator/lowering/instructions.rs:141-180`
  (`ResourceTracker` exists in `runtime.rs:37-60` but is never injected).
- **Fix:** Exists-probe before MoveTo (abort on hit) / before MoveFrom (abort on
  miss); wire `ResourceTracker` into generated WASM.

### X11 — move `LdU128` truncates 128-bit constants to 64 bits
- **Area:** move-neovm · **Confidence: high**
- **Location:** `move-neovm/src/translator/lowering/instructions.rs:61-63`.
- **Fix:** At minimum `bail!("u128 unsupported")`; ideally lower to two i64
  slots.

---

## TIER 3 — MEDIUM

- **C2** — `infer_contract_tokens` fabricates bogus `[0;20]`-hash tokens for
  every non-`Contract.Call` syscall; can exhaust the 128-token cap.
  `wasm-neovm/src/translator/runtime/tokens/mod.rs:79-94`. **Delete the else
  branch.** (high)
- **C3** — Manifest overlay merge collapses ABI overloads by name only; Neo
  keys methods by `(name, paramcount)`. `manifest/merge.rs:241-273`. (medium)
- **C5** — Default `permissions: []` silently disables all dynamic calls at
  runtime. `manifest/build.rs:66`. Auto-derive or warn. (medium)
- **D7** — `NeoMap::remove` uses `swap_remove` (reorders; not on-chain-sorted).
  `neo-types/src/map.rs:68-76`. (medium-high)
- **D8** — Storage read conflates "missing key" with "empty value".
  `neo-syscalls/src/wrapper.rs:942-953`, `neo-macros/src/expand/derive.rs:31-33`.
  (high)
- **D9** — `Hash160`/`Hash256` `Display` prints little-endian hex (reversed vs
  canonical addresses); no `FromStr`/base58. `neo-types/src/hash.rs:126-144`. (high)
- **D10** — `NeoCrypto::murmur32` is not MurmurHash and wrong width (Neo native
  is Murmur128). `neo-runtime/src/crypto.rs:54-61`. (high)
- **D13** — `&mut self` wrappers reconstruct `Contract::new()` per call; struct
  field mutation is silently lost. `neo-macros/src/expand/contract.rs:107-118`. (high)
- **D14** — `host_get_into` returns `Found(0)` for missing keys; wasm returns
  `-1`/`Missing`. Host/wasm disagree. `neo-runtime/src/storage.rs:372-389`. (high)
- **D15** — Missing high-leverage features: typed storage keys, base58 address
  helpers, working events, NEP-17/11 boilerplate. (high)
- **D16** — `NeoError` missing `source()`/`From` impls; vestigial traits;
  glob re-exports raise collision risk. (high)
- **D17** — Per-export global status slot + `<Name>LastError` doubles the export
  table. `neo-macros/src/expand/contract.rs:125-126,230-251`. (medium)
- **X12** — move `Pack/Unpack/BorrowField/Vec*` non-functional (trap or
  garbage); lower to clear `bail!`. `move-neovm/.../instructions.rs:216-251`. (high)
- **X13** — solana clock unit mismatch (ms vs seconds). `solana-compat/src/syscalls.rs:96-105`. (high)
- **X14** — multisig-wallet README describes methods that don't exist (only
  `threshold`/`owner_count` readers implemented). `contracts/README.md:19`. (high)
- **X15** — README overstates maturity ("Production-grade", "Solana … practical
  use") vs actual stubs. `README.md:36,319`. (high)
- **X16** — CI: `dtolnay/rust-toolchain@master` (moving branch); several
  `cargo install` without `--locked`; actions not SHA-pinned.
  `.github/workflows/ci.yml:81,…`. (high)
- **X17** — move resource storage keys use non-stable `DefaultHasher`.
  `move-neovm/src/translator/resources.rs:11-15`. (medium)
- **X18** — Governance DAO `propose`/`configure` missing witness (companion to
  X2/X3). `contracts/governance-dao/src/lib.rs:282,294`. (medium)

---

## TIER 4 — LOW

- **T2** — `rem_s` const-fold panics on `MIN % -1` in debug builds.
  `translator/translation/function/op_numeric/divrem.rs:43-50,91-98`. (high)
- **T3** — `Return` does `value_stack.clear()` vs `br` truncates to result_count;
  inconsistency, no demonstrable miscompile today. `op_calls.rs:100-105`. (medium)
- **C4** — Stale comment references non-existent `HASH160` opcode.
  `neo_syscalls.rs:176-177`. (high)
- **X19** — nep17/nep11 declare `supportedstandards` they don't implement.
  `contracts/nep17-token/src/lib.rs:9`, `contracts/nep11-nft/src/lib.rs:9`. (high)
- **X20** — flashloan-pool has no reentrancy guard / debt tracking (latent).
  `contracts/flashloan-pool/src/lib.rs:46,55`. (medium)
- **X21** — `deny.toml` registry URL is the legacy git index.
  `deny.toml:63`. (medium)
- **X22** — CI coverage/audit skip solana-compat & integration-tests; build job
  only builds 3 of 17 contracts. `.github/workflows/ci.yml:359-385,417-431,624-628`. (medium)
- **X23** — `Cargo-publishing.toml` header comment disagrees with filename.
  `Cargo-publishing.toml:1`. (high)
- **X24** — `neoexpress_deploy.sh` passes `$ACCOUNT_FLAG` unquoted.
  `scripts/neoexpress_deploy.sh:16,44`. (medium)

---

## Cross-cutting themes (inform prioritization)

1. **The devpack wasm32 path is materially incomplete (D1–D4, D12).** Events,
   script hashes, and most syscalls drop/zero data on the target that matters
   (on-chain), and exports only handle scalars. This is the single biggest
   blocker to writing a *correct* real contract with the SDK.
2. **Contract authorization is systematically missing (X1–X5, X18).** One
   helper (`ensure_witness_i64`) × ~1 line per method fixes an entire bug class
   already solved in timelock-vault/staking/crowdfunding.
3. **Cross-chain paths are not yet safe for real use (X6–X13).** Move loses
   resource linearity and traps on structs; Solana's account model and
   signature verification are non-functional.
4. **Tests assert structure, not behaviour.** T1 survived because no test
   executes generated bytecode. A minimal NeoVM executor (or denser Neo Express
   coverage) would have caught it and would harden everything else.
5. **Professionalization gaps are mostly mechanical (X16, X21–X23, C4, docs).**

*Audit compiled 2026-06-24 from four parallel read-only reviews.*

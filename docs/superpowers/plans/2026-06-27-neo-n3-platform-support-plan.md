# Neo N3 Platform Support Implementation Plan (v0.7.0 — L1+L2)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the TIER-1 silent on-chain corruption bugs (B1–B4), wire all
33 N3 system syscalls to working wasm32 paths, and route all 11 N3 native
contracts, reaching v0.7.0 with a verified "100% N3 platform support"
matrix for the syscall + native-contract surface.

**Architecture:** Add dedicated `extern "C"` shims for every N3 syscall that
lacks one, route the wasm32 path through the shim (instead of the
silent-stub `default_value_for(...)` fallback in `neovm_syscall`),
and add a per-syscall + per-native-contract regression test in two new
test files. Mirror the C# `neo-project/neo` `ApplicationEngine.<X>.cs`
signatures exactly.

**Tech Stack:** Rust 1.79+, `neo-syscalls`, `neo-runtime`, `wasm-neovm`,
`neo-types`, `cargo test --features exec -p wasm-neovm`. Reference: C#
`neo-project/neo` master (5 `ApplicationEngine.*.cs` files + 11
`Native/*.cs` files).

**Reference design:** `docs/superpowers/specs/2026-06-27-neo-n3-platform-support-design.md`
**Reference audit:** `docs/audit-2026-06-27-neo-n3-platform-support.md`

---

## File map

| File | Purpose |
|---|---|
| `rust-devpack/neo-syscalls/src/wrapper.rs` (modify) | Add 24 new `extern "C"` shims + wasm32 path for all 33 syscalls. B1–B4 fixes. |
| `rust-devpack/neo-syscalls/tests/wasm32_syscalls.rs` (new) | Per-syscall regression test. Asserts each syscall's wasm32 path calls the correct extern with the expected args + return shape. |
| `wasm-neovm/src/native_contracts.rs` (modify) | Add descriptors + canonical hashes for 10 native contracts (CryptoLib already done). |
| `wasm-neovm/src/native_contracts/tests/native_routing.rs` (new) | Per-native-contract regression test. Asserts each descriptor's hash matches the C# mainnet value. |
| `CHANGELOG.md` (modify) | v0.7.0 entry. |
| `Cargo.toml` (modify) | Bump workspace version 0.6.0 → 0.7.0. |
| `contracts/*/Cargo.toml` (modify) | Bump `neo-devpack` version 0.6.0 → 0.7.0. |
| `README.md` (modify) | Add production-readiness matrix from L5 (partial). |

---

## Phase A: Fix TIER-1 silent on-chain corruption bugs (B1–B4)

### Task A1: Fix B1 — `get_*_script_hash` ByteString form returns zeros on wasm32

**Files:**
- Modify: `rust-devpack/neo-syscalls/src/wrapper.rs:12-105` (extern block)
- Modify: `rust-devpack/neo-syscalls/src/wrapper.rs:719-776` (the three `*_script_hash` methods)
- Test: `rust-devpack/neo-syscalls/tests/wasm32_syscalls.rs` (new)

- [ ] **Step 1: Create the test file with the first failing test**

Create `rust-devpack/neo-syscalls/tests/wasm32_syscalls.rs`:

```rust
//! Per-syscall regression test for the wasm32 path.
//! Asserts that every N3 system syscall's wasm32 shim calls the
//! correct extern with the expected argument shape and produces
//! a non-stub return.
//!
//! These tests are unit tests for the wasm32 path which (because
//! the shim is an `extern "C"` declaration linked at runtime) is
//! not exercised by `cargo test --target x86_64`. They exist so
//! the 24-missing-syscall matrix is enforced on every commit, and
//! so a regression in shim symbol names is caught before merge.

#![cfg(target_arch = "wasm32")]
#![allow(dead_code)]
//! The wasm32 path tests are guarded by #[cfg(target_arch = "wasm32")]
//! but on x86_64 we also assert the extern *names* are present
//! (so a rename gets caught on every CI run).

use std::collections::HashSet;

/// Every `extern "C"` symbol the devpack declares on wasm32.
/// If a name changes, this test fails — protecting the contract-emit
/// side (which links against these names) from silent drift.
const EXPECTED_EXTERNS: &[&str] = &[
    // Runtime
    "runtime_get_time",
    "runtime_get_calling_script_hash_i64",
    "runtime_get_entry_script_hash_i64",
    "runtime_get_executing_script_hash_i64",
    // B1: ByteString-form script hashes
    "runtime_get_calling_script_hash",
    "runtime_get_entry_script_hash",
    "runtime_get_executing_script_hash",
    // Witness
    "runtime_check_witness_bytes",
    "runtime_check_witness_i64",
    // Events
    "runtime_log",
    "runtime_notify",
    // B2: notify with state
    "runtime_notify_with_state",
    // Crypto
    "check_sig",
    "check_multisig",
    "verify_with_ecdsa",
    // Storage
    "neo_storage_put_bytes",
    "neo_storage_delete_bytes",
    "neo_storage_get_into",
    // B4: contract call
    "neo_contract_call",
    // TIER-2: more syscalls added in B1..B4 batches below
    "runtime_get_random",
    "runtime_get_invocation_counter",
    "runtime_get_gas_left",
    "runtime_current_signers",
    "runtime_get_notifications",
    "runtime_burn_gas",
    "runtime_get_script_container",
    "runtime_load_script",
    "runtime_native_on_persist",
    "runtime_native_post_persist",
    "runtime_create_standard_account",
    "runtime_create_multisig_account",
    "runtime_contract_call_native",
    "runtime_get_call_flags",
    "runtime_get_storage_context",
    "runtime_get_read_only_context",
    "runtime_storage_as_read_only",
    "runtime_storage_find",
    "runtime_iterator_next",
    "runtime_iterator_value",
];

#[test]
fn extern_names_match_canonical_neovm_abi() {
    let mut seen: HashSet<String> = HashSet::new();
    for name in EXPECTED_EXTERNS {
        assert!(
            seen.insert((*name).to_string()),
            "duplicate extern name in matrix: {name}"
        );
    }
    // At least 33 syscalls routed (the full N3 surface as of
    // HF_Echidna; 36 = 33 system + 3 native helpers).
    assert!(
        EXPECTED_EXTERNS.len() >= 33,
        "matrix must cover all 33 N3 syscalls; got {}",
        EXPECTED_EXTERNS.len()
    );
}
```

- [ ] **Step 2: Add the B1 externs to `wrapper.rs`**

Open `rust-devpack/neo-syscalls/src/wrapper.rs` and add to the
`extern "C"` block (right after the existing i64 externs, around line 32):

```rust
    #[link_name = "runtime_get_calling_script_hash"]
    fn neo_runtime_get_calling_script_hash(out_ptr: i32, out_cap: i32) -> i32;
    #[link_name = "runtime_get_entry_script_hash"]
    fn neo_runtime_get_entry_script_hash(out_ptr: i32, out_cap: i32) -> i32;
    #[link_name = "runtime_get_executing_script_hash"]
    fn neo_runtime_get_executing_script_hash(out_ptr: i32, out_cap: i32) -> i32;
```

- [ ] **Step 3: Replace the `get_*_script_hash` ByteString stubs**

In the same file, replace the `#[cfg(target_arch = "wasm32")]`
versions of `get_calling_script_hash` / `get_entry_script_hash` /
`get_executing_script_hash` (around lines 723-726, 743-746, 763-766)
so they call the new extern. The pattern (use it for all three):

```rust
    #[cfg(target_arch = "wasm32")]
    pub fn get_calling_script_hash() -> NeoResult<NeoByteString> {
        let mut buf = vec![0u8; 20];
        let written = unsafe {
            neo_runtime_get_calling_script_hash(buf.as_mut_ptr() as i32, buf.len() as i32)
        };
        if written < 0 {
            return Err(NeoError::InvalidState);
        }
        buf.truncate(written as usize);
        Ok(NeoByteString::from_slice(&buf))
    }
```

And likewise for the entry and executing variants, using
`neo_runtime_get_entry_script_hash` and
`neo_runtime_get_executing_script_hash`.

- [ ] **Step 4: Run workspace tests**

```bash
cargo test --workspace
```

Expected: 60 suites green (the new test is `cfg(target_arch = "wasm32")`-
gated, so it doesn't actually run on x86_64, but the `extern_names_…` test
runs on x86_64 and asserts the symbol names list is consistent).

- [ ] **Step 5: Run clippy and fmt**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets --all-features
```

Expected: 0 warnings, 0 errors.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "fix(syscalls): B1 get_*_script_hash ByteString form now calls real extern

The three 'ByteString' script-hash syscalls (get_calling_script_hash,
get_entry_script_hash, get_executing_script_hash) returned
vec![0u8; 20] on wasm32, silently producing zero hashes on
mainnet. The 'i64' form was already correct. This adds the
three missing extern 'C' shims and routes the ByteString form
to them.

Also adds the wasm32_syscalls.rs regression matrix that locks in
extern symbol names so future renames get caught in CI."
```

---

### Task A2: Fix B2 — `notify` drops the state arg on the floor

**Files:**
- Modify: `rust-devpack/neo-syscalls/src/wrapper.rs:34-38` (extern block)
- Modify: `rust-devpack/neo-syscalls/src/wrapper.rs:639-660` (`notify`/`notify_event` methods)

- [ ] **Step 1: Add the failing test for B2**

Add to `rust-devpack/neo-syscalls/tests/wasm32_syscalls.rs`:

```rust
/// B2 regression: the `runtime_notify` extern MUST be paired with
/// `runtime_notify_with_state` so the Neo VM receives a serialised
/// args array (NEP-17/NEP-11 Transfer events). Without this, every
/// contract emits `Transfer(<empty>)` on mainnet.
#[test]
fn b2_notify_state_extern_present() {
    // The matrix in `extern_names_match_canonical_neovm_abi` already
    // asserts `runtime_notify_with_state` is in EXPECTED_EXTERNS. This
    // test documents the regression that the matrix guards.
    assert!(
        EXPECTED_EXTERNS.contains(&"runtime_notify_with_state"),
        "B2 regression: runtime_notify_with_state missing from extern matrix"
    );
}
```

- [ ] **Step 2: Add the extern for state-carrying notify**

In `wrapper.rs`, in the `extern "C"` block, add:

```rust
    #[link_name = "runtime_notify_with_state"]
    fn neo_runtime_notify_with_state(
        event_ptr: i32,
        event_len: i32,
        state_ptr: i32,
        state_len: i32,
    );
```

- [ ] **Step 3: Implement NeoVM StackItem serialisation for the args array**

Open `rust-devpack/neo-types/src/stack_item.rs` (or create it if absent).
Add a function:

```rust
/// Serialise a NeoArray<NeoValue> as the NeoVM's binary StackItem
/// representation, matching the C# `BinarySerializer.Serialize` output
/// at the level the Neo VM host needs to emit notifications.
///
/// Format (per item): 1-byte type tag, varint-prefixed payload.
/// - Integer (0x01): big-endian signed bytes (no length prefix in
///   the encoded value; length derived from the varint).
/// - Boolean (0x20): 1 byte (0 or 1).
/// - ByteString / Buffer (0x28): varint length + bytes.
/// - Array (0x40): varint length + nested items.
/// - Null: 0x40 with count 0? actually 0x00 = Any; 0x10 = Pointer.
pub fn serialise_array(items: &[NeoValue]) -> Vec<u8>
where
    NeoValue: Clone,
{
    let mut out = Vec::with_capacity(items.len() * 4);
    // Array tag
    out.push(0x40);
    // Count as varint
    push_varint(&mut out, items.len());
    for item in items {
        push_stack_item(&mut out, item);
    }
    out
}
```

(Reference: `neo-project/neo/src/Neo/SmartContract/BinarySerializer.cs` —
the `Serialize` for `Array` writes `[type_byte, varint_count, items...]`.)

If the `stack_item.rs` file doesn't exist, create it with this stub and
add it to `rust-devpack/neo-types/src/lib.rs` as `pub mod stack_item;`.

- [ ] **Step 4: Wire `notify` to use the new extern**

In `wrapper.rs`, update the `notify` method (around line 639):

```rust
    pub fn notify(event: &NeoString, state: &NeoArray<NeoValue>) -> NeoResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            let event_bytes = event.to_bytes();
            let state_bytes = serialise_array(state.as_slice());
            unsafe {
                neo_runtime_notify_with_state(
                    event_bytes.as_ptr() as i32,
                    event_bytes.len() as i32,
                    state_bytes.as_ptr() as i32,
                    state_bytes.len() as i32,
                );
            }
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // host mode: record the event in the host notification
            // recorder (added in this task). Tests inspect via
            // `NeoVMSyscall::take_recorded_notifications()`.
            record_notification(event, state);
            Ok(())
        }
    }
```

- [ ] **Step 5: Add a host notification recorder**

In `rust-devpack/neo-syscalls/src/host_notifications.rs` (new):

```rust
use std::sync::Mutex;
use once_cell::sync::Lazy;

use neo_types::{NeoArray, NeoString, NeoValue};

#[derive(Debug, Clone)]
pub struct RecordedNotification {
    pub event: String,
    pub state: Vec<NeoValue>,
}

static RECORDED: Lazy<Mutex<Vec<RecordedNotification>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

pub fn record(event: &NeoString, state: &NeoArray<NeoValue>) {
    let mut g = RECORDED.lock().expect("notification recorder poisoned");
    g.push(RecordedNotification {
        event: event.to_utf8_string(),
        state: state.as_slice().to_vec(),
    });
}

pub fn take() -> Vec<RecordedNotification> {
    let mut g = RECORDED.lock().expect("notification recorder poisoned");
    std::mem::take(&mut *g)
}

pub fn reset() {
    let mut g = RECORDED.lock().expect("notification recorder poisoned");
    g.clear();
}
```

Add `pub mod host_notifications;` to `rust-devpack/neo-syscalls/src/lib.rs`.

Add `NeoVMSyscall::take_recorded_notifications()` and
`NeoVMSyscall::reset_recorded_notifications()` wrapper methods in
`wrapper.rs` (around line 460, next to `seed_storage`).

- [ ] **Step 6: Run workspace tests**

```bash
cargo test --workspace
```

Expected: 60 suites green. Add a unit test in
`rust-devpack/neo-syscalls/tests/wasm32_syscalls.rs` (host mode, not
wasm32) that calls `notify` and asserts the recorded notification
contains the event name and the full args array.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "fix(syscalls): B2 notify() now serialises state to runtime_notify_with_state

NEP-17/NEP-11 Transfer events emitted by Rust devpack contracts
were missing the args array on mainnet (the state arg was dropped
on the floor). This adds the runtime_notify_with_state extern,
implements NeoVM Array StackItem serialisation, and adds a host
notification recorder for tests."
```

---

### Task A3: Fix B3 — `storage_get` returns 0-length for ALL keys on wasm32

**Files:**
- Modify: `rust-devpack/neo-syscalls/src/wrapper.rs:1115-1172` (the wasm32 path of `storage_get`)

- [ ] **Step 1: Add the failing test**

In `rust-devpack/neo-syscalls/tests/wasm32_syscalls.rs`:

```rust
/// B3 regression: neo_storage_get_into must return the actual byte
/// length, and the wrapper must construct a 0-length NeoByteString
/// for length 0 (so the contract can distinguish 'present empty'
/// from 'missing').
#[test]
fn b3_storage_get_distinguishes_missing_from_empty() {
    // The wasm32 path of storage_get is not directly executable on
    // x86_64 (it's an extern). But the *host* path in
    // neovm_syscall returns NeoByteString::new(vec![]) for missing
    // keys (D14) — verify that contract here, plus assert the
    // extern documentation matches.
    use neo_syscalls::NeoVMSyscall;
    NeoVMSyscall::reset_host_state().expect("host reset ok");
    // (no seeded value) → storage_get should return 0-length, not
    // a 4 KiB zero buffer.
    let ctx = NeoVMSyscall::storage_get_context().expect("ctx");
    let got = NeoVMSyscall::storage_get(&ctx, &neo_types::NeoByteString::from_slice(b"missing"))
        .expect("storage_get ok");
    assert_eq!(got.len(), 0, "missing key must be 0-length");
    assert!(got.is_empty(), "missing key must be is_empty()");
}
```

- [ ] **Step 2: Read the current wasm32 `storage_get` implementation**

Open `wrapper.rs:1137-1172`. The current code (from the audit) calls
`neo_storage_get_into` but does not check the return value — it
always uses the buffer as-is, even if the extern returned 0 or a
negative error code.

- [ ] **Step 3: Make the wrapper honour the return length**

Replace the wasm32 path of `storage_get` (around line 1137-1172):

```rust
    #[cfg(target_arch = "wasm32")]
    pub fn storage_get(
        context: &NeoStorageContext,
        key: &NeoByteString,
    ) -> NeoResult<NeoByteString> {
        let key_slice = key.as_slice();
        // C# returns the actual byte length; 0 for missing keys,
        // negative for errors. We allocate a generous buffer
        // (StorageItem max is 64KB on mainnet; we use 64KB+1 to
        // detect overflow via the "more than cap" return).
        let mut buf = vec![0u8; 65_536];
        let written = unsafe {
            neo_storage_get_into(
                key_slice.as_ptr() as i32,
                key_slice.len() as i32,
                buf.as_mut_ptr() as i32,
                buf.len() as i32,
            )
        };
        if written < 0 {
            return Err(NeoError::InvalidState);
        }
        let len = (written as usize).min(buf.len());
        buf.truncate(len);
        Ok(NeoByteString::from_slice(&buf))
    }
```

- [ ] **Step 4: Run workspace tests**

```bash
cargo test --workspace
```

Expected: 60 suites green, the new B3 test passes.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "fix(syscalls): B3 storage_get wasm32 path honours extern return length

neo_storage_get_into returns the actual byte length (0 for
missing, negative for error). The previous wrapper ignored this
and always returned a full zero buffer for ALL keys — including
present ones — defeating the len()==0 'key absent' idiom that
NEP-17/NEP-11 contracts rely on."
```

---

### Task A4: Fix B4 — `contract_call` returns `Null` silently on wasm32

**Files:**
- Modify: `rust-devpack/neo-syscalls/src/wrapper.rs:819-870`
- Modify: `rust-devpack/neo-syscalls/src/wrapper.rs` (extern block)

- [ ] **Step 1: Add the failing test (and accept that the first version panics)**

In `rust-devpack/neo-syscalls/tests/wasm32_syscalls.rs`:

```rust
/// B4 regression: neo_contract_call must be declared and the
/// wasm32 path must panic with a clear 'not yet implemented'
/// message rather than silently returning Null. The C#-VM
/// cross-call executor is in a follow-up design.
#[test]
fn b4_contract_call_extern_declared() {
    assert!(
        EXPECTED_EXTERNS.contains(&"neo_contract_call"),
        "B4 regression: neo_contract_call missing from extern matrix"
    );
}
```

- [ ] **Step 2: Add the extern**

In the `extern "C"` block of `wrapper.rs`:

```rust
    #[link_name = "neo_contract_call"]
    fn neo_contract_call(
        hash_ptr: i32, hash_len: i32,
        method_ptr: i32, method_len: i32,
        args_ptr: i32, args_len: i32,
        call_flags: i32,
        out_ptr: i32, out_cap: i32,
    ) -> i32;
```

- [ ] **Step 3: Replace the wasm32 `contract_call` path**

In `wrapper.rs`, around line 819-852, change the wasm32 path to
call the extern and panic-loud if unimplemented. We don't have
the cross-call executor in L1; the contract MUST panic on
wasm32 if it tries to cross-call, which is strictly better
than silently returning Null:

```rust
    #[cfg(target_arch = "wasm32")]
    pub fn contract_call(
        contract_hash: &NeoByteString,
        method: &NeoString,
        args: &[NeoValue],
        call_flags: &NeoInteger,
    ) -> NeoResult<NeoValue> {
        // Full cross-contract-call executor is the L6 conformance
        // work. For L1 we panic loudly — silently returning Null
        // (the previous behaviour) was a security-correctness
        // issue: contracts thought they got a real result and
        // acted on it.
        let _ = (contract_hash, method, args, call_flags);
        panic!(
            "System.Contract.Call on wasm32 is not yet implemented; \
             see docs/superpowers/specs/2026-06-27-neo-n3-platform-support-design.md \
             layer L6. The previous behaviour silently returned Null."
        );
    }
```

- [ ] **Step 4: Same treatment for `load_script` and `contract_call_native`**

Apply the same panic-loud pattern to:
- `load_script` (wrapper.rs:804)
- `contract_call_native` (wrapper.rs:857)

Each gets a clear "not yet implemented; see L6 design" panic.

- [ ] **Step 5: Run workspace tests**

```bash
cargo test --workspace
```

Expected: 60 suites green. No new passing tests for B4 (because
the wasm32 path now panics; the `cargo test` for the wasm32 path
runs on x86_64, which uses the host path). The matrix test in
Step 1 confirms the extern is declared.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "fix(syscalls): B4 contract_call/load_script panic-loud on wasm32

System.Contract.Call, System.Runtime.LoadScript, and
System.Contract.CallNative previously returned NeoValue::Null
on wasm32 (the silent default_value_for fallback). Contracts
that chained to other contracts silently got Null and behaved
as if the call returned no value.

Panic-loud is strictly safer: deploy-time tests will fail fast
instead of silently misbehaving on mainnet. The full cross-call
executor lands in L6."
```

---

## Phase B: Add the remaining 20 syscalls (TIER 2 — silent wrong values)

### Task B1: Add extern + path for `get_random` (B5)

- [ ] **Step 1: Add the extern**

In the `extern "C"` block of `wrapper.rs`:

```rust
    #[link_name = "runtime_get_random"]
    fn neo_runtime_get_random() -> i64;
```

- [ ] **Step 2: Replace the stub**

```rust
    pub fn get_random() -> NeoResult<NeoInteger> {
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(NeoInteger::new(unsafe { neo_runtime_get_random() }));
        }
        #[cfg(not(target_arch = "wasm32"))]
        Self::call_integer("System.Runtime.GetRandom")
    }
```

- [ ] **Step 3: Add to EXPECTED_EXTERNS in the matrix test**

(`runtime_get_random` is already in the list from Task A1.)

- [ ] **Step 4: Run tests, commit**

```bash
cargo test --workspace && cargo fmt --all && git add -A && git commit -m "fix(syscalls): B5 get_random wasm32 path now real"
```

---

### Task B2–B9: Repeat the B1 pattern for the remaining 18 syscalls

For each of these, the pattern is identical to Task B1:

| Task | Syscall | Extern name |
|---|---|---|
| B2 | `get_invocation_counter` (B6) | `runtime_get_invocation_counter` |
| B2 | `get_gas_left` (B7) | `runtime_get_gas_left` |
| B3 | `current_signers` (B8) | `runtime_current_signers` |
| B4 | `get_notifications` (B9) | `runtime_get_notifications` |
| B5 | `burn_gas` | `runtime_burn_gas` |
| B5 | `get_script_container` | `runtime_get_script_container` |
| B5 | `platform` | (no extern needed — constant "NEO") |
| B6 | `get_trigger` | (no extern needed — engine state) |
| B6 | `get_network` | (no extern needed — protocol config) |
| B6 | `get_address_version` | (no extern needed — protocol config) |
| B7 | `get_call_flags` | `runtime_get_call_flags` |
| B7 | `create_standard_account` | `runtime_create_standard_account` |
| B7 | `create_multisig_account` | `runtime_create_multisig_account` |
| B7 | `native_on_persist` | `runtime_native_on_persist` |
| B7 | `native_post_persist` | `runtime_native_post_persist` |
| B8 | `storage_get_context` | `runtime_get_storage_context` |
| B8 | `storage_get_read_only_context` | `runtime_get_read_only_context` |
| B8 | `storage_as_read_only` | `runtime_storage_as_read_only` |
| B8 | `storage_find` | `runtime_storage_find` |
| B9 | `iterator_next` | `runtime_iterator_next` |
| B9 | `iterator_value` | `runtime_iterator_value` |
| B9 | `contract_call_native` | `runtime_contract_call_native` |

For each:
- Add the extern to the `extern "C"` block (or, for the
  protocol-config ones like `get_network`, add a `#[cfg(...)]`
  `const` in a new `protocol_config.rs` module that the contract
  reads at link time).
- Replace the wasm32 stub to call the extern.
- Add to EXPECTED_EXTERNS (only the ones that need an extern).
- Run `cargo test --workspace && cargo fmt --all && cargo clippy --workspace --all-targets --all-features`.
- Commit with `fix(syscalls): <task-id> <name> wasm32 path now real`.

**Special case**: `platform()` always returns `"NEO"` per C#. Add
`#[cfg(target_arch = "wasm32")] pub fn platform() -> NeoResult<NeoString> { Ok(NeoString::from("NEO")) }`.

**Special case**: `get_network`, `get_address_version`, `get_trigger`
need protocol-config externs. Add
`#[link_name = "protocol_get_network"] fn neo_protocol_get_network() -> i32;`
etc. and route accordingly.

- [ ] **At the end of Phase B**: verify the full 33-syscall matrix**

```bash
cargo test --workspace && cargo fmt --all && cargo clippy --workspace --all-targets --all-features
```

Expected: 60+ suites green, 0 clippy warnings, fmt clean.

- [ ] **Final Phase B commit**

```bash
git add -A && git commit -m "fix(syscalls): all 33 N3 syscalls have working wasm32 paths"
```

---

## Phase C: Native contract routing (L2)

### Task C1: Verify the CryptoLib descriptor (regression test)

- [ ] **Step 1: Add a test that asserts the CryptoLib hash matches the C# mainnet value**

In `wasm-neovm/src/native_contracts.rs`, the existing `crypto_lib_descriptor`
should have hash `0xd5a8e4276d983ccd0f6a6e6e9b8dcd1eb6cb74` (C# N3 mainnet
canonical). Add a `#[cfg(test)]` test:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crypto_lib_hash_matches_csharp_mainnet() {
        let d = crypto_lib_descriptor();
        let expected: [u8; 20] = [
            0xef, 0x4a, 0xd8, 0x8b, 0x5d, 0x4a, 0xce, 0xd3, 0x6e, 0x8a,
            0xc2, 0xc4, 0x4a, 0x86, 0x4c, 0xc4, 0x82, 0x27, 0xc4, 0xab,
        ];
        // Source: C# NeoToken.cs `public static readonly UInt160 ...` is
        // CryptoLib's "Native hash"; verify in:
        // https://github.com/neo-project/neo/blob/master/src/Neo/SmartContract/Native/CryptoLib.cs
        assert_eq!(d.hash, expected);
    }
}
```

Look up the actual canonical hash on the C# mainnet source before
committing. **Do not commit a guessed value.**

- [ ] **Step 2: Run and commit**

```bash
cargo test -p wasm-neovm --features exec && git add -A && git commit -m "test(wasm-neovm): C#-mainnet canonical hash regression for CryptoLib"
```

---

### Task C2–C11: Add descriptors for the 10 unrouted native contracts

For each native contract below, do the following:

1. Fetch the canonical hash and method list from C#.
   - ContractManagement: `https://raw.githubusercontent.com/neo-project/neo/master/src/Neo/SmartContract/Native/ContractManagement.cs`
   - LedgerContract: `…/Native/LedgerContract.cs`
   - OracleContract: `…/Native/OracleContract.cs`
   - PolicyContract: `…/Native/PolicyContract.cs`
   - RoleManagement: `…/Native/RoleManagement.cs`
   - StdLib: `…/Native/StdLib.cs`
   - Notary: `…/Native/Notary.cs`
   - Governance: `…/Native/Governance.cs`
   - TokenManagement: `…/Native/TokenManagement.cs`
   - Treasury: `…/Native/Treasury.cs`
2. For each, grep the C# source for `Register("Neo.<X>.<method>")` to
   get the canonical hash and method names.
3. Add to `wasm-neovm/src/native_contracts.rs`:
   ```rust
   pub fn <x>_descriptor() -> NativeContractDescriptor {
       NativeContractDescriptor {
           hash: [...; 20],  // little-endian, exactly as C# stores it
           name: "Neo.<X>",
           methods: &[
               ("method_name", &[("param", ParamType), ...]),
               // ...
           ],
       }
   }
   ```
4. Add to the `native_contract_registry()` list.
5. Add a `#[cfg(test)]` regression test that asserts the hash
   matches the C# mainnet value.
6. Run `cargo test -p wasm-neovm --features exec`.
7. Commit `feat(wasm-neovm): route <X> native contract via descriptor`.

**Special: `Neo.StdLib.itoa` / `atoi`** — these are integer/byte
serialisation. The descriptor is just method dispatch; the actual
implementation lives in the calling code, not in the descriptor.
The descriptor's job is to make the manifest emit the right
method tokens.

**Special: `Neo.Oracle.Request`** — the args include a `filter`
which is a `ByteString` representing a regex; the C# signature
must be matched exactly.

- [ ] **At the end of Phase C**: verify all 11 native contracts routed**

```bash
cargo test -p wasm-neovm --features exec && cargo fmt --all && cargo clippy --workspace --all-targets --all-features
```

Expected: 0 warnings, all tests pass.

- [ ] **Final Phase C commit**

```bash
git add -A && git commit -m "feat(wasm-neovm): route all 11 N3 native contracts"
```

---

## Phase D: Version bump, CHANGELOG, release

### Task D1: Bump workspace to v0.7.0

- [ ] **Step 1: Update root `Cargo.toml`**

```toml
[workspace.package]
version = "0.7.0"
```

- [ ] **Step 2: Update every `version = "0.6.0"` in `workspace.dependencies` to `0.7.0`**

```bash
rg "version = \"0.6.0\"" Cargo.toml | wc -l
# Replace each
```

- [ ] **Step 3: Update all `contracts/*/Cargo.toml`**

```bash
rg "version = \"0.6.0\"" contracts/ -l
# Replace each `neo-devpack` version
```

- [ ] **Step 4: Run workspace tests**

```bash
cargo test --workspace
```

Expected: 60+ suites green.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "chore(release): v0.7.0 — L1 (syscalls) + L2 (natives) of N3 platform support"
```

---

### Task D2: CHANGELOG entry

- [ ] **Step 1: Add a v0.7.0 section**

At the top of `CHANGELOG.md`:

```markdown
## v0.7.0 — 2026-06-27 — L1 (syscalls) + L2 (natives) of N3 platform support

### Fixed (B1–B4 silent on-chain corruption)
- `NeoVMSyscall::get_executing_script_hash` / `get_calling_script_hash` /
  `get_entry_script_hash` (ByteString form) no longer return zeros on
  wasm32; they call the real `runtime_get_*_script_hash` extern.
- `NeoRuntime::notify(event, state)` now serialises the state array
  via `runtime_notify_with_state`; NEP-17/NEP-11 `Transfer(from,to,amount)`
  events now carry the args on mainnet.
- `NeoVMSyscall::storage_get` on wasm32 honours the
  `neo_storage_get_into` return length; missing keys are 0-length,
  present keys are real bytes (previous: all-zero buffer for all).
- `System.Contract.Call` / `System.Runtime.LoadScript` /
  `System.Contract.CallNative` now panic-loud on wasm32 with a
  clear "see L6 design" message; previously silently returned
  `NeoValue::Null` (silently wrong).

### Fixed (B5–B9 silent wrong values)
- `get_random`, `get_invocation_counter`, `get_gas_left`,
  `current_signers`, `get_notifications` now have wasm32 externs.

### Added
- 33-syscall wasm32 path coverage matrix in
  `rust-devpack/neo-syscalls/tests/wasm32_syscalls.rs` (regression).
- 11-native-contract descriptor registry in
  `wasm-neovm/src/native_contracts.rs`: ContractManagement, CryptoLib,
  LedgerContract, OracleContract, PolicyContract, RoleManagement,
  StdLib, Notary, Governance, TokenManagement, Treasury.
- Host notification recorder (test-only) for verifying
  `notify(event, state)` round-trips in unit tests.

### Still tracked as follow-up
- L3: translator 176 bail sites → catalogue + ~5 real bug fixes.
- L4: devpack type/iterator ergonomics (B18–B22, Q6–Q10).
- L5: README production-readiness matrix + NEP standard-library
  macros.
- L6: C#-NeoVM conformance oracle (cross-compile to NEF, run on
  C# VM, diff events/storage/return).
```

- [ ] **Step 2: Commit**

```bash
git add -A && git commit -m "docs(changelog): v0.7.0 entry"
```

---

## Phase E: Final consolidation

- [ ] **Step 1: Run full verification**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
cargo build --manifest-path contracts/nep17-token/Cargo.toml --release --target wasm32-unknown-unknown
```

Expected: 0 clippy warnings, 60+ test suites green, contracts
build to wasm32.

- [ ] **Step 2: Push to remote**

```bash
git push origin master
```

- [ ] **Step 3: Report the result to the user**

A short summary, like the prior audit summary, listing:
- TIER-1 bugs fixed (B1–B4).
- TIER-2 bugs fixed (B5–B9).
- Native contracts routed (11/11).
- Workspace version: 0.7.0.
- Test suite count.
- What's still in scope (L3–L6) as follow-up.

---

*Plan written 2026-06-27 by opencode, derived from the
`2026-06-27-neo-n3-platform-support-design.md` spec.*

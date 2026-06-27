# B5–B9: Host-Mode State for Runtime Syscalls — Design

**Status:** proposed (v0.13.0 milestone)
**Author:** auto-generated
**Date:** 2026-06-27
**Predecessor:** v0.12.0 L6 real cross-call executor.

## Goal

The v0.7.0 audit identified B5–B9 as TIER 2 silent-wrong-value
bugs: the host-mode test framework returned 0 / empty for
several runtime syscalls because the host has no state to
return. The wasm32 path is correct (each wrapper calls an
extern that the host provides), but the **host-mode test
framework** that ships with the devpack was missing the
state. This made it impossible to test contracts that use
these syscalls.

B5–B9 fixes the host-mode state and the `neovm_syscall`
dispatch, so that tests can configure the state and assert
real values flow through.

## The five bugs

| ID | Audit | Host-mode current | What it should do |
|---|---|---|---|
| B5 | `get_random` returns 0 on wasm32 | Returns 0 | Read `ACTIVE_RANDOM` (configured by tests) |
| B6 | `get_time` (ByteString form) and `get_invocation_counter` return zero | Returns 0 | Read `ACTIVE_TIME` / `ACTIVE_INVOCATION_COUNTER` |
| B7 | `get_gas_left` returns 0 | Returns 0 | Read `ACTIVE_GAS_LEFT` |
| B8 | `current_signers` returns empty | Returns empty | Read `ACTIVE_SIGNERS` (configured by tests) |
| B9 | `get_notifications(hash)` returns empty | Returns empty | Read `ACTIVE_NOTIFICATIONS` (recorded by `notify` test) |

(Note: B5–B7 are integer syscalls; B8 and B9 return arrays.
The wasm32 externs already exist and the wasm32 path returns
real values; this design is for the **host-mode test
framework only**.)

## Mechanism

### 1. Add host-side state

In `rust-devpack/neo-syscalls/src/storage.rs`:

```rust
pub(crate) static ACTIVE_TIME: Lazy<RwLock<i64>> =
    Lazy::new(|| RwLock::new(0));
pub(crate) static ACTIVE_RANDOM: Lazy<RwLock<i64>> =
    Lazy::new(|| RwLock::new(0));
pub(crate) static ACTIVE_GAS_LEFT: Lazy<RwLock<i64>> =
    Lazy::new(|| RwLock::new(0));
pub(crate) static ACTIVE_INVOCATION_COUNTER: Lazy<RwLock<i32>> =
    Lazy::new(|| RwLock::new(0));
pub(crate) static ACTIVE_SIGNERS: Lazy<RwLock<Vec<NeoSigner>>> =
    Lazy::new(|| RwLock::new(Vec::new()));
pub(crate) static ACTIVE_NOTIFICATIONS: Lazy<RwLock<Vec<RecordedNotification>>> =
    Lazy::new(|| RwLock::new(Vec::new()));
```

`NeoSigner` is a new struct:
```rust
pub(crate) struct NeoSigner {
    pub(crate) account: NeoByteString,
    pub(crate) scopes: NeoInteger,
}
```

`RecordedNotification` already exists (in `host_notifications.rs`).

### 2. Add setters

In `rust-devpack/neo-syscalls/src/wrapper.rs`:

```rust
#[cfg(not(target_arch = "wasm32"))]
pub fn set_active_time(t: i64) -> NeoResult<()> { ... }
#[cfg(not(target_arch = "wasm32"))]
pub fn set_active_random(r: i64) -> NeoResult<()> { ... }
#[cfg(not(target_arch = "wasm32"))]
pub fn set_active_gas_left(g: i64) -> NeoResult<()> { ... }
#[cfg(not(target_arch = "wasm32"))]
pub fn set_active_invocation_counter(c: i32) -> NeoResult<()> { ... }
#[cfg(not(target_arch = "wasm32"))]
pub fn set_active_signers(s: &[NeoSigner]) -> NeoResult<()> { ... }
```

`ACTIVE_NOTIFICATIONS` is populated by the existing
`record_notification` (already wired); no new setter needed.
`reset_host_state` clears all of these.

### 3. Wire `neovm_syscall` dispatch

In `rust-devpack/neo-syscalls/src/wrapper.rs`, add to the
`#[cfg(not(target_arch = "wasm32"))]` block in `neovm_syscall`:

```rust
if info.name == "System.Runtime.GetTime" {
    return Ok(NeoInteger::new(*ACTIVE_TIME.read().unwrap()).into());
}
if info.name == "System.Runtime.GetRandom" {
    return Ok(NeoInteger::new(*ACTIVE_RANDOM.read().unwrap()).into());
}
if info.name == "System.Runtime.GasLeft" {
    return Ok(NeoInteger::new(*ACTIVE_GAS_LEFT.read().unwrap()).into());
}
if info.name == "System.Runtime.GetInvocationCounter" {
    return Ok(NeoInteger::new(*ACTIVE_INVOCATION_COUNTER.read().unwrap()).into());
}
if info.name == "System.Runtime.CurrentSigners" {
    let signers = ACTIVE_SIGNERS.read().unwrap();
    let arr: NeoArray<NeoValue> = signers.iter()
        .map(|s| NeoValue::from(NeoArray::from(vec![
            NeoValue::from(s.account.clone()),
            NeoValue::from(s.scopes.clone()),
        ])))
        .collect();
    return Ok(NeoValue::from(arr));
}
if info.name == "System.Runtime.GetNotifications" {
    // ... filter by hash arg, return recorded notifications
}
```

### 4. Tests

`rust-devpack/tests/b5_b9_host_state.rs` (new file), 5 tests:
- `b5_get_random_returns_active_value`
- `b6_get_time_returns_active_value`
- `b7_get_gas_left_returns_active_value`
- `b8_current_signers_returns_active_signers`
- `b9_get_notifications_returns_recorded_notifications`

Each test: set host state, call syscall, assert return value.
Each test cleans up via `reset_host_state()`.

### 5. CHANGELOG + version bump

v0.13.0. CHANGELOG entry noting the host-mode state additions
and the audit-closure of TIER 1+2 (B1–B9 all closed).

## Affected sites

- `rust-devpack/neo-syscalls/src/storage.rs`: 5 new
  `Lazy<RwLock<...>>` statics; `NeoSigner` struct.
- `rust-devpack/neo-syscalls/src/wrapper.rs`: 5 new setters;
  5 new branches in `neovm_syscall` dispatch; updates to
  `reset_host_state`.
- `rust-devpack/neo-syscalls/src/host_notifications.rs`:
  `RecordedNotification` may need to be re-exported; check.
- `rust-devpack/tests/b5_b9_host_state.rs`: 5 new tests.
- `CHANGELOG.md`: v0.13.0 entry.

## Definition of done

- 5 new host-state statics + NeoSigner struct.
- 5 new setters + dispatch branches.
- 5 new tests in `b5_b9_host_state.rs`.
- `cargo test --workspace` + `cargo clippy --workspace
  --all-targets --all-features` both green.
- All 66 prior test suites still pass.
- CHANGELOG entry for v0.13.0.
- Merged to `master`, pushed to `origin`.

## Open questions

- `NeoSigner` field shape: the C# struct has `Account`,
  `Scopes`, `ContractParameters`. We only need `Account` +
  `Scopes` for the host-mode test framework. Default to
  minimal (Account + Scopes).
- Should `set_active_signers` accept a slice or a Vec?
  Default to slice (consistent with `set_active_witnesses`).
- The `get_notifications` filter is by `Option<&NeoByteString>`.
  If `Some(hash)`, return only notifications for that hash.
  If `None`, return all. Default to both code paths in the
  test (one for each variant).

// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Feature-coverage sample: the runtime / interop-syscall surface reachable
//! from a real wasm32 contract via the `NeoRuntime` facade, `NeoRuntimeContext`,
//! and the `NeoVMSyscall` extras. Every method reduces its result to a scalar
//! (`i64` / `bool`) for the export boundary, and the three `NeoResult<_>` export
//! return kinds (`NeoInteger`, `NeoBoolean`, `()`) are each exercised.
//!
//! ## Scope: scalar vs. buffer-returning syscalls
//!
//! NeoVM syscalls that return an **integer** (GetTime, GetTrigger, GetNetwork,
//! GetAddressVersion, GasLeft, GetRandom, GetInvocationCounter, GetCallFlags,
//! CheckWitness) lower cleanly to `SYSCALL <hash>` and are covered here.
//!
//! Syscalls that return a **ByteString / Array** to a linear-memory buffer
//! (`out_ptr, out_cap -> len` ABI) — `GetExecutingScriptHash`/etc. in their
//! byte-string form, `Platform`, `GetNotifications`, `CurrentSigners`,
//! `GetScriptContainer`, `CreateStandardAccount`, `CreateMultisigAccount`,
//! `LoadScript` — require the translator to marshal the NeoVM stack item into
//! wasm linear memory. That marshalling is implemented only for `CheckWitness`
//! and the script-hash **i64** prefix forms, so the byte-string/array forms are
//! intentionally NOT exercised here (they would emit a NEF that faults on
//! chain). The script-hash feature is instead covered through its i64 prefix
//! form (`calling_id` / `entry_id` / `me_id`), which is fully bridged.

use neo_devpack::prelude::*;
use neo_devpack::NeoVMSyscall;

neo_manifest_overlay!(r#"{ "name": "FeatureRuntime" }"#);

#[neo_contract]
pub struct RuntimeContract;

#[neo_contract]
impl RuntimeContract {
    pub fn new() -> Self {
        Self
    }

    /// `get_time` returned as `NeoResult<NeoInteger>` — covers that export kind.
    #[neo_method(safe)]
    pub fn now_big() -> NeoResult<NeoInteger> {
        NeoRuntime::get_time()
    }

    /// `get_time_i64` (the scalar convenience form).
    #[neo_method(safe)]
    pub fn now_ms() -> i64 {
        NeoRuntime::get_time_i64().unwrap_or(-1)
    }

    /// `check_witness_i64` returned as `NeoResult<NeoBoolean>` — covers that
    /// export kind.
    #[neo_method(safe)]
    pub fn witness(acct: i64) -> NeoResult<NeoBoolean> {
        NeoRuntime::check_witness_i64(acct)
    }

    /// `check_witness` (20-byte ByteString) + `check_witness_bytes` (&[u8]).
    /// Both forms marshal the account bytes from linear memory.
    #[neo_method(safe)]
    pub fn witness_bytes() -> bool {
        let acct = NeoByteString::from_slice(&[0u8; 20]);
        let a = NeoRuntime::check_witness(&acct)
            .map(|b| b.as_bool())
            .unwrap_or(false);
        let b = NeoRuntime::check_witness_bytes(&[0u8; 20])
            .map(|x| x.as_bool())
            .unwrap_or(false);
        a || b
    }

    /// `require_witness_i64` (asserting form built on CheckWitness).
    #[neo_method(safe)]
    pub fn require_wit(acct: i64) -> bool {
        NeoRuntime::require_witness_i64(acct)
    }

    /// `get_trigger` (reachable via the `protocol_get_trigger` alias fix).
    #[neo_method(safe)]
    pub fn trig() -> i64 {
        NeoRuntime::get_trigger()
            .map(|n| n.as_i64_saturating())
            .unwrap_or(-1)
    }

    /// `get_invocation_counter`.
    #[neo_method(safe)]
    pub fn inv_count() -> i64 {
        NeoRuntime::get_invocation_counter()
            .map(|n| n.as_i64_saturating())
            .unwrap_or(-1)
    }

    /// `get_random`.
    #[neo_method(safe)]
    pub fn rand() -> i64 {
        NeoRuntime::get_random()
            .map(|n| n.as_i64_saturating())
            .unwrap_or(0)
    }

    /// `get_network` (reachable via the `protocol_get_network` alias fix).
    #[neo_method(safe)]
    pub fn network() -> i64 {
        NeoRuntime::get_network()
            .map(|n| n.as_i64_saturating())
            .unwrap_or(-1)
    }

    /// `get_address_version` (reachable via the `protocol_get_address_version`
    /// alias fix).
    #[neo_method(safe)]
    pub fn addr_version() -> i64 {
        NeoRuntime::get_address_version()
            .map(|n| n.as_i64_saturating())
            .unwrap_or(-1)
    }

    /// `get_gas_left` (reachable via the `runtime_get_gas_left` alias fix).
    #[neo_method(safe)]
    pub fn gas_left() -> i64 {
        NeoRuntime::get_gas_left()
            .map(|n| n.as_i64_saturating())
            .unwrap_or(-1)
    }

    /// `get_calling_script_hash` reduced to its i64 prefix (fully bridged form).
    #[neo_method(safe)]
    pub fn calling_id() -> i64 {
        NeoRuntime::get_calling_script_hash_i64().unwrap_or(0)
    }

    /// `get_entry_script_hash` i64 prefix.
    #[neo_method(safe)]
    pub fn entry_id() -> i64 {
        NeoRuntime::get_entry_script_hash_i64().unwrap_or(0)
    }

    /// `get_executing_script_hash` i64 prefix.
    #[neo_method(safe)]
    pub fn me_id() -> i64 {
        NeoRuntime::get_executing_script_hash_i64().unwrap_or(0)
    }

    /// `burn_gas` — covers the `NeoResult<()>` export kind.
    #[neo_method]
    pub fn burn(g: i64) -> NeoResult<()> {
        NeoRuntime::burn_gas(&NeoInteger::new(g))
    }

    /// `log` — covers `NeoResult<()>` and the runtime Log syscall (which the
    /// conformance oracle can actually observe).
    #[neo_method]
    pub fn log_msg() -> NeoResult<()> {
        NeoRuntime::log(&NeoString::from_str("feature-runtime"))
    }

    /// `get_call_flags` via `NeoVMSyscall` (reachable via the
    /// `runtime_get_call_flags` alias fix).
    #[neo_method(safe)]
    pub fn call_flags() -> i64 {
        NeoVMSyscall::get_call_flags()
            .map(|n| n.as_i64_saturating())
            .unwrap_or(-1)
    }

    /// `NeoRuntimeContext` scalar accessors: trigger / gas_left /
    /// invocation_counter, summed (all integer-returning, all bridged).
    #[neo_method(safe)]
    pub fn ctx_scalars() -> i64 {
        let ctx = NeoRuntimeContext::new();
        let t = ctx.trigger().map(|n| n.as_i64_saturating()).unwrap_or(0);
        let g = ctx.gas_left().map(|n| n.as_i64_saturating()).unwrap_or(0);
        let i = ctx
            .invocation_counter()
            .map(|n| n.as_i64_saturating())
            .unwrap_or(0);
        t.wrapping_add(g).wrapping_add(i)
    }
}

// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Feature-coverage sample: contract events and the full manifest-macro
//! surface.
//!
//! - `#[neo_event]` structs (`Transfer` with rich `NeoByteString`/`NeoInteger`
//!   fields, `Ping` with one field) and their generated `.emit()` method,
//!   which lowers to `System.Runtime.Notify` (a dedicated, marshalled handler
//!   in the translator — the conformance oracle can observe these).
//! - `NeoRuntime::notify_event` (name-only notify).
//! - Module-level manifest macros: `neo_manifest_overlay!`,
//!   `neo_supported_standards!`, `neo_permission!`, `neo_trusts!`,
//!   `neo_safe_methods!`.
//! - Method/function attributes: `#[neo_method(safe)]` (per-method safe) and
//!   `#[neo_entry]` (deploy entry point).
//!
//! The two `safe` mechanisms exercised here are `#[neo_method(safe)]` (method
//! `marked`) and the `neo_safe_methods!(["pingCount"])` macro (method
//! `pingCount`, declared without `(safe)` on its attribute). The third macro,
//! `#[neo_safe]`, is intentionally NOT used: it expands to an anonymous
//! `const _: () = …` overlay item, which is illegal inside an `impl` block, and
//! when placed on a module-level free function the overlay marks a method name
//! that the devpack never exports (free functions are not exported), so it
//! fails translation either way. Use `#[neo_method(safe)]` or
//! `neo_safe_methods!` instead.
//!
//! ## Notifications carry their payload on wasm32 too
//!
//! `#[neo_event]::emit()` uses the same state-carrying path on every target:
//! `NeoRuntime::notify(name, state)`. On `wasm32` that crosses the
//! `runtime_notify_with_state` import with the state serialised in the
//! canonical NeoVM `BinarySerializer` wire format; the wasm-neovm translator
//! marshals both buffers out of linear memory, decodes the state on-VM via
//! the StdLib native's `deserialize` (the scoped manifest permission is
//! auto-inserted), and emits `SYSCALL System.Runtime.Notify` — so on-chain
//! notifications carry exactly the payload host tests observe. `raw_notify`
//! below exercises `NeoRuntime::notify` directly with mixed value types.

use neo_devpack::prelude::*;

neo_manifest_overlay!(r#"{ "name": "FeatureEventsManifest", "features": { "storage": true } }"#);
// Events emitted through `NeoRuntime::notify`/`notify_event` directly (not via
// `#[neo_event]`, which generates its own ABI overlay) must be declared by
// hand: Neo N3 (HF_Basilisk) faults notifications whose name/arity is not in
// the manifest `events` ABI.
neo_manifest_overlay!(
    r#"{ "abi": { "events": [
        { "name": "Started", "parameters": [] },
        { "name": "Mixed", "parameters": [
            { "name": "flag", "type": "Boolean" },
            { "name": "tag", "type": "ByteArray" },
            { "name": "value", "type": "Integer" }
        ] }
    ] } }"#
);
neo_supported_standards!(["NEP-17"]);
neo_permission!("*", ["balanceOf", "transfer"]);
neo_trusts!(["0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"]);
neo_safe_methods!(["pingCount"]);

/// Rich-field event (ByteString + Integer fields exercise the
/// `NeoValue::from(field)` codegen for non-scalar types).
#[neo_event]
pub struct Transfer {
    pub from: NeoByteString,
    pub to: NeoByteString,
    pub amount: NeoInteger,
}

/// Single-field event.
#[neo_event]
pub struct Ping {
    pub seq: NeoInteger,
}

#[neo_contract]
pub struct EventsContract;

#[neo_contract]
impl EventsContract {
    pub fn new() -> Self {
        Self
    }

    /// Emit a `Transfer` event via the generated `.emit()`.
    #[neo_method]
    pub fn fire_transfer(amt: i64) -> NeoResult<()> {
        Transfer {
            from: NeoByteString::from_slice(&[0u8; 20]),
            to: NeoByteString::from_slice(&[1u8; 20]),
            amount: NeoInteger::new(amt),
        }
        .emit()
    }

    /// Emit a `Ping` event.
    #[neo_method]
    pub fn fire_ping(seq: i64) -> NeoResult<()> {
        Ping {
            seq: NeoInteger::new(seq),
        }
        .emit()
    }

    /// Name-only `NeoRuntime::notify_event`.
    #[neo_method]
    pub fn raw_notify_event() -> NeoResult<()> {
        NeoRuntime::notify_event("Started")
    }

    /// State-carrying `NeoRuntime::notify` with mixed value types
    /// (Boolean + ByteString + Integer), exercising the serialised-state
    /// bridge beyond the `#[neo_event]` field kinds.
    #[neo_method]
    pub fn raw_notify(flag: bool, value: i64) -> NeoResult<()> {
        let name = NeoString::from_str("Mixed");
        let mut state = NeoArray::new();
        state.push(NeoValue::from(NeoBoolean::new(flag)));
        state.push(NeoValue::from(NeoByteString::from_slice(b"payload")));
        state.push(NeoValue::from(NeoInteger::new(value)));
        NeoRuntime::notify(&name, &state)
    }

    /// Read-only method marked safe via the `neo_safe_methods!` macro (note:
    /// no `(safe)` on the attribute — the macro supplies the safe flag).
    #[neo_method]
    pub fn ping_count() -> i64 {
        42
    }

    /// Read-only method marked safe via the per-method attribute form.
    #[neo_method(safe)]
    pub fn marked(x: i64) -> i64 {
        x.wrapping_mul(2)
    }
}

/// `#[neo_entry]` deploy entry point.
#[neo_entry]
pub fn deploy() -> NeoResult<()> {
    Ok(())
}

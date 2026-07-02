// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! wasm32 `neo` import-module ABI surface.
//!
//! Every `extern "C"` declaration here is a wasm32 link-time import that the
//! wasm-neovm translator must recognize by name. `WASM32_IMPORT_ABI` below is
//! the declarative mirror of this extern block: one row per `link_name`,
//! carrying the canonical Neo syscall descriptor the import lowers to and
//! whether the SDK-side wrapper is still a stub. The unit test at the bottom
//! keeps the table and the extern declarations in lock-step, and the ABI
//! parity test in `wasm-neovm` (dev-dependency edge) asserts the translator
//! recognizes every non-allowlisted row — so bridge drift fails a test
//! instead of shipping.

// Some declarations here are the reserved host ABI for syscalls whose
// wasm32 wrapper is still a stub (e.g. the storage-context accessors) or are
// superseded by a differently-named extern (`runtime_load_script` /
// `runtime_contract_call_native`). They are intentionally declared so the
// ABI surface is documented in one place; `allow(dead_code)` keeps the
// wasm32 contract build warning-free until each wrapper is wired up.
#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
#[link(wasm_import_module = "neo")]
extern "C" {
    #[link_name = "runtime_check_witness_bytes"]
    pub(crate) fn neo_runtime_check_witness_bytes(ptr: i32, len: i32) -> i32;

    #[link_name = "runtime_check_witness_i64"]
    pub(crate) fn neo_runtime_check_witness_i64(account: i64) -> i32;

    #[link_name = "runtime_get_time"]
    pub(crate) fn neo_runtime_get_time() -> i64;

    #[link_name = "runtime_get_calling_script_hash_i64"]
    pub(crate) fn neo_runtime_get_calling_script_hash_i64() -> i64;

    #[link_name = "runtime_get_entry_script_hash_i64"]
    pub(crate) fn neo_runtime_get_entry_script_hash_i64() -> i64;

    #[link_name = "runtime_get_executing_script_hash_i64"]
    pub(crate) fn neo_runtime_get_executing_script_hash_i64() -> i64;

    /// B1: ByteString-form (20-byte) script hashes. Returns the number
    /// of bytes written into `out_ptr` (20 on success, negative on error).
    /// Previously the wrappers returned `vec![0u8; 20]` on wasm32, silently
    /// producing zero hashes on mainnet. B1 fix.
    #[link_name = "runtime_get_calling_script_hash"]
    pub(crate) fn neo_runtime_get_calling_script_hash(out_ptr: i32, out_cap: i32) -> i32;
    #[link_name = "runtime_get_entry_script_hash"]
    pub(crate) fn neo_runtime_get_entry_script_hash(out_ptr: i32, out_cap: i32) -> i32;
    #[link_name = "runtime_get_executing_script_hash"]
    pub(crate) fn neo_runtime_get_executing_script_hash(out_ptr: i32, out_cap: i32) -> i32;

    #[link_name = "runtime_log"]
    pub(crate) fn neo_runtime_log(ptr: i32, len: i32);

    /// B2: state-carrying notify. The args array is serialised as a
    /// NeoVM `Array` StackItem (1-byte tag, varint count, items…) — the
    /// canonical `BinarySerializer` wire format — and handed to the VM.
    /// Previously `runtime_notify` only carried the event name and
    /// dropped the state, so NEP-17/NEP-11 Transfer events emitted
    /// `Transfer(<empty>)` on mainnet. B2 fix. The wasm-neovm translator
    /// lowers `runtime_notify_with_state` by marshalling both buffers out
    /// of linear memory and decoding the state on-VM via the StdLib
    /// native's `deserialize`, then emitting `System.Runtime.Notify`.
    #[link_name = "runtime_notify"]
    pub(crate) fn neo_runtime_notify(event_ptr: i32, event_len: i32);
    #[link_name = "runtime_notify_with_state"]
    pub(crate) fn neo_runtime_notify_with_state(
        event_ptr: i32,
        event_len: i32,
        state_ptr: i32,
        state_len: i32,
    );

    /// D3: lowered to `SYSCALL System.Crypto.CheckSig(pubkey, signature)`.
    /// Verifies the ECDSA signature of the current script hash using `pubkey`.
    /// The translator's `neo::*` alias maps `check_sig` to that syscall.
    #[link_name = "check_sig"]
    pub(crate) fn neo_runtime_check_sig(
        pubkey_ptr: i32,
        pubkey_len: i32,
        sig_ptr: i32,
        sig_len: i32,
    ) -> i32;

    /// D3: lowered to `SYSCALL System.Crypto.CheckMultisig(pubkeys, signatures)`.
    #[link_name = "check_multisig"]
    pub(crate) fn neo_runtime_check_multisig(
        pubkeys_ptr: i32,
        pubkeys_len: i32,
        sigs_ptr: i32,
        sigs_len: i32,
    ) -> i32;

    /// D3: lowered to `System.Contract.Call(cryptoLib, "verifyWithECDsa", ...)`
    /// via the C1 native-contract routing.
    #[link_name = "verify_with_ecdsa"]
    pub(crate) fn neo_runtime_verify_with_ecdsa(
        msg_ptr: i32,
        msg_len: i32,
        pubkey_ptr: i32,
        pubkey_len: i32,
        sig_ptr: i32,
        sig_len: i32,
        curve: i32,
    ) -> i32;

    /// Lowered to `CALL_L` -> helper that emits
    ///   `SYSCALL System.Storage.GetContext;
    ///    SUBSTR <key>; SUBSTR <value>;
    ///    SYSCALL System.Storage.Put`.
    #[link_name = "neo_storage_put_bytes"]
    pub(crate) fn neo_storage_put_bytes(key_ptr: i32, key_len: i32, value_ptr: i32, value_len: i32);

    /// Lowered to `CALL_L` -> helper that emits
    ///   `SYSCALL System.Storage.GetContext;
    ///    SUBSTR <key>;
    ///    SYSCALL System.Storage.Delete`.
    #[link_name = "neo_storage_delete_bytes"]
    pub(crate) fn neo_storage_delete_bytes(key_ptr: i32, key_len: i32);

    /// Lowered to `CALL_L` -> helper that emits the `Get` SYSCALL and then
    /// copies the returned `ByteString` back into wasm memory at `out_ptr`.
    /// Returns:
    ///   - the stored value's length on success (`>= 0`),
    ///   - `-1` if the key is not present in storage,
    ///   - `-needed_len` if the caller-supplied buffer was too small.
    #[link_name = "neo_storage_get_into"]
    pub(crate) fn neo_storage_get_into(
        key_ptr: i32,
        key_len: i32,
        out_ptr: i32,
        out_cap: i32,
    ) -> i32;

    /// TIER-2: runtime syscalls that previously returned zero/empty on
    /// wasm32 (silent wrong values). Each is paired with a real extern
    /// here; the wasm32 path on `NeoVMSyscall` is updated to call it.
    #[link_name = "runtime_get_random"]
    pub(crate) fn neo_runtime_get_random() -> i64;
    #[link_name = "runtime_get_invocation_counter"]
    pub(crate) fn neo_runtime_get_invocation_counter() -> i32;
    #[link_name = "runtime_get_gas_left"]
    pub(crate) fn neo_runtime_get_gas_left() -> i64;
    #[link_name = "runtime_get_notifications"]
    pub(crate) fn neo_runtime_get_notifications(
        script_hash_ptr: i32,
        script_hash_len: i32,
        out_ptr: i32,
        out_cap: i32,
    ) -> i32;
    #[link_name = "runtime_current_signers"]
    pub(crate) fn neo_runtime_current_signers(out_ptr: i32, out_cap: i32) -> i32;
    #[link_name = "runtime_burn_gas"]
    pub(crate) fn neo_runtime_burn_gas(gas: i64);
    #[link_name = "runtime_get_script_container"]
    pub(crate) fn neo_runtime_get_script_container(out_ptr: i32, out_cap: i32) -> i32;
    #[link_name = "runtime_load_script"]
    pub(crate) fn neo_runtime_load_script(
        script_ptr: i32,
        script_len: i32,
        call_flags: i32,
        args_ptr: i32,
        args_len: i32,
    );
    #[link_name = "runtime_create_standard_account"]
    pub(crate) fn neo_runtime_create_standard_account(
        pubkey_ptr: i32,
        pubkey_len: i32,
        out_ptr: i32,
        out_cap: i32,
    ) -> i32;
    #[link_name = "runtime_create_multisig_account"]
    pub(crate) fn neo_runtime_create_multisig_account(
        threshold: i32,
        pubkeys_ptr: i32,
        pubkeys_len: i32,
        out_ptr: i32,
        out_cap: i32,
    ) -> i32;
    #[link_name = "runtime_contract_call_native"]
    pub(crate) fn neo_runtime_contract_call_native(
        native_id: i32,
        method_ptr: i32,
        method_len: i32,
        args_ptr: i32,
        args_len: i32,
        out_ptr: i32,
        out_cap: i32,
    ) -> i32;
    #[link_name = "runtime_get_call_flags"]
    pub(crate) fn neo_runtime_get_call_flags() -> i32;
    #[link_name = "runtime_get_storage_context"]
    pub(crate) fn neo_runtime_get_storage_context() -> i32;
    #[link_name = "runtime_get_read_only_context"]
    pub(crate) fn neo_runtime_get_read_only_context() -> i32;
    #[link_name = "runtime_storage_as_read_only"]
    pub(crate) fn neo_runtime_storage_as_read_only(context_id: i32) -> i32;

    /// Prefix-scan ABI (SINGLE-LIVE-ITERATOR model).
    ///
    /// `runtime_storage_find(prefix)` lowers to
    /// `SYSCALL System.Storage.Find(GetContext(), prefix, FindOptions.None)`
    /// and parks the returned iterator InteropInterface in a
    /// translator-managed static slot — wasm code cannot hold VM stack
    /// items, so there is exactly ONE live iterator per contract execution
    /// and a new `find` replaces the previous iterator. `FindOptions.None`
    /// means each element is a `Struct{ key (incl. prefix), value }`.
    ///
    /// `runtime_iterator_next()` lowers to `SYSCALL System.Iterator.Next` on
    /// the parked iterator and returns 0/1. Calling it with no live iterator
    /// FAULTs the VM (slot is null).
    ///
    /// `runtime_iterator_value(out_ptr, out_cap)` lowers to
    /// `SYSCALL System.Iterator.Value`, flattens the current
    /// `Struct{key,value}` element to `key_len(4B LE) || key || value`, and
    /// writes it into the caller's buffer using the `neo_storage_get_into`
    /// return convention: bytes written on success (`>= 0`),
    /// `-needed_len` when the buffer is too small (retry with a larger one;
    /// the iterator does not advance). Only valid after `next` returned 1.
    ///
    /// The SDK wrapper (`NeoVMSyscall::storage_find`) drains the scan
    /// eagerly, so the static slot is free again by the time it returns and
    /// nested `storage_find` calls remain safe at the Rust API level.
    #[link_name = "runtime_storage_find"]
    pub(crate) fn neo_runtime_storage_find(prefix_ptr: i32, prefix_len: i32);
    #[link_name = "runtime_iterator_next"]
    pub(crate) fn neo_runtime_iterator_next() -> i32;
    #[link_name = "runtime_iterator_value"]
    pub(crate) fn neo_runtime_iterator_value(out_ptr: i32, out_cap: i32) -> i32;

    /// Protocol-config syscalls (constant per chain). The host
    /// returns the value at link time.
    #[link_name = "protocol_get_network"]
    pub(crate) fn neo_protocol_get_network() -> i32;
    #[link_name = "protocol_get_address_version"]
    pub(crate) fn neo_protocol_get_address_version() -> i32;
    #[link_name = "protocol_get_trigger"]
    pub(crate) fn neo_protocol_get_trigger() -> i32;

    /// L6: cross-contract-call extern. The wasm32 wrapper calls
    /// this so the wasm-neovm translator sees the import and emits
    /// `SYSCALL System.Contract.Call`. The host's NeoVM dispatches
    /// the call at runtime.
    #[link_name = "neo_contract_call"]
    pub(crate) fn neo_contract_call(
        hash_ptr: i32,
        hash_len: i32,
        method_ptr: i32,
        method_len: i32,
        args_ptr: i32,
        args_len: i32,
        call_flags: i32,
        out_ptr: i32,
        out_cap: i32,
    ) -> i32;

    /// L6: load-script extern. The wasm32 wrapper calls this so
    /// the translator emits `SYSCALL System.Runtime.LoadScript`.
    #[link_name = "neo_load_script"]
    pub(crate) fn neo_load_script(
        script_ptr: i32,
        script_len: i32,
        call_flags: i32,
        args_ptr: i32,
        args_len: i32,
    ) -> i32;

    /// L6: call-native extern. The wasm32 wrapper calls this so
    /// the translator emits `SYSCALL System.Contract.CallNative`.
    #[link_name = "neo_call_native"]
    pub(crate) fn neo_call_native(
        native_id: i32,
        method_ptr: i32,
        method_len: i32,
        args_ptr: i32,
        args_len: i32,
        out_ptr: i32,
        out_cap: i32,
    ) -> i32;
}

/// Whether the SDK-side wasm32 wrapper actually invokes the extern, or the
/// declaration is a reserved/superseded stub kept for ABI documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wasm32WrapperStatus {
    /// A wasm32 wrapper in `wrapper.rs` calls this extern.
    Wired,
    /// Declared-only: reserved host ABI (no wrapper calls it yet) or
    /// superseded by a differently-named extern. See the row comments.
    Stub,
}

/// One row of the wasm32 `neo` import ABI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wasm32ImportAbi {
    /// The `link_name` of the extern declaration above (= the wasm import name).
    pub link_name: &'static str,
    /// The canonical Neo N3 syscall descriptor this import lowers to.
    pub descriptor: &'static str,
    /// Whether the SDK wrapper is wired to the extern or it is a stub.
    pub status: Wasm32WrapperStatus,
}

use Wasm32WrapperStatus::{Stub, Wired};

/// Declarative mirror of the extern block above, in declaration order.
///
/// This is the single source of truth for the SDK <-> translator ABI parity
/// gates: `abi_parity_tests` below pins it to the extern declarations and to
/// the host-mode dispatch registry, and `wasm-neovm`'s
/// `translator::translation::imports::syscall` parity test asserts the
/// translator recognizes every import name (modulo its documented allowlist
/// of known gaps). Adding an extern without a row here — or a row whose
/// descriptor the host registry cannot dispatch — fails the tests.
pub const WASM32_IMPORT_ABI: &[Wasm32ImportAbi] = &[
    Wasm32ImportAbi {
        link_name: "runtime_check_witness_bytes",
        descriptor: "System.Runtime.CheckWitness",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_check_witness_i64",
        descriptor: "System.Runtime.CheckWitness",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_get_time",
        descriptor: "System.Runtime.GetTime",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_get_calling_script_hash_i64",
        descriptor: "System.Runtime.GetCallingScriptHash",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_get_entry_script_hash_i64",
        descriptor: "System.Runtime.GetEntryScriptHash",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_get_executing_script_hash_i64",
        descriptor: "System.Runtime.GetExecutingScriptHash",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_get_calling_script_hash",
        descriptor: "System.Runtime.GetCallingScriptHash",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_get_entry_script_hash",
        descriptor: "System.Runtime.GetEntryScriptHash",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_get_executing_script_hash",
        descriptor: "System.Runtime.GetExecutingScriptHash",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_log",
        descriptor: "System.Runtime.Log",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_notify",
        descriptor: "System.Runtime.Notify",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_notify_with_state",
        descriptor: "System.Runtime.Notify",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "check_sig",
        descriptor: "System.Crypto.CheckSig",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "check_multisig",
        descriptor: "System.Crypto.CheckMultisig",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "verify_with_ecdsa",
        descriptor: "Neo.Crypto.VerifyWithECDsa",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "neo_storage_put_bytes",
        descriptor: "System.Storage.Put",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "neo_storage_delete_bytes",
        descriptor: "System.Storage.Delete",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "neo_storage_get_into",
        descriptor: "System.Storage.Get",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_get_random",
        descriptor: "System.Runtime.GetRandom",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_get_invocation_counter",
        descriptor: "System.Runtime.GetInvocationCounter",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_get_gas_left",
        descriptor: "System.Runtime.GasLeft",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_get_notifications",
        descriptor: "System.Runtime.GetNotifications",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_current_signers",
        descriptor: "System.Runtime.CurrentSigners",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_burn_gas",
        descriptor: "System.Runtime.BurnGas",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_get_script_container",
        descriptor: "System.Runtime.GetScriptContainer",
        status: Wired,
    },
    // Superseded by `neo_load_script`; the wrapper never calls it.
    Wasm32ImportAbi {
        link_name: "runtime_load_script",
        descriptor: "System.Runtime.LoadScript",
        status: Stub,
    },
    Wasm32ImportAbi {
        link_name: "runtime_create_standard_account",
        descriptor: "System.Contract.CreateStandardAccount",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_create_multisig_account",
        descriptor: "System.Contract.CreateMultisigAccount",
        status: Wired,
    },
    // Superseded by `neo_call_native`; the wrapper never calls it.
    Wasm32ImportAbi {
        link_name: "runtime_contract_call_native",
        descriptor: "System.Contract.CallNative",
        status: Stub,
    },
    Wasm32ImportAbi {
        link_name: "runtime_get_call_flags",
        descriptor: "System.Contract.GetCallFlags",
        status: Wired,
    },
    // Reserved storage-context accessors: the wasm32 wrappers return a
    // sentinel `NeoStorageContext` instead of crossing the boundary.
    Wasm32ImportAbi {
        link_name: "runtime_get_storage_context",
        descriptor: "System.Storage.GetContext",
        status: Stub,
    },
    Wasm32ImportAbi {
        link_name: "runtime_get_read_only_context",
        descriptor: "System.Storage.GetReadOnlyContext",
        status: Stub,
    },
    Wasm32ImportAbi {
        link_name: "runtime_storage_as_read_only",
        descriptor: "System.Storage.AsReadOnly",
        status: Stub,
    },
    // Single-live-iterator prefix scan: `NeoVMSyscall::storage_find` starts
    // the scan and eagerly drains it through the iterator externs (see the
    // extern-block doc comment for the flattened value ABI).
    Wasm32ImportAbi {
        link_name: "runtime_storage_find",
        descriptor: "System.Storage.Find",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_iterator_next",
        descriptor: "System.Iterator.Next",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "runtime_iterator_value",
        descriptor: "System.Iterator.Value",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "protocol_get_network",
        descriptor: "System.Runtime.GetNetwork",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "protocol_get_address_version",
        descriptor: "System.Runtime.GetAddressVersion",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "protocol_get_trigger",
        descriptor: "System.Runtime.GetTrigger",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "neo_contract_call",
        descriptor: "System.Contract.Call",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "neo_load_script",
        descriptor: "System.Runtime.LoadScript",
        status: Wired,
    },
    Wasm32ImportAbi {
        link_name: "neo_call_native",
        descriptor: "System.Contract.CallNative",
        status: Wired,
    },
];

#[cfg(all(test, not(target_arch = "wasm32")))]
mod abi_parity_tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Extract the `link_name` attribute values from this module's own
    /// source, i.e. the mechanically-reliable enumeration of the extern
    /// import surface. Comment lines are skipped so prose never counts.
    fn extern_link_names() -> Vec<String> {
        let source = include_str!("syscalls_abi.rs");
        source
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with("//"))
            .filter_map(|line| {
                let rest = line.strip_prefix("#[link_name = \"")?;
                let end = rest.find('"')?;
                Some(rest[..end].to_string())
            })
            .collect()
    }

    /// The declarative table must mirror the extern declarations exactly:
    /// same names, no duplicates, nothing missing on either side.
    #[test]
    fn abi_table_matches_extern_declarations() {
        let externs = extern_link_names();
        assert!(!externs.is_empty(), "no extern link_name attributes found");

        let extern_set: BTreeSet<&str> = externs.iter().map(String::as_str).collect();
        assert_eq!(
            extern_set.len(),
            externs.len(),
            "duplicate link_name in the extern block"
        );

        let table_set: BTreeSet<&str> = WASM32_IMPORT_ABI.iter().map(|row| row.link_name).collect();
        assert_eq!(
            table_set.len(),
            WASM32_IMPORT_ABI.len(),
            "duplicate link_name in WASM32_IMPORT_ABI"
        );

        let missing_rows: Vec<_> = extern_set.difference(&table_set).collect();
        let stale_rows: Vec<_> = table_set.difference(&extern_set).collect();
        assert!(
            missing_rows.is_empty() && stale_rows.is_empty(),
            "WASM32_IMPORT_ABI drifted from the extern block: \
             externs without a table row: {missing_rows:?}; \
             table rows without an extern: {stale_rows:?}"
        );
    }

    /// Every import must lower to a descriptor the host-mode dispatch can
    /// resolve, so `neovm_syscall` has an arm (registry entry) for it.
    #[test]
    fn abi_descriptors_have_host_dispatch_arms() {
        for row in WASM32_IMPORT_ABI {
            assert!(
                crate::SYSCALL_REGISTRY.has_syscall(row.descriptor),
                "import '{}' maps to descriptor '{}' with no host-mode \
                 dispatch arm (not in the SYSCALLS registry)",
                row.link_name,
                row.descriptor
            );
        }
    }
}

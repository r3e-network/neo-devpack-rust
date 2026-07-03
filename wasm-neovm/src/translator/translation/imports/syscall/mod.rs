// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! `neo`-module import lowering: entry points for the translator's import
//! dispatch. Grouped handlers live in the child modules: `storage` (devpack
//! storage facade + raw i64 primitives), `runtime` (log/notify events and
//! script-hash getters), `crypto` (CryptoLib native-contract routing).

use super::*;

mod crypto;
mod runtime;
mod storage;

use crypto::{emit_cryptolib_call, try_handle_crypto_import};
use runtime::{
    emit_chunked_bytes_argument, try_handle_notify_with_state_import,
    try_handle_runtime_event_import, try_handle_runtime_hash_i64_import,
};
use storage::try_handle_storage_import;

pub(in super::super) fn try_handle_neo_import(
    import: &FunctionImport,
    func_type: &FuncType,
    params: &[StackValue],
    runtime: &mut RuntimeHelpers,
    script: &mut Vec<u8>,
) -> Result<Option<&'static str>> {
    if !import.module.eq_ignore_ascii_case("neo") {
        return Ok(None);
    }

    if let Some(descriptor) = try_handle_storage_import(import, func_type, runtime, script)? {
        return Ok(Some(descriptor));
    }

    if let Some(descriptor) =
        try_handle_runtime_event_import(import, func_type, params, runtime, script)?
    {
        return Ok(Some(descriptor));
    }

    if let Some(descriptor) =
        try_handle_notify_with_state_import(import, func_type, runtime, script)?
    {
        return Ok(Some(descriptor));
    }

    if let Some(descriptor) = try_handle_runtime_hash_i64_import(import, func_type, script)? {
        return Ok(Some(descriptor));
    }

    // CryptoLib methods: full buffer-ABI marshalling + System.Contract.Call
    // (this path has RuntimeHelpers access; the fall-through
    // `emit_neo_syscall` legacy path does not and cannot marshal).
    if let Some(descriptor) = try_handle_crypto_import(import, func_type, runtime, script)? {
        return Ok(Some(descriptor));
    }

    let is_witness_bytes = import
        .name
        .eq_ignore_ascii_case("runtime_check_witness_bytes");
    let is_witness_i64 = import
        .name
        .eq_ignore_ascii_case("runtime_check_witness_i64");
    if !is_witness_bytes && !is_witness_i64 {
        return Ok(None);
    }

    if is_witness_i64 {
        if func_type.params() != [ValType::I64] {
            bail!(
                "neo import '{}::{}' expects a single i64 account parameter",
                import.module,
                import.name
            );
        }
        if func_type.results() != [ValType::I32] {
            bail!(
                "neo import '{}::{}' must return a single i32",
                import.module,
                import.name
            );
        }

        let convert =
            opcodes::lookup("CONVERT").ok_or_else(|| anyhow!("CONVERT opcode metadata missing"))?;
        if convert.operand_size != 1 || convert.operand_size_prefix != 0 {
            bail!("unexpected CONVERT operand metadata");
        }
        const STACKITEMTYPE_BYTESTRING: u8 = 0x28;
        script.push(convert.byte);
        script.push(STACKITEMTYPE_BYTESTRING);
        emit_push_data(script, &[0u8; 19])?;
        script.push(op::CAT);

        let syscall = syscalls::lookup_extended("System.Runtime.CheckWitness")
            .ok_or_else(|| anyhow!("syscall 'System.Runtime.CheckWitness' not found"))?;
        let syscall_op =
            opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;
        if syscall_op.operand_size != 4 || syscall_op.operand_size_prefix != 0 {
            bail!("unexpected SYSCALL operand metadata");
        }
        script.push(syscall_op.byte);
        script.extend_from_slice(&syscall.hash.to_le_bytes());
        return Ok(Some(syscall.name));
    }

    if func_type.params() != [ValType::I32, ValType::I32] {
        bail!(
            "neo import '{}::{}' expects i32 pointer and i32 length parameters",
            import.module,
            import.name
        );
    }
    if func_type.results() != [ValType::I32] {
        bail!(
            "neo import '{}::{}' must return a single i32",
            import.module,
            import.name
        );
    }

    let syscall = syscalls::lookup_extended("System.Runtime.CheckWitness")
        .ok_or_else(|| anyhow!("syscall 'System.Runtime.CheckWitness' not found"))?;
    let syscall_op =
        opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;
    if syscall_op.operand_size != 4 || syscall_op.operand_size_prefix != 0 {
        bail!("unexpected SYSCALL operand metadata");
    }
    let embedded_static_bytes = params
        .first()
        .and_then(|param| param.const_value)
        .zip(params.get(1).and_then(|param| param.const_value))
        .and_then(|(ptr, len)| {
            let ptr = usize::try_from(ptr).ok()?;
            let len = usize::try_from(len).ok()?;
            runtime.active_data_slice(ptr, len)
        });

    if let Some(bytes) = embedded_static_bytes {
        emit_push_data(script, bytes)?;
    } else {
        emit_chunked_bytes_argument(runtime, script)?;
    }
    script.push(syscall_op.byte);
    script.extend_from_slice(&syscall.hash.to_le_bytes());

    Ok(Some(syscall.name))
}

pub(super) fn emit_syscall_call(
    import: &FunctionImport,
    script: &mut Vec<u8>,
) -> Result<&'static str> {
    let syscall = syscalls::lookup(&import.name)
        .ok_or_else(|| anyhow!("unknown syscall '{}'", import.name))?;
    let opcode =
        opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;

    // SYSCALL has a 4-byte immediate hash.
    if opcode.operand_size != 4 || opcode.operand_size_prefix != 0 {
        bail!("unexpected SYSCALL operand metadata");
    }

    script.push(opcode.byte);
    script.extend_from_slice(&syscall.hash.to_le_bytes());
    Ok(syscall.name)
}

pub(super) fn emit_descriptor_syscall(
    descriptor: &str,
    script: &mut Vec<u8>,
) -> Result<&'static str> {
    // CryptoLib methods must be invoked via System.Contract.Call, not a bare
    // SYSCALL (no Register("Neo.Crypto.*") interop exists on Neo N3).
    if let Some(method) = crate::native_contracts::crypto_lib_method(descriptor) {
        return emit_cryptolib_call(script, method.contract_hash, method.method, descriptor);
    }
    // Composite Neo.Crypto.Hash160/Hash256 have no single native method; lower
    // them as explicit call sequences elsewhere, not a dead syscall.
    if crate::native_contracts::is_crypto_alias(descriptor) {
        bail!(
            "{descriptor} is a composite hash with no single CryptoLib method; \
             lower it explicitly as sha256+ripemd160 (Hash160) or double-sha256 (Hash256)"
        );
    }
    let syscall = syscalls::lookup_extended(descriptor)
        .ok_or_else(|| anyhow!("syscall '{}' not found", descriptor))?;
    let opcode =
        opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;

    if opcode.operand_size != 4 || opcode.operand_size_prefix != 0 {
        bail!("unexpected SYSCALL operand metadata");
    }

    script.push(opcode.byte);
    script.extend_from_slice(&syscall.hash.to_le_bytes());
    Ok(syscall.name)
}

pub(super) fn emit_neo_syscall(
    import: &FunctionImport,
    script: &mut Vec<u8>,
) -> Result<&'static str> {
    if import
        .name
        .eq_ignore_ascii_case("runtime_check_witness_hash")
    {
        let convert =
            opcodes::lookup("CONVERT").ok_or_else(|| anyhow!("CONVERT opcode metadata missing"))?;
        if convert.operand_size != 1 || convert.operand_size_prefix != 0 {
            bail!("unexpected CONVERT operand metadata");
        }
        // NeoVM StackItemType.ByteString
        const STACKITEMTYPE_BYTESTRING: u8 = 0x28;
        script.push(convert.byte);
        script.push(STACKITEMTYPE_BYTESTRING);

        let syscall = syscalls::lookup_extended("System.Runtime.CheckWitness")
            .ok_or_else(|| anyhow!("syscall 'System.Runtime.CheckWitness' not found"))?;
        let syscall_op =
            opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;
        if syscall_op.operand_size != 4 || syscall_op.operand_size_prefix != 0 {
            bail!("unexpected SYSCALL operand metadata");
        }
        script.push(syscall_op.byte);
        script.extend_from_slice(&syscall.hash.to_le_bytes());
        return Ok(syscall.name);
    }

    let syscall_name = neo_syscalls::lookup_neo_syscall(&import.name)
        .ok_or_else(|| anyhow!("unknown Neo syscall import '{}'", import.name))?;

    // CryptoLib methods (sha256/ripemd160/keccak256/murmur32/verifyWithECDsa)
    // live on the CryptoLib *native contract* and must be invoked via
    // `System.Contract.Call`. There is no `Register("Neo.Crypto.*")` interop,
    // so emitting `SYSCALL <Neo.Crypto hash>` would deploy but fault at the
    // first execution with "InteropService not found". Route these aliases to
    // a real contract call instead of a dead syscall.
    if let Some(method) = crate::native_contracts::crypto_lib_method(syscall_name) {
        return emit_cryptolib_call(script, method.contract_hash, method.method, &import.name);
    }
    if crate::native_contracts::is_crypto_alias(syscall_name) {
        bail!(
            "{syscall_name} (import '{}') is a composite hash with no single \
             CryptoLib method; lower it explicitly as sha256+ripemd160 (Hash160) \
             or double-sha256 (Hash256)",
            import.name
        );
    }

    let syscall = syscalls::lookup_extended(syscall_name)
        .ok_or_else(|| anyhow!("syscall '{}' not found", syscall_name))?;
    let opcode =
        opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;

    if opcode.operand_size != 4 || opcode.operand_size_prefix != 0 {
        bail!("unexpected SYSCALL operand metadata");
    }

    script.push(opcode.byte);
    script.extend_from_slice(&syscall.hash.to_le_bytes());
    Ok(syscall.name)
}

/// The witness import names handled directly by `try_handle_neo_import` /
/// `emit_neo_syscall` above (case-insensitively), with the descriptor each
/// lowers to. Keep in lock-step with those name checks; the ABI parity test
/// below drives from this list.
#[cfg(test)]
const HANDLED_WITNESS_IMPORTS: &[(&str, &str)] = &[
    ("runtime_check_witness_bytes", "System.Runtime.CheckWitness"),
    ("runtime_check_witness_i64", "System.Runtime.CheckWitness"),
    ("runtime_check_witness_hash", "System.Runtime.CheckWitness"),
];

/// Resolve the Neo syscall descriptor this translator lowers a `neo`-module
/// import name to, mirroring the dispatch order of `try_handle_neo_import`
/// (specially-handled names first) followed by `emit_neo_syscall`'s
/// `lookup_neo_syscall` fallback. Returns `None` for unrecognized names.
#[cfg(test)]
fn recognized_neo_import_descriptor(name: &str) -> Option<&'static str> {
    HANDLED_WITNESS_IMPORTS
        .iter()
        .chain(runtime::HANDLED_IMPORTS)
        .chain(storage::HANDLED_IMPORTS)
        .chain(crypto::HANDLED_IMPORTS)
        .find(|(handled, _)| handled.eq_ignore_ascii_case(name))
        .map(|(_, descriptor)| *descriptor)
        .or_else(|| crate::neo_syscalls::lookup_neo_syscall(name))
}

/// ABI parity gate: the wasm32 extern import surface declared by the SDK
/// (`rust-devpack/neo-syscalls/src/syscalls_abi.rs`, mirrored by
/// `neo_syscalls::WASM32_IMPORT_ABI` via the dev-dependency edge) must be
/// recognized by this translator, and must lower to the descriptor the SDK
/// documents. The two sides of the syscall bridge have historically drifted
/// apart (missing buffer-ABI marshalling, alias reachability, name-only
/// events); this test makes NEW drift a loud failure while keeping today's
/// known gaps visible in an explicit allowlist.
#[cfg(test)]
mod abi_parity_tests {
    use super::*;

    /// KNOWN gaps: SDK externs the translator does not recognize today.
    /// Every entry is a documented TODO, not a grandfathered pass — the
    /// `known_gap_allowlist_is_not_stale` test fails as soon as one of these
    /// becomes recognized, forcing its removal from this list.
    const KNOWN_UNRECOGNIZED_IMPORTS: &[&str] = &[
        // Buffer-ABI account constructors: the SDK wrappers call these, but
        // no alias/canonical name reaches them (`create_standard_account` is
        // the reachable alias; the `runtime_`-prefixed extern names are not).
        "runtime_create_standard_account",
        "runtime_create_multisig_account",
        // Superseded by `neo_call_native`; the SDK wrapper never calls it.
        "runtime_contract_call_native",
        // Reserved storage-context ABI: the SDK wrappers are stubs (sentinel
        // contexts; the translator emits a fresh `System.Storage.GetContext`
        // inside every storage helper), and the translator has no lowering
        // for them yet.
        "runtime_get_storage_context",
        "runtime_get_read_only_context",
        "runtime_storage_as_read_only",
    ];

    /// Every wasm32 extern the SDK declares must be recognized by the
    /// translator and lower to the descriptor the SDK documents for it —
    /// unless it is an explicitly allowlisted known gap.
    #[test]
    fn translator_recognizes_every_sdk_wasm32_import() {
        for row in ::neo_syscalls::WASM32_IMPORT_ABI {
            if KNOWN_UNRECOGNIZED_IMPORTS.contains(&row.link_name) {
                continue;
            }
            match recognized_neo_import_descriptor(row.link_name) {
                None => panic!(
                    "SDK wasm32 import '{}' is not recognized by the translator \
                     (new bridge drift): add a handler/alias, or document it in \
                     KNOWN_UNRECOGNIZED_IMPORTS if it is intentionally deferred",
                    row.link_name
                ),
                Some(descriptor) => assert_eq!(
                    descriptor, row.descriptor,
                    "SDK wasm32 import '{}' lowers to '{}' but the SDK ABI \
                     table documents '{}'",
                    row.link_name, descriptor, row.descriptor
                ),
            }
        }
    }

    /// The allowlist must only contain imports that (a) still exist in the
    /// SDK ABI table and (b) are still unrecognized — fixed gaps must be
    /// removed so the list never silently grandfathers regressions.
    #[test]
    fn known_gap_allowlist_is_not_stale() {
        for name in KNOWN_UNRECOGNIZED_IMPORTS {
            assert!(
                ::neo_syscalls::WASM32_IMPORT_ABI
                    .iter()
                    .any(|row| row.link_name == *name),
                "allowlisted import '{name}' no longer exists in the SDK ABI table"
            );
            assert!(
                recognized_neo_import_descriptor(name).is_none(),
                "allowlisted import '{name}' is now recognized by the \
                 translator — remove it from KNOWN_UNRECOGNIZED_IMPORTS"
            );
        }
    }
}

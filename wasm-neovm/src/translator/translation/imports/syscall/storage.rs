// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Storage import handlers: the devpack's pointer/length-encoded storage
//! facade and the raw i64-keyed storage primitives.

use super::*;

/// The import names the two handlers below match, with the descriptor each
/// lowers to. Keep in lock-step with the `match` arms in
/// `try_handle_storage_import` / `try_handle_direct_i64_storage_import`;
/// the ABI parity test in `super` drives from this list.
#[cfg(test)]
pub(super) const HANDLED_IMPORTS: &[(&str, &str)] = &[
    ("neo_storage_put_bytes", "System.Storage.Put"),
    ("neo_storage_delete_bytes", "System.Storage.Delete"),
    ("neo_storage_get_into", "System.Storage.Get"),
    ("raw_storage_put_i64", "System.Storage.Put"),
    ("raw_storage_get_i64", "System.Storage.Get"),
    ("raw_storage_has_i64", "System.Storage.Get"),
    ("raw_storage_delete_i64", "System.Storage.Delete"),
    ("runtime_storage_find", "System.Storage.Find"),
    ("runtime_iterator_next", "System.Iterator.Next"),
    ("runtime_iterator_value", "System.Iterator.Value"),
];

/// Recognize the devpack's pointer/length-encoded storage primitives and emit
/// a `CALL_L` to the shared marshalling helper. Returns the underlying Neo
/// SYSCALL descriptor so feature tracking marks the contract as storage-using.
pub(super) fn try_handle_storage_import(
    import: &FunctionImport,
    func_type: &FuncType,
    runtime: &mut RuntimeHelpers,
    script: &mut Vec<u8>,
) -> Result<Option<&'static str>> {
    if let Some(descriptor) =
        try_handle_direct_i64_storage_import(import, func_type, runtime, script)?
    {
        return Ok(Some(descriptor));
    }

    if let Some(descriptor) =
        try_handle_storage_iterator_import(import, func_type, runtime, script)?
    {
        return Ok(Some(descriptor));
    }

    let (helper_kind, descriptor, expected_params) = match import.name.as_str() {
        "neo_storage_put_bytes" => (
            crate::translator::runtime::StorageHelperKind::PutBytes,
            "System.Storage.Put",
            &[ValType::I32, ValType::I32, ValType::I32, ValType::I32][..],
        ),
        "neo_storage_delete_bytes" => (
            crate::translator::runtime::StorageHelperKind::DeleteBytes,
            "System.Storage.Delete",
            &[ValType::I32, ValType::I32][..],
        ),
        "neo_storage_get_into" => (
            crate::translator::runtime::StorageHelperKind::GetInto,
            "System.Storage.Get",
            &[ValType::I32, ValType::I32, ValType::I32, ValType::I32][..],
        ),
        _ => return Ok(None),
    };

    if func_type.params() != expected_params {
        bail!(
            "neo import '{}::{}' has unexpected parameter signature",
            import.module,
            import.name
        );
    }

    let expected_results: &[ValType] = match helper_kind {
        crate::translator::runtime::StorageHelperKind::GetInto => &[ValType::I32],
        _ => &[],
    };
    if func_type.results() != expected_results {
        bail!(
            "neo import '{}::{}' has unexpected result signature",
            import.module,
            import.name
        );
    }

    ensure_memory_access(runtime, 0)?;
    runtime.emit_memory_init_call(script)?;
    runtime.emit_storage_helper(script, helper_kind)?;
    Ok(Some(descriptor))
}

/// Recognize the prefix-scan/iterator bridge imports (see the SDK extern
/// docs in `neo-syscalls/src/syscalls_abi.rs` for the single-live-iterator
/// model and the flattened `key_len(4B LE) || key || value` element ABI).
/// Each lowers to a `CALL_L` into the matching shared helper; the helpers
/// share one dedicated static slot that parks the live iterator
/// InteropInterface (assigned during finalize, see
/// `RuntimeHelpers::storage_iterator_slot`).
fn try_handle_storage_iterator_import(
    import: &FunctionImport,
    func_type: &FuncType,
    runtime: &mut RuntimeHelpers,
    script: &mut Vec<u8>,
) -> Result<Option<&'static str>> {
    match import.name.as_str() {
        "runtime_storage_find" => {
            if func_type.params() != [ValType::I32, ValType::I32] {
                bail!(
                    "neo import '{}::{}' expects (prefix_ptr, prefix_len) i32 parameters",
                    import.module,
                    import.name
                );
            }
            if !func_type.results().is_empty() {
                bail!(
                    "neo import '{}::{}' must not return a value",
                    import.module,
                    import.name
                );
            }
            ensure_memory_access(runtime, 0)?;
            runtime.emit_memory_init_call(script)?;
            runtime.emit_storage_helper(script, crate::translator::runtime::StorageHelperKind::Find)?;
            Ok(Some("System.Storage.Find"))
        }
        "runtime_iterator_next" => {
            if !func_type.params().is_empty() {
                bail!(
                    "neo import '{}::{}' must not take parameters",
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
            // No memory marshalling, but the init guard ensures INITSSLOT
            // has allocated the iterator static slot before it is read.
            runtime.emit_memory_init_call(script)?;
            runtime.emit_storage_helper(
                script,
                crate::translator::runtime::StorageHelperKind::IteratorNext,
            )?;
            Ok(Some("System.Iterator.Next"))
        }
        "runtime_iterator_value" => {
            if func_type.params() != [ValType::I32, ValType::I32] {
                bail!(
                    "neo import '{}::{}' expects (out_ptr, out_cap) i32 parameters",
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
            ensure_memory_access(runtime, 0)?;
            runtime.emit_memory_init_call(script)?;
            runtime.emit_storage_helper(
                script,
                crate::translator::runtime::StorageHelperKind::IteratorValue,
            )?;
            Ok(Some("System.Iterator.Value"))
        }
        _ => Ok(None),
    }
}

fn try_handle_direct_i64_storage_import(
    import: &FunctionImport,
    func_type: &FuncType,
    runtime: &mut RuntimeHelpers,
    script: &mut Vec<u8>,
) -> Result<Option<&'static str>> {
    match import.name.as_str() {
        "raw_storage_put_i64" => {
            if func_type.params() != [ValType::I64, ValType::I64] {
                bail!(
                    "neo import '{}::{}' expects (i64 key, i64 value)",
                    import.module,
                    import.name
                );
            }
            if !func_type.results().is_empty() {
                bail!(
                    "neo import '{}::{}' must not return a value",
                    import.module,
                    import.name
                );
            }
            runtime.emit_storage_helper(
                script,
                crate::translator::runtime::StorageHelperKind::PutI64,
            )?;
            Ok(Some("System.Storage.Put"))
        }
        "raw_storage_get_i64" => {
            if func_type.params() != [ValType::I64] {
                bail!(
                    "neo import '{}::{}' expects a single i64 key",
                    import.module,
                    import.name
                );
            }
            if func_type.results() != [ValType::I64] {
                bail!(
                    "neo import '{}::{}' must return a single i64",
                    import.module,
                    import.name
                );
            }
            runtime.emit_storage_helper(
                script,
                crate::translator::runtime::StorageHelperKind::GetI64,
            )?;
            Ok(Some("System.Storage.Get"))
        }
        "raw_storage_has_i64" => {
            if func_type.params() != [ValType::I64] {
                bail!(
                    "neo import '{}::{}' expects a single i64 key",
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
            runtime.emit_storage_helper(
                script,
                crate::translator::runtime::StorageHelperKind::HasI64,
            )?;
            Ok(Some("System.Storage.Get"))
        }
        "raw_storage_delete_i64" => {
            if func_type.params() != [ValType::I64] {
                bail!(
                    "neo import '{}::{}' expects a single i64 key",
                    import.module,
                    import.name
                );
            }
            if !func_type.results().is_empty() {
                bail!(
                    "neo import '{}::{}' must not return a value",
                    import.module,
                    import.name
                );
            }
            runtime.emit_storage_helper(
                script,
                crate::translator::runtime::StorageHelperKind::DeleteI64,
            )?;
            Ok(Some("System.Storage.Delete"))
        }
        _ => Ok(None),
    }
}

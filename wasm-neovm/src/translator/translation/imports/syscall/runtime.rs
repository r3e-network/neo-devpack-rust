// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Runtime-event (`log`/`notify`) and script-hash-getter import handlers,
//! plus the shared memory-bytes argument marshalling they use.

use super::*;

/// The import names the two handlers below match (case-insensitively), with
/// the descriptor each lowers to. Keep in lock-step with the name checks in
/// `try_handle_runtime_hash_i64_import` / `try_handle_runtime_event_import`;
/// the ABI parity test in `super` drives from this list.
#[cfg(test)]
pub(super) const HANDLED_IMPORTS: &[(&str, &str)] = &[
    ("log", "System.Runtime.Log"),
    ("runtime_log", "System.Runtime.Log"),
    ("notify", "System.Runtime.Notify"),
    ("runtime_notify", "System.Runtime.Notify"),
    (
        "runtime_get_calling_script_hash_i64",
        "System.Runtime.GetCallingScriptHash",
    ),
    (
        "runtime_get_entry_script_hash_i64",
        "System.Runtime.GetEntryScriptHash",
    ),
    (
        "runtime_get_executing_script_hash_i64",
        "System.Runtime.GetExecutingScriptHash",
    ),
];

pub(super) fn try_handle_runtime_hash_i64_import(
    import: &FunctionImport,
    func_type: &FuncType,
    script: &mut Vec<u8>,
) -> Result<Option<&'static str>> {
    let descriptor = if import
        .name
        .eq_ignore_ascii_case("runtime_get_calling_script_hash_i64")
    {
        "System.Runtime.GetCallingScriptHash"
    } else if import
        .name
        .eq_ignore_ascii_case("runtime_get_entry_script_hash_i64")
    {
        "System.Runtime.GetEntryScriptHash"
    } else if import
        .name
        .eq_ignore_ascii_case("runtime_get_executing_script_hash_i64")
    {
        "System.Runtime.GetExecutingScriptHash"
    } else {
        return Ok(None);
    };

    if !func_type.params().is_empty() {
        bail!(
            "neo import '{}::{}' must not take parameters",
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

    emit_descriptor_syscall(descriptor, script)?;
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("PUSH8")?.byte);
    script.push(lookup_opcode("SUBSTR")?.byte);

    let convert =
        opcodes::lookup("CONVERT").ok_or_else(|| anyhow!("CONVERT opcode metadata missing"))?;
    if convert.operand_size != 1 || convert.operand_size_prefix != 0 {
        bail!("unexpected CONVERT operand metadata");
    }
    const STACKITEM_TYPE_INTEGER: u8 = 0x21;
    script.push(convert.byte);
    script.push(STACKITEM_TYPE_INTEGER);

    let syscall = syscalls::lookup_extended(descriptor)
        .ok_or_else(|| anyhow!("syscall '{}' not found", descriptor))?;
    Ok(Some(syscall.name))
}

pub(super) fn try_handle_runtime_event_import(
    import: &FunctionImport,
    func_type: &FuncType,
    params: &[StackValue],
    runtime: &mut RuntimeHelpers,
    script: &mut Vec<u8>,
) -> Result<Option<&'static str>> {
    let descriptor = if import.name.eq_ignore_ascii_case("log")
        || import.name.eq_ignore_ascii_case("runtime_log")
    {
        "System.Runtime.Log"
    } else if import.name.eq_ignore_ascii_case("notify")
        || import.name.eq_ignore_ascii_case("runtime_notify")
    {
        "System.Runtime.Notify"
    } else {
        return Ok(None);
    };

    if func_type.params() != [ValType::I32, ValType::I32] {
        bail!(
            "neo import '{}::{}' expects i32 pointer and i32 length parameters",
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

    emit_memory_bytes_argument(params, runtime, script)?;
    if descriptor == "System.Runtime.Notify" {
        script.push(lookup_opcode("NEWARRAY0")?.byte);
    }

    let syscall = syscalls::lookup_extended(descriptor)
        .ok_or_else(|| anyhow!("syscall '{}' not found", descriptor))?;
    let syscall_op =
        opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;
    if syscall_op.operand_size != 4 || syscall_op.operand_size_prefix != 0 {
        bail!("unexpected SYSCALL operand metadata");
    }
    script.push(syscall_op.byte);
    script.extend_from_slice(&syscall.hash.to_le_bytes());
    Ok(Some(syscall.name))
}

fn emit_memory_bytes_argument(
    params: &[StackValue],
    runtime: &mut RuntimeHelpers,
    script: &mut Vec<u8>,
) -> Result<()> {
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

    Ok(())
}

/// Marshal a runtime-built `(ptr, len)` byte-slice argument (already on the
/// NeoVM stack, `ptr` then `len`) into a single `ByteString` left on the stack.
///
/// This replaces the old `LDSFLD0; REVERSE3; SWAP; SUBSTR` flat-bytestring
/// sequence, which faulted (`invalid conversion: Array/ByteString`) under the
/// chunked memory layout because `LDSFLD0` yields an `Array` of page `Buffer`s,
/// not a flat `ByteString`. The work is delegated to a shared
/// `ExtractMemoryBytes` runtime helper (see
/// `runtime::storage::emit_extract_memory_bytes_helper`) that is correct for
/// both the compact and chunked layouts and is `CALL_L`'d here — so it needs no
/// scratch local slots in the *caller's* frame (the helper allocates its own
/// via `INITSLOT`, avoiding any collision with the contract function's locals).
pub(super) fn emit_chunked_bytes_argument(
    runtime: &mut RuntimeHelpers,
    script: &mut Vec<u8>,
) -> Result<()> {
    ensure_memory_access(runtime, 0)?;
    runtime.emit_memory_init_call(script)?;
    runtime.emit_storage_helper(
        script,
        crate::translator::runtime::StorageHelperKind::ExtractMemoryBytes,
    )?;
    Ok(())
}

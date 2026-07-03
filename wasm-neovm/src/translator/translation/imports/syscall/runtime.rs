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
    ("runtime_notify_with_state", "System.Runtime.Notify"),
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
    script.push(op::PUSH0);
    script.push(op::PUSH8);
    script.push(op::SUBSTR);

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
        // `System.Runtime.Notify` pops the event NAME first, then the state
        // array (neo-go runtime/engine.go `Notify`), so the name must end up
        // on TOP. `NEWARRAY0` lands the empty state above the just-marshalled
        // name; SWAP restores (name, state) pop order. Without the SWAP the
        // VM pops the Array as the name and faults with
        // "invalid conversion: Array/ByteString".
        script.push(op::NEWARRAY0);
        script.push(op::SWAP);
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

/// Lower the SDK's state-carrying notify import,
/// `runtime_notify_with_state(event_ptr, event_len, state_ptr, state_len)`.
///
/// The SDK (`NeoRuntime::notify` in `neo-syscalls/src/wrapper.rs`) serialises
/// the state array with `neo_types::serialise_array`, which is byte-for-byte
/// the canonical NeoVM `BinarySerializer` wire format (C#
/// `Neo/SmartContract/BinarySerializer.cs`; neo-go
/// `pkg/vm/stackitem/serialization.go`). That means the buffer sitting in
/// wasm linear memory can be decoded on-VM by the StdLib native contract's
/// `deserialize` method — no bespoke TLV decoder in emitted NeoVM code and
/// no SDK wire-format change (host-mode behaviour is untouched).
///
/// Emitted sequence (entry stack, top→bottom: `state_len, state_ptr,
/// event_len, event_ptr` — wasm push order):
///  1. `ExtractMemoryBytes` gathers the state buffer out of linear memory
///     (correct for both the compact and chunked memory layouts, the same
///     helper the storage facade and `check_witness` marshalling use).
///  2. `PUSH1; PACK` wraps it as the single `System.Contract.Call` argument,
///     then `deserialize` is invoked on the StdLib native. Stack argument
///     order mirrors neo-go `pkg/core/interop/contract/call.go::Call`, which
///     pops (hash, method, callFlags, args) top-first.
///  3. Two `ROT`s bring the event-name `(ptr, len)` pair back on top (len
///     above ptr, the `ExtractMemoryBytes` convention) and the name is
///     gathered the same way.
///  4. `SYSCALL System.Runtime.Notify` pops (name, state) top-first — the
///     name is on top, the decoded state `Array` beneath it.
///
/// Runtime requirements on-chain: the calling context needs
/// `CallFlags.ReadStates|AllowCall` for the `System.Contract.Call` hop (the
/// manifest permission for `StdLib.deserialize` is auto-inserted by the
/// finalizer — see `RuntimeHelpers::stdlib_deserialize_used`).
pub(super) fn try_handle_notify_with_state_import(
    import: &FunctionImport,
    func_type: &FuncType,
    runtime: &mut RuntimeHelpers,
    script: &mut Vec<u8>,
) -> Result<Option<&'static str>> {
    if !import
        .name
        .eq_ignore_ascii_case("runtime_notify_with_state")
    {
        return Ok(None);
    }

    if func_type.params() != [ValType::I32, ValType::I32, ValType::I32, ValType::I32] {
        bail!(
            "neo import '{}::{}' expects (event_ptr, event_len, state_ptr, state_len) i32 parameters",
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

    // 1. (state_ptr, state_len) -> ByteString of the serialised state.
    runtime.emit_storage_helper(
        script,
        crate::translator::runtime::StorageHelperKind::ExtractMemoryBytes,
    )?;

    // 2. Decode it into the state Array via StdLib.deserialize.
    emit_stdlib_deserialize_call(script)?;
    runtime.mark_stdlib_deserialize_used();

    // 3. (event_ptr, event_len) -> ByteString of the event name. The pair
    //    sits beneath the decoded array; two ROTs restore (len, ptr) on top.
    script.push(op::ROT);
    script.push(op::ROT);
    runtime.emit_storage_helper(
        script,
        crate::translator::runtime::StorageHelperKind::ExtractMemoryBytes,
    )?;

    // 4. Notify pops (name, state) top-first; name is on top now.
    let syscall = syscalls::lookup_extended("System.Runtime.Notify")
        .ok_or_else(|| anyhow!("syscall 'System.Runtime.Notify' not found"))?;
    let syscall_op =
        opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;
    if syscall_op.operand_size != 4 || syscall_op.operand_size_prefix != 0 {
        bail!("unexpected SYSCALL operand metadata");
    }
    script.push(syscall_op.byte);
    script.extend_from_slice(&syscall.hash.to_le_bytes());
    Ok(Some(syscall.name))
}

/// Emit `System.Contract.Call(StdLib, "deserialize", ReadOnly, [top-of-stack])`.
///
/// Expects the serialised `ByteString` on top of the stack; leaves the
/// deserialized stack item in its place. Stack order mirrors neo-go
/// `contract/call.go::Call`'s pop order — (hash, method, callFlags, args)
/// top-first — so the args array is pushed first (deepest) and the contract
/// hash last (top). `CallFlags.ReadOnly = ReadStates | AllowCall = 0b0101`
/// is least privilege for the pure `deserialize` native method (required
/// flags: none).
fn emit_stdlib_deserialize_call(script: &mut Vec<u8>) -> Result<()> {
    const CALLFLAGS_READ_ONLY: i128 = 0b0101;

    // args = [serialised_state]
    let _ = emit_push_int(script, 1);
    script.push(op::PACK);
    // callFlags, method, contractHash (LE bytes, as Contract.Call consumes).
    let _ = emit_push_int(script, CALLFLAGS_READ_ONLY);
    emit_push_data(script, b"deserialize")?;
    emit_push_data(script, &crate::native_contracts::STDLIB_DESCRIPTOR.hash)?;

    let call_syscall = syscalls::lookup_extended("System.Contract.Call")
        .ok_or_else(|| anyhow!("System.Contract.Call syscall not found"))?;
    let syscall_op =
        opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;
    if syscall_op.operand_size != 4 || syscall_op.operand_size_prefix != 0 {
        bail!("unexpected SYSCALL operand metadata");
    }
    script.push(syscall_op.byte);
    script.extend_from_slice(&call_syscall.hash.to_le_bytes());
    Ok(())
}

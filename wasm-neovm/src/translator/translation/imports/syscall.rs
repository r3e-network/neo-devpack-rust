// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

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
        script.push(lookup_opcode("CAT")?.byte);

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
        script.push(syscall_op.byte);
        script.extend_from_slice(&syscall.hash.to_le_bytes());
    } else {
        ensure_memory_access(runtime, 0)?;
        runtime.emit_memory_init_call(script)?;

        script.push(lookup_opcode("LDSFLD0")?.byte);
        script.push(lookup_opcode("REVERSE3")?.byte);
        script.push(lookup_opcode("SWAP")?.byte);
        script.push(lookup_opcode("SUBSTR")?.byte);
        script.push(syscall_op.byte);
        script.extend_from_slice(&syscall.hash.to_le_bytes());
    }

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

/// Recognize the devpack's pointer/length-encoded storage primitives and emit
/// a `CALL_L` to the shared marshalling helper. Returns the underlying Neo
/// SYSCALL descriptor so feature tracking marks the contract as storage-using.
fn try_handle_storage_import(
    import: &FunctionImport,
    func_type: &FuncType,
    runtime: &mut RuntimeHelpers,
    script: &mut Vec<u8>,
) -> Result<Option<&'static str>> {
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

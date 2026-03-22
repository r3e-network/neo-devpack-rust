// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

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

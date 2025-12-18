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

// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::super::*;

pub(in crate::translator::runtime) fn emit_memory_load_helper(
    script: &mut Vec<u8>,
    bytes: u32,
) -> Result<()> {
    let bytes_i128 = bytes as i128;

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_negative = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("DUP")?.byte);
    let _ = emit_push_int(script, bytes_i128);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);
    let _ = emit_push_int(script, bytes_i128);
    script.push(lookup_opcode("SUBSTR")?.byte);
    script.push(CONVERT);
    script.push(STACKITEMTYPE_INTEGER);
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);

    patch_jump(script, trap_negative, trap_label)?;
    patch_jump(script, trap_oob, trap_label)?;

    Ok(())
}

pub(in crate::translator::runtime) fn emit_memory_store_helper(
    script: &mut Vec<u8>,
    bytes: u32,
) -> Result<()> {
    let bytes_i128 = bytes as i128;

    script.push(lookup_opcode("SWAP")?.byte);

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let trap_negative = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("DUP")?.byte);
    let _ = emit_push_int(script, bytes_i128);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("SWAP")?.byte);
    // Mask value to byte width: compute (1 << (bytes*8)) - 1 inline
    let bit_width = bytes * 8;
    let _ = emit_push_int(script, 1);
    let _ = emit_push_int(script, bit_width as i128);
    script.push(lookup_opcode("SHL")?.byte);
    script.push(lookup_opcode("DEC")?.byte);
    script.push(lookup_opcode("AND")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);

    for i in 0..bytes {
        script.push(lookup_opcode("OVER")?.byte);
        let shift = i * 8;
        if shift > 0 {
            let _ = emit_push_int(script, shift as i128);
            script.push(lookup_opcode("SHR")?.byte);
        }
        let _ = emit_push_int(script, 0xFF);
        script.push(lookup_opcode("AND")?.byte);
        script.push(lookup_opcode("OVER")?.byte);
        if i > 0 {
            let _ = emit_push_int(script, i as i128);
            script.push(lookup_opcode("ADD")?.byte);
        }
        script.push(lookup_opcode("SWAP")?.byte);
        script.push(lookup_opcode("LDSFLD0")?.byte);
        script.push(lookup_opcode("ROT")?.byte);
        script.push(lookup_opcode("SETITEM")?.byte);
    }

    script.push(lookup_opcode("DROP")?.byte);
    script.push(lookup_opcode("DROP")?.byte);
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);
    patch_jump(script, trap_negative, trap_label)?;
    patch_jump(script, trap_oob, trap_label)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_no_conditional_jump_drop_pair(script: &[u8], jump_opcode: u8, drop_opcode: u8) {
        // Walk instruction boundaries to check if a conditional jump is followed by DROP.
        // JMPIF_L is 5 bytes (opcode + 4-byte offset), so we skip the operand.
        let mut pc = 0usize;
        while pc < script.len() {
            let op = script[pc];
            let info = crate::opcodes::lookup_by_byte(op);
            let size = match info {
                Some(i) if i.operand_size_prefix == 0 => 1 + i.operand_size as usize,
                _ => 1,
            };
            if op == jump_opcode && pc + size < script.len() && script[pc + size] == drop_opcode {
                panic!(
                    "conditional jump 0x{jump_opcode:02x} at offset {pc} followed by DROP at offset {}",
                    pc + size
                );
            }
            pc += size;
        }
    }

    #[test]
    fn memory_load_helper_does_not_drop_after_jump() {
        let mut script = Vec::new();
        emit_memory_load_helper(&mut script, 4).expect("emit helper");

        let jmpif_l = lookup_opcode("JMPIF_L").unwrap().byte;
        let drop = lookup_opcode("DROP").unwrap().byte;
        assert_no_conditional_jump_drop_pair(&script, jmpif_l, drop);
    }

    #[test]
    fn memory_store_helper_does_not_drop_after_jump() {
        let mut script = Vec::new();
        emit_memory_store_helper(&mut script, 4).expect("emit helper");

        let jmpif_l = lookup_opcode("JMPIF_L").unwrap().byte;
        let drop = lookup_opcode("DROP").unwrap().byte;
        assert_no_conditional_jump_drop_pair(&script, jmpif_l, drop);
    }
}

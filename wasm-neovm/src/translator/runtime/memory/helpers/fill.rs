// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::super::*;

pub(in crate::translator::runtime) fn emit_memory_fill_helper(script: &mut Vec<u8>) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(3);
    script.push(0);

    script.push(lookup_opcode("STLOC2")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    emit_mask_u32(script)?;
    script.push(lookup_opcode("STLOC2")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    emit_mask_u32(script)?;
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC1")?.byte);
    let _ = emit_push_int(script, 0xFF);
    script.push(lookup_opcode("AND")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);

    let loop_start = script.len();

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let exit_jump = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("SETITEM")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("INC")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("DEC")?.byte);
    script.push(lookup_opcode("STLOC2")?.byte);

    let loop_back = emit_jump_placeholder(script, "JMP_L")?;

    let exit_label = script.len();
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);

    patch_jump(script, trap_oob, trap_label)?;
    patch_jump(script, exit_jump, exit_label)?;
    patch_jump(script, loop_back, loop_start)?;
    Ok(())
}

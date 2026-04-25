// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

pub(in super::super) fn emit_table_grow_helper(
    script: &mut Vec<u8>,
    table_slot: usize,
    maximum: Option<usize>,
) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(5);
    script.push(0);

    script.push(lookup_opcode("STLOC1")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);
    emit_load_static(script, table_slot)?;
    script.push(lookup_opcode("STLOC2")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    let mask = (1u128 << 32) - 1;
    let _ = emit_push_int(script, mask as i128);
    script.push(lookup_opcode("AND")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(lookup_opcode("STLOC3")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    let exceed_jump = if let Some(maximum) = maximum {
        script.push(lookup_opcode("LDLOC3")?.byte);
        script.push(lookup_opcode("LDLOC1")?.byte);
        script.push(lookup_opcode("ADD")?.byte);
        let _ = emit_push_int(script, maximum as i128);
        script.push(lookup_opcode("GT")?.byte);
        let jump = emit_jump_placeholder(script, "JMPIF_L")?;
        Some(jump)
    } else {
        None
    };

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("STLOC4")?.byte);

    let loop_start = script.len();
    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let loop_exit = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("APPEND")?.byte);
    script.push(lookup_opcode("STLOC2")?.byte);

    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("DEC")?.byte);
    script.push(lookup_opcode("STLOC4")?.byte);
    let loop_back = emit_jump_placeholder(script, "JMP_L")?;

    let exit_label = script.len();
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(RET);
    patch_jump(script, loop_exit, exit_label)?;
    patch_jump(script, loop_back, loop_start)?;

    let zero_label = script.len();
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(RET);
    patch_jump(script, zero_branch, zero_label)?;
    if let Some(exceed_jump) = exceed_jump {
        let fail_label = script.len();
        script.push(lookup_opcode("PUSHM1")?.byte);
        script.push(RET);
        patch_jump(script, exceed_jump, fail_label)?;
    }
    Ok(())
}

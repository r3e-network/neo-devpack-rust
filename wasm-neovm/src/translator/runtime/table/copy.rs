// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

pub(in super::super) fn emit_table_copy_helper(
    script: &mut Vec<u8>,
    dst_slot: usize,
    src_slot: usize,
    mask_u32_offset: Option<usize>,
) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(7);
    script.push(0);

    script.push(lookup_opcode("STLOC2")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);
    emit_load_static(script, dst_slot)?;
    script.push(lookup_opcode("STLOC3")?.byte);
    emit_load_static(script, src_slot)?;
    script.push(lookup_opcode("STLOC4")?.byte);
    script.push(lookup_opcode("NEWARRAY0")?.byte);
    script.push(lookup_opcode("STLOC5")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("STLOC6")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(lookup_opcode("STLOC2")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(lookup_opcode("STLOC1")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_dst_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    let collect_start = script.len();
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let collect_exit = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    script.push(lookup_opcode("LDLOC5")?.byte);
    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("PICKITEM")?.byte);
    script.push(lookup_opcode("APPEND")?.byte);
    script.push(lookup_opcode("STLOC5")?.byte);

    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("INC")?.byte);
    script.push(lookup_opcode("STLOC6")?.byte);
    let collect_back = emit_jump_placeholder(script, "JMP_L")?;
    let collect_done = script.len();
    patch_jump(script, collect_exit, collect_done)?;
    patch_jump(script, collect_back, collect_start)?;

    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("STLOC6")?.byte);

    let store_start = script.len();
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let store_exit = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDLOC5")?.byte);
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("PICKITEM")?.byte);
    script.push(lookup_opcode("SETITEM")?.byte);

    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("INC")?.byte);
    script.push(lookup_opcode("STLOC6")?.byte);
    let store_back = emit_jump_placeholder(script, "JMP_L")?;
    let store_done = script.len();
    patch_jump(script, store_exit, store_done)?;
    patch_jump(script, store_back, store_start)?;

    let zero_label = script.len();
    script.push(RET);
    patch_jump(script, zero_branch, zero_label)?;

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);
    patch_jump(script, trap_dst_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    Ok(())
}

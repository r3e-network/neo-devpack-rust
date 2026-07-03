// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

pub(in super::super) fn emit_table_copy_helper(
    script: &mut Vec<u8>,
    dst_slot: usize,
    src_slot: usize,
    mask_u32_offset: Option<usize>,
) -> Result<()> {
    script.push(op::INITSLOT);
    script.push(7);
    script.push(0);

    script.push(op::STLOC2);
    script.push(op::STLOC1);
    script.push(op::STLOC0);
    emit_load_static(script, dst_slot)?;
    script.push(op::STLOC3);
    emit_load_static(script, src_slot)?;
    script.push(op::STLOC4);
    script.push(op::NEWARRAY0);
    script.push(op::STLOC5);
    script.push(op::PUSH0);
    script.push(op::STLOC6);

    script.push(op::LDLOC2);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(op::STLOC2);

    script.push(op::LDLOC0);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(op::STLOC0);

    script.push(op::LDLOC1);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(op::STLOC1);

    script.push(op::LDLOC0);
    script.push(op::LDLOC2);
    script.push(op::ADD);
    script.push(op::LDLOC3);
    script.push(op::SIZE);
    script.push(op::GT);
    let trap_dst_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC1);
    script.push(op::LDLOC2);
    script.push(op::ADD);
    script.push(op::LDLOC4);
    script.push(op::SIZE);
    script.push(op::GT);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC2);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    let collect_start = script.len();
    script.push(op::LDLOC6);
    script.push(op::LDLOC2);
    script.push(op::LT);
    let collect_exit = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    script.push(op::LDLOC5);
    script.push(op::DUP);
    script.push(op::LDLOC4);
    script.push(op::LDLOC1);
    script.push(op::LDLOC6);
    script.push(op::ADD);
    script.push(op::PICKITEM);
    script.push(op::APPEND);
    script.push(op::STLOC5);

    script.push(op::LDLOC6);
    script.push(op::INC);
    script.push(op::STLOC6);
    let collect_back = emit_jump_placeholder(script, "JMP_L")?;
    let collect_done = script.len();
    patch_jump(script, collect_exit, collect_done)?;
    patch_jump(script, collect_back, collect_start)?;

    script.push(op::PUSH0);
    script.push(op::STLOC6);

    let store_start = script.len();
    script.push(op::LDLOC6);
    script.push(op::LDLOC2);
    script.push(op::LT);
    let store_exit = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    script.push(op::LDLOC3);
    script.push(op::LDLOC0);
    script.push(op::LDLOC6);
    script.push(op::ADD);
    script.push(op::LDLOC5);
    script.push(op::LDLOC6);
    script.push(op::PICKITEM);
    script.push(op::SETITEM);

    script.push(op::LDLOC6);
    script.push(op::INC);
    script.push(op::STLOC6);
    let store_back = emit_jump_placeholder(script, "JMP_L")?;
    let store_done = script.len();
    patch_jump(script, store_exit, store_done)?;
    patch_jump(script, store_back, store_start)?;

    let zero_label = script.len();
    script.push(RET);
    patch_jump(script, zero_branch, zero_label)?;

    let trap_label = script.len();
    script.push(op::ABORT);
    patch_jump(script, trap_dst_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    Ok(())
}

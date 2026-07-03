// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::super::*;
use super::super::chunked::emit_chunked_store_byte_at_local;

pub(in crate::translator::runtime) fn emit_memory_fill_helper(
    script: &mut Vec<u8>,
    mask_u32_offset: Option<usize>,
) -> Result<()> {
    script.push(op::INITSLOT);
    script.push(3);
    script.push(0);

    script.push(op::STLOC2);
    script.push(op::STLOC1);
    script.push(op::STLOC0);

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

    script.push(op::LDLOC0);
    script.push(op::LDLOC2);
    script.push(op::ADD);
    script.push(op::LDSFLD1);
    script.push(op::GT);
    let trap_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC1);
    let _ = emit_push_int(script, 0xFF);
    script.push(op::AND);
    script.push(op::STLOC1);

    let loop_start = script.len();

    script.push(op::LDLOC2);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let exit_jump = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDSFLD0);
    script.push(op::LDLOC0);
    script.push(op::LDLOC1);
    script.push(op::SETITEM);

    script.push(op::LDLOC0);
    script.push(op::INC);
    script.push(op::STLOC0);

    script.push(op::LDLOC2);
    script.push(op::DEC);
    script.push(op::STLOC2);

    let loop_back = emit_jump_placeholder(script, "JMP_L")?;

    let exit_label = script.len();
    script.push(RET);

    let trap_label = script.len();
    script.push(op::ABORT);

    patch_jump(script, trap_oob, trap_label)?;
    patch_jump(script, exit_jump, exit_label)?;
    patch_jump(script, loop_back, loop_start)?;
    Ok(())
}

pub(in crate::translator::runtime) fn emit_chunked_memory_fill_helper(
    script: &mut Vec<u8>,
    mask_u32_offset: Option<usize>,
) -> Result<()> {
    script.push(op::INITSLOT);
    script.push(4);
    script.push(0);

    script.push(op::STLOC2); // len
    script.push(op::STLOC1); // value
    script.push(op::STLOC0); // dest

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

    script.push(op::LDLOC0);
    script.push(op::LDLOC2);
    script.push(op::ADD);
    script.push(op::LDSFLD1);
    script.push(op::GT);
    let trap_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC1);
    let _ = emit_push_int(script, 0xFF);
    script.push(op::AND);
    script.push(op::STLOC1);

    let loop_start = script.len();

    script.push(op::LDLOC2);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let exit_jump = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC0);
    script.push(op::STLOC3);
    emit_chunked_store_byte_at_local(script, 3, 1)?;

    script.push(op::LDLOC0);
    script.push(op::INC);
    script.push(op::STLOC0);

    script.push(op::LDLOC2);
    script.push(op::DEC);
    script.push(op::STLOC2);

    let loop_back = emit_jump_placeholder(script, "JMP_L")?;

    let exit_label = script.len();
    script.push(RET);

    let trap_label = script.len();
    script.push(op::ABORT);

    patch_jump(script, trap_oob, trap_label)?;
    patch_jump(script, exit_jump, exit_label)?;
    patch_jump(script, loop_back, loop_start)?;
    Ok(())
}

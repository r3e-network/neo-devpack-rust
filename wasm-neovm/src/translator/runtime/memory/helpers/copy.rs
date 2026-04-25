// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::super::*;
use super::super::chunked::{emit_chunked_load_byte_at_local, emit_chunked_store_byte_at_local};

pub(in crate::translator::runtime) fn emit_memory_copy_helper(
    script: &mut Vec<u8>,
    mask_u32_offset: Option<usize>,
) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(5);
    script.push(0);

    script.push(lookup_opcode("STLOC2")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC2")?.byte);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(lookup_opcode("STLOC2")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(lookup_opcode("STLOC1")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_dest_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_len = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let forward_copy = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("STLOC3")?.byte);

    let back_loop = script.len();
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let back_exit = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("DEC")?.byte);
    script.push(lookup_opcode("STLOC3")?.byte);

    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("PICKITEM")?.byte);
    script.push(lookup_opcode("STLOC4")?.byte);

    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("SETITEM")?.byte);

    let back_jump = emit_jump_placeholder(script, "JMP_L")?;

    let back_exit_label = script.len();
    script.push(RET);

    patch_jump(script, back_exit, back_exit_label)?;
    patch_jump(script, back_jump, back_loop)?;

    let forward_label = script.len();
    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("MEMCPY")?.byte);
    script.push(RET);

    let zero_label = script.len();
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);

    patch_jump(script, trap_dest_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    patch_jump(script, zero_len, zero_label)?;
    patch_jump(script, forward_copy, forward_label)?;
    Ok(())
}

pub(in crate::translator::runtime) fn emit_chunked_memory_copy_helper(
    script: &mut Vec<u8>,
    mask_u32_offset: Option<usize>,
) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(6);
    script.push(0);

    script.push(lookup_opcode("STLOC2")?.byte); // len
    script.push(lookup_opcode("STLOC1")?.byte); // src
    script.push(lookup_opcode("STLOC0")?.byte); // dest

    script.push(lookup_opcode("LDLOC2")?.byte);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(lookup_opcode("STLOC2")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(lookup_opcode("STLOC1")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_dest_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_len = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let forward_copy = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("STLOC3")?.byte);

    let back_loop = script.len();
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let back_exit = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("DEC")?.byte);
    script.push(lookup_opcode("STLOC3")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("STLOC5")?.byte);
    emit_chunked_load_byte_at_local(script, 5)?;
    script.push(lookup_opcode("STLOC4")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("STLOC5")?.byte);
    emit_chunked_store_byte_at_local(script, 5, 4)?;

    let back_jump = emit_jump_placeholder(script, "JMP_L")?;

    let back_exit_label = script.len();
    script.push(RET);

    patch_jump(script, back_exit, back_exit_label)?;
    patch_jump(script, back_jump, back_loop)?;

    let forward_label = script.len();
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("STLOC3")?.byte);

    let forward_loop = script.len();
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let forward_exit = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("STLOC5")?.byte);
    emit_chunked_load_byte_at_local(script, 5)?;
    script.push(lookup_opcode("STLOC4")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("STLOC5")?.byte);
    emit_chunked_store_byte_at_local(script, 5, 4)?;

    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("INC")?.byte);
    script.push(lookup_opcode("STLOC3")?.byte);
    let forward_back = emit_jump_placeholder(script, "JMP_L")?;

    let forward_exit_label = script.len();
    script.push(RET);
    patch_jump(script, forward_exit, forward_exit_label)?;
    patch_jump(script, forward_back, forward_loop)?;

    let zero_label = script.len();
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);

    patch_jump(script, trap_dest_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    patch_jump(script, zero_len, zero_label)?;
    patch_jump(script, forward_copy, forward_label)?;
    Ok(())
}

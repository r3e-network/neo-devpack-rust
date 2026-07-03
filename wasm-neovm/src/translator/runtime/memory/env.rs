// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::*;
use super::chunked::{emit_chunked_load_byte_at_local, emit_chunked_store_byte_at_local};

pub(in crate::translator::runtime) fn emit_env_memcpy_helper(
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

    script.push(op::LDLOC1);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(op::STLOC1);

    script.push(op::LDLOC0);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(op::LDLOC2);
    script.push(op::ADD);
    script.push(op::LDSFLD1);
    script.push(op::GT);
    let trap_dest_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC1);
    script.push(op::LDLOC2);
    script.push(op::ADD);
    script.push(op::LDSFLD1);
    script.push(op::GT);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDSFLD0);
    script.push(op::LDLOC0);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(op::LDSFLD0);
    script.push(op::LDLOC1);
    script.push(op::LDLOC2);
    script.push(op::MEMCPY);
    script.push(op::LDLOC0);
    script.push(RET);

    let trap_label = script.len();
    script.push(op::ABORT);

    patch_jump(script, trap_dest_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    Ok(())
}

pub(in crate::translator::runtime) fn emit_env_memmove_helper(
    script: &mut Vec<u8>,
    mask_u32_offset: Option<usize>,
) -> Result<()> {
    script.push(op::INITSLOT);
    script.push(7);
    script.push(0);

    script.push(op::STLOC2);
    script.push(op::STLOC1);
    script.push(op::STLOC0);

    script.push(op::LDLOC0);
    script.push(op::STLOC5);

    script.push(op::LDLOC2);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(op::STLOC2);

    script.push(op::LDLOC1);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(op::STLOC1);

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
    let trap_dest_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC1);
    script.push(op::LDLOC2);
    script.push(op::ADD);
    script.push(op::LDSFLD1);
    script.push(op::GT);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC2);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let zero_len = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC0);
    script.push(op::LDLOC1);
    script.push(op::LT);
    let forward_copy = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC2);
    script.push(op::STLOC3);

    let back_loop = script.len();
    script.push(op::LDLOC3);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let back_exit = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC3);
    script.push(op::DEC);
    script.push(op::STLOC3);

    script.push(op::LDSFLD0);
    script.push(op::LDLOC1);
    script.push(op::LDLOC3);
    script.push(op::ADD);
    script.push(op::PICKITEM);
    script.push(op::STLOC4);

    script.push(op::LDSFLD0);
    script.push(op::LDLOC0);
    script.push(op::LDLOC3);
    script.push(op::ADD);
    script.push(op::LDLOC4);
    script.push(op::SETITEM);

    let back_jump = emit_jump_placeholder(script, "JMP_L")?;

    let back_exit_label = script.len();
    script.push(op::LDLOC5);
    script.push(RET);

    patch_jump(script, back_exit, back_exit_label)?;
    patch_jump(script, back_jump, back_loop)?;

    let forward_label = script.len();
    script.push(op::LDSFLD0);
    script.push(op::LDLOC0);
    script.push(op::LDSFLD0);
    script.push(op::LDLOC1);
    script.push(op::LDLOC2);
    script.push(op::MEMCPY);
    script.push(op::LDLOC5);
    script.push(RET);

    let zero_label = script.len();
    script.push(op::LDLOC5);
    script.push(RET);

    let trap_label = script.len();
    script.push(op::ABORT);

    patch_jump(script, trap_dest_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    patch_jump(script, zero_len, zero_label)?;
    patch_jump(script, forward_copy, forward_label)?;
    Ok(())
}

pub(in crate::translator::runtime) fn emit_env_memset_helper(
    script: &mut Vec<u8>,
    mask_u32_offset: Option<usize>,
) -> Result<()> {
    script.push(op::INITSLOT);
    script.push(4);
    script.push(0);

    script.push(op::STLOC2);
    script.push(op::STLOC1);
    script.push(op::STLOC0);

    script.push(op::LDLOC0);
    script.push(op::STLOC3);

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
    script.push(op::LDLOC3);
    script.push(RET);

    let trap_label = script.len();
    script.push(op::ABORT);

    patch_jump(script, trap_oob, trap_label)?;
    patch_jump(script, exit_jump, exit_label)?;
    patch_jump(script, loop_back, loop_start)?;
    Ok(())
}

pub(in crate::translator::runtime) fn emit_chunked_env_memcpy_helper(
    script: &mut Vec<u8>,
    mask_u32_offset: Option<usize>,
) -> Result<()> {
    script.push(op::INITSLOT);
    script.push(6);
    script.push(0);

    script.push(op::STLOC2); // len
    script.push(op::STLOC1); // src
    script.push(op::STLOC0); // dest
    script.push(op::LDLOC0);
    script.push(op::STLOC5); // original dest

    script.push(op::LDLOC2);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(op::STLOC2);

    script.push(op::LDLOC1);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(op::STLOC1);

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
    let trap_dest_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC1);
    script.push(op::LDLOC2);
    script.push(op::ADD);
    script.push(op::LDSFLD1);
    script.push(op::GT);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::PUSH0);
    script.push(op::STLOC3);

    let loop_start = script.len();
    script.push(op::LDLOC3);
    script.push(op::LDLOC2);
    script.push(op::EQUAL);
    let loop_exit = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC1);
    script.push(op::LDLOC3);
    script.push(op::ADD);
    script.push(op::STLOC6);
    emit_chunked_load_byte_at_local(script, 6)?;
    script.push(op::STLOC4);

    script.push(op::LDLOC0);
    script.push(op::LDLOC3);
    script.push(op::ADD);
    script.push(op::STLOC6);
    emit_chunked_store_byte_at_local(script, 6, 4)?;

    script.push(op::LDLOC3);
    script.push(op::INC);
    script.push(op::STLOC3);
    let loop_back = emit_jump_placeholder(script, "JMP_L")?;

    let exit_label = script.len();
    script.push(op::LDLOC5);
    script.push(RET);

    let trap_label = script.len();
    script.push(op::ABORT);

    patch_jump(script, trap_dest_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    patch_jump(script, loop_exit, exit_label)?;
    patch_jump(script, loop_back, loop_start)?;
    Ok(())
}

pub(in crate::translator::runtime) fn emit_chunked_env_memmove_helper(
    script: &mut Vec<u8>,
    mask_u32_offset: Option<usize>,
) -> Result<()> {
    script.push(op::INITSLOT);
    script.push(7);
    script.push(0);

    script.push(op::STLOC2); // len
    script.push(op::STLOC1); // src
    script.push(op::STLOC0); // dest
    script.push(op::LDLOC0);
    script.push(op::STLOC6); // original dest

    script.push(op::LDLOC2);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(op::STLOC2);

    script.push(op::LDLOC1);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(op::STLOC1);

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
    let trap_dest_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC1);
    script.push(op::LDLOC2);
    script.push(op::ADD);
    script.push(op::LDSFLD1);
    script.push(op::GT);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC2);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let zero_len = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC0);
    script.push(op::LDLOC1);
    script.push(op::LT);
    let forward_copy = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC2);
    script.push(op::STLOC3);

    let back_loop = script.len();
    script.push(op::LDLOC3);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let back_exit = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC3);
    script.push(op::DEC);
    script.push(op::STLOC3);

    script.push(op::LDLOC1);
    script.push(op::LDLOC3);
    script.push(op::ADD);
    script.push(op::STLOC5);
    emit_chunked_load_byte_at_local(script, 5)?;
    script.push(op::STLOC4);

    script.push(op::LDLOC0);
    script.push(op::LDLOC3);
    script.push(op::ADD);
    script.push(op::STLOC5);
    emit_chunked_store_byte_at_local(script, 5, 4)?;

    let back_jump = emit_jump_placeholder(script, "JMP_L")?;

    let back_exit_label = script.len();
    script.push(op::LDLOC6);
    script.push(RET);

    patch_jump(script, back_exit, back_exit_label)?;
    patch_jump(script, back_jump, back_loop)?;

    let forward_label = script.len();
    script.push(op::PUSH0);
    script.push(op::STLOC3);

    let forward_loop = script.len();
    script.push(op::LDLOC3);
    script.push(op::LDLOC2);
    script.push(op::EQUAL);
    let forward_exit = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC1);
    script.push(op::LDLOC3);
    script.push(op::ADD);
    script.push(op::STLOC5);
    emit_chunked_load_byte_at_local(script, 5)?;
    script.push(op::STLOC4);

    script.push(op::LDLOC0);
    script.push(op::LDLOC3);
    script.push(op::ADD);
    script.push(op::STLOC5);
    emit_chunked_store_byte_at_local(script, 5, 4)?;

    script.push(op::LDLOC3);
    script.push(op::INC);
    script.push(op::STLOC3);
    let forward_back = emit_jump_placeholder(script, "JMP_L")?;

    let forward_exit_label = script.len();
    script.push(op::LDLOC6);
    script.push(RET);
    patch_jump(script, forward_exit, forward_exit_label)?;
    patch_jump(script, forward_back, forward_loop)?;

    let zero_label = script.len();
    script.push(op::LDLOC6);
    script.push(RET);

    let trap_label = script.len();
    script.push(op::ABORT);

    patch_jump(script, trap_dest_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    patch_jump(script, zero_len, zero_label)?;
    patch_jump(script, forward_copy, forward_label)?;
    Ok(())
}

pub(in crate::translator::runtime) fn emit_chunked_env_memset_helper(
    script: &mut Vec<u8>,
    mask_u32_offset: Option<usize>,
) -> Result<()> {
    script.push(op::INITSLOT);
    script.push(4);
    script.push(0);

    script.push(op::STLOC2); // len
    script.push(op::STLOC1); // value
    script.push(op::STLOC0); // dest
    script.push(op::LDLOC0);
    script.push(op::STLOC3); // original dest

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

    emit_chunked_store_byte_at_local(script, 0, 1)?;

    script.push(op::LDLOC0);
    script.push(op::INC);
    script.push(op::STLOC0);

    script.push(op::LDLOC2);
    script.push(op::DEC);
    script.push(op::STLOC2);

    let loop_back = emit_jump_placeholder(script, "JMP_L")?;

    let exit_label = script.len();
    script.push(op::LDLOC3);
    script.push(RET);

    let trap_label = script.len();
    script.push(op::ABORT);

    patch_jump(script, trap_oob, trap_label)?;
    patch_jump(script, exit_jump, exit_label)?;
    patch_jump(script, loop_back, loop_start)?;
    Ok(())
}

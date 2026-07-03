// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

pub(in super::super) fn emit_table_init_from_passive_helper(
    script: &mut Vec<u8>,
    table_slot: usize,
    value_slot: usize,
    drop_slot: usize,
    mask_u32_offset: Option<usize>,
) -> Result<()> {
    script.push(op::INITSLOT);
    script.push(7);
    script.push(0);

    script.push(op::STLOC2);
    script.push(op::STLOC1);
    script.push(op::STLOC0);
    emit_load_static(script, table_slot)?;
    script.push(op::STLOC3);
    emit_load_static(script, value_slot)?;
    script.push(op::STLOC4);
    emit_load_static(script, drop_slot)?;
    script.push(op::STLOC5);
    script.push(op::PUSH0);
    script.push(op::STLOC6);

    script.push(op::LDLOC5);
    script.push(op::PUSH0);
    script.push(op::NOTEQUAL);
    let dropped_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC4);
    script.push(op::SIZE);
    script.push(op::STLOC5);
    let continue_len = emit_jump_placeholder(script, "JMP_L")?;

    let dropped_label = script.len();
    script.push(op::PUSH0);
    script.push(op::STLOC5);

    let len_ready_label = script.len();
    patch_jump(script, dropped_branch, dropped_label)?;
    patch_jump(script, continue_len, len_ready_label)?;

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
    script.push(op::LDLOC5);
    script.push(op::GT);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC2);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    let loop_start = script.len();
    script.push(op::LDLOC6);
    script.push(op::LDLOC2);
    script.push(op::LT);
    let loop_exit = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    script.push(op::LDLOC3);
    script.push(op::LDLOC0);
    script.push(op::LDLOC6);
    script.push(op::ADD);
    script.push(op::LDLOC4);
    script.push(op::LDLOC1);
    script.push(op::LDLOC6);
    script.push(op::ADD);
    script.push(op::PICKITEM);
    script.push(op::SETITEM);

    script.push(op::LDLOC6);
    script.push(op::INC);
    script.push(op::STLOC6);
    let loop_back = emit_jump_placeholder(script, "JMP_L")?;
    let loop_done = script.len();
    patch_jump(script, loop_exit, loop_done)?;
    patch_jump(script, loop_back, loop_start)?;

    let zero_label = script.len();
    script.push(RET);
    patch_jump(script, zero_branch, zero_label)?;

    let trap_label = script.len();
    script.push(op::ABORT);
    patch_jump(script, trap_dst_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    Ok(())
}

pub(in super::super) fn emit_elem_drop_helper(
    script: &mut Vec<u8>,
    drop_slot: usize,
) -> Result<()> {
    let _ = emit_push_int(script, 1);
    emit_store_static(script, drop_slot)?;
    script.push(RET);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elem_drop_helper_is_idempotent() {
        let mut script = Vec::new();
        emit_elem_drop_helper(&mut script, 5).expect("emit helper");

        let notequal = lookup_opcode("NOTEQUAL").unwrap().byte;
        assert!(
            !script.contains(&notequal),
            "elem.drop helper should not branch on prior drop state"
        );

        let abort = lookup_opcode("ABORT").unwrap().byte;
        assert!(
            !script.contains(&abort),
            "elem.drop helper should not trap when invoked repeatedly"
        );
    }

    #[test]
    fn table_init_helper_treats_dropped_segment_as_empty() {
        let mut script = Vec::new();
        emit_table_init_from_passive_helper(&mut script, 0, 1, 2, None).expect("emit helper");

        let push0 = lookup_opcode("PUSH0").unwrap().byte;
        let size = lookup_opcode("SIZE").unwrap().byte;
        let stloc5 = lookup_opcode("STLOC5").unwrap().byte;

        assert!(
            script.windows(2).any(|window| window == [push0, stloc5]),
            "expected dropped branch to record an empty segment length"
        );
        assert!(
            script.windows(2).any(|window| window == [size, stloc5]),
            "expected non-dropped branch to record segment length from SIZE"
        );
    }
}

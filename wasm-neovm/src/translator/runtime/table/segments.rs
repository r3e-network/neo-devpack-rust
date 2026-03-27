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
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(7);
    script.push(0);

    script.push(lookup_opcode("STLOC2")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);
    script.push(lookup_opcode("STLOC0")?.byte);
    emit_load_static(script, table_slot)?;
    script.push(lookup_opcode("STLOC3")?.byte);
    emit_load_static(script, value_slot)?;
    script.push(lookup_opcode("STLOC4")?.byte);
    emit_load_static(script, drop_slot)?;
    script.push(lookup_opcode("STLOC5")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("STLOC6")?.byte);

    script.push(lookup_opcode("LDLOC5")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("NOTEQUAL")?.byte);
    let dropped_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(lookup_opcode("STLOC5")?.byte);
    let continue_len = emit_jump_placeholder(script, "JMP_L")?;

    let dropped_label = script.len();
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("STLOC5")?.byte);

    let len_ready_label = script.len();
    patch_jump(script, dropped_branch, dropped_label)?;
    patch_jump(script, continue_len, len_ready_label)?;

    script.push(lookup_opcode("LDLOC2")?.byte);
    if let Some(off) = mask_u32_offset { emit_call_to(script, off)?; } else { emit_mask_u32(script)?; }
    script.push(lookup_opcode("STLOC2")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    if let Some(off) = mask_u32_offset { emit_call_to(script, off)?; } else { emit_mask_u32(script)?; }
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    if let Some(off) = mask_u32_offset { emit_call_to(script, off)?; } else { emit_mask_u32(script)?; }
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
    script.push(lookup_opcode("LDLOC5")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    let loop_start = script.len();
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("LT")?.byte);
    let loop_exit = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    script.push(lookup_opcode("LDLOC3")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("LDLOC4")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("PICKITEM")?.byte);
    script.push(lookup_opcode("SETITEM")?.byte);

    script.push(lookup_opcode("LDLOC6")?.byte);
    script.push(lookup_opcode("INC")?.byte);
    script.push(lookup_opcode("STLOC6")?.byte);
    let loop_back = emit_jump_placeholder(script, "JMP_L")?;
    let loop_done = script.len();
    patch_jump(script, loop_exit, loop_done)?;
    patch_jump(script, loop_back, loop_start)?;

    let zero_label = script.len();
    script.push(RET);
    patch_jump(script, zero_branch, zero_label)?;

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);
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

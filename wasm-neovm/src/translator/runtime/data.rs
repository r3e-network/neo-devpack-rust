// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::memory::emit_chunked_store_byte_at_local;
use super::*;

pub(super) fn emit_data_init_helper(
    script: &mut Vec<u8>,
    byte_slot: usize,
    drop_slot: usize,
    segment_len: usize,
) -> Result<()> {
    script.push(op::INITSLOT);
    script.push(4);
    script.push(0);

    script.push(op::STLOC2);
    script.push(op::STLOC1);
    script.push(op::STLOC0);
    emit_load_static(script, drop_slot)?;
    script.push(op::PUSH0);
    script.push(op::NOTEQUAL);
    let dropped_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    let _ = emit_push_int(script, segment_len as i128);
    script.push(op::STLOC3);
    let continue_len = emit_jump_placeholder(script, "JMP_L")?;

    let dropped_label = script.len();
    script.push(op::PUSH0);
    script.push(op::STLOC3);

    let len_ready_label = script.len();
    patch_jump(script, dropped_branch, dropped_label)?;
    patch_jump(script, continue_len, len_ready_label)?;

    script.push(op::LDLOC2);
    emit_mask_u32(script)?;
    script.push(op::STLOC2);

    script.push(op::LDLOC1);
    emit_mask_u32(script)?;
    script.push(op::STLOC1);

    script.push(op::LDLOC0);
    emit_mask_u32(script)?;
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
    script.push(op::LDLOC3);
    script.push(op::GT);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC2);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let skip_copy = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDSFLD0);
    script.push(op::LDLOC0);
    emit_load_static(script, byte_slot)?;
    script.push(op::LDLOC1);
    script.push(op::LDLOC2);
    script.push(op::MEMCPY);

    let done_label = script.len();
    script.push(op::RET);

    let trap_label = script.len();
    script.push(op::ABORT);

    patch_jump(script, trap_dest_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    patch_jump(script, skip_copy, done_label)?;
    Ok(())
}

pub(super) fn emit_chunked_data_init_helper(
    script: &mut Vec<u8>,
    byte_slot: usize,
    drop_slot: usize,
    segment_len: usize,
) -> Result<()> {
    script.push(op::INITSLOT);
    script.push(6);
    script.push(0);

    script.push(op::STLOC2); // len
    script.push(op::STLOC1); // src offset
    script.push(op::STLOC0); // dest
    emit_load_static(script, drop_slot)?;
    script.push(op::PUSH0);
    script.push(op::NOTEQUAL);
    let dropped_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    let _ = emit_push_int(script, segment_len as i128);
    script.push(op::STLOC3);
    let continue_len = emit_jump_placeholder(script, "JMP_L")?;

    let dropped_label = script.len();
    script.push(op::PUSH0);
    script.push(op::STLOC3);

    let len_ready_label = script.len();
    patch_jump(script, dropped_branch, dropped_label)?;
    patch_jump(script, continue_len, len_ready_label)?;

    script.push(op::LDLOC2);
    emit_mask_u32(script)?;
    script.push(op::STLOC2);

    script.push(op::LDLOC1);
    emit_mask_u32(script)?;
    script.push(op::STLOC1);

    script.push(op::LDLOC0);
    emit_mask_u32(script)?;
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
    script.push(op::LDLOC3);
    script.push(op::GT);
    let trap_src_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::PUSH0);
    script.push(op::STLOC3); // copied byte count

    let loop_start = script.len();
    script.push(op::LDLOC3);
    script.push(op::LDLOC2);
    script.push(op::EQUAL);
    let loop_exit = emit_jump_placeholder(script, "JMPIF_L")?;

    emit_load_static(script, byte_slot)?;
    script.push(op::LDLOC1);
    script.push(op::LDLOC3);
    script.push(op::ADD);
    script.push(op::PICKITEM);
    script.push(op::STLOC4);

    script.push(op::LDLOC0);
    script.push(op::LDLOC3);
    script.push(op::ADD);
    script.push(op::STLOC5);
    emit_chunked_store_byte_at_local(script, 5, 4)?;

    script.push(op::LDLOC3);
    script.push(op::INC);
    script.push(op::STLOC3);
    let loop_back = emit_jump_placeholder(script, "JMP_L")?;

    let done_label = script.len();
    script.push(RET);

    let trap_label = script.len();
    script.push(op::ABORT);

    patch_jump(script, trap_dest_oob, trap_label)?;
    patch_jump(script, trap_src_oob, trap_label)?;
    patch_jump(script, loop_exit, done_label)?;
    patch_jump(script, loop_back, loop_start)?;
    Ok(())
}

pub(super) fn emit_data_drop_helper(script: &mut Vec<u8>, drop_slot: usize) -> Result<()> {
    let _ = emit_push_int(script, 1);
    emit_store_static(script, drop_slot)?;
    script.push(RET);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_drop_helper_is_idempotent() {
        let mut script = Vec::new();
        emit_data_drop_helper(&mut script, 5).expect("emit helper");

        let notequal = lookup_opcode("NOTEQUAL").unwrap().byte;
        assert!(
            !script.contains(&notequal),
            "data.drop helper should not branch on prior drop state"
        );
    }

    #[test]
    fn data_init_helper_uses_effective_segment_length() {
        let mut script = Vec::new();
        emit_data_init_helper(&mut script, 0, 1, 5).expect("emit helper");

        let initslot = lookup_opcode("INITSLOT").unwrap().byte;
        assert_eq!(script.first().copied(), Some(initslot));
        assert_eq!(script.get(1).copied(), Some(4));

        let stloc3 = lookup_opcode("STLOC3").unwrap().byte;
        let writes = script.iter().filter(|&&byte| byte == stloc3).count();
        assert_eq!(
            writes, 2,
            "expected dropped/non-dropped branches to both store effective segment length"
        );

        let jmpif_l = lookup_opcode("JMPIF_L").unwrap().byte;
        let drop = lookup_opcode("DROP").unwrap().byte;
        assert!(
            !script.windows(2).any(|window| window == [jmpif_l, drop]),
            "unexpected JMPIF_L followed by DROP in data.init helper"
        );
    }
}

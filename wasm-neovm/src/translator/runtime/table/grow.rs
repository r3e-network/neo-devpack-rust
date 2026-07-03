// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

pub(in super::super) fn emit_table_grow_helper(
    script: &mut Vec<u8>,
    table_slot: usize,
    maximum: Option<usize>,
) -> Result<()> {
    script.push(op::INITSLOT);
    script.push(5);
    script.push(0);

    script.push(op::STLOC1);
    script.push(op::STLOC0);
    emit_load_static(script, table_slot)?;
    script.push(op::STLOC2);

    script.push(op::LDLOC1);
    let mask = (1u128 << 32) - 1;
    let _ = emit_push_int(script, mask as i128);
    script.push(op::AND);
    script.push(op::STLOC1);

    script.push(op::LDLOC2);
    script.push(op::SIZE);
    script.push(op::STLOC3);

    script.push(op::LDLOC1);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    let exceed_jump = if let Some(maximum) = maximum {
        script.push(op::LDLOC3);
        script.push(op::LDLOC1);
        script.push(op::ADD);
        let _ = emit_push_int(script, maximum as i128);
        script.push(op::GT);
        let jump = emit_jump_placeholder(script, "JMPIF_L")?;
        Some(jump)
    } else {
        None
    };

    script.push(op::LDLOC1);
    script.push(op::STLOC4);

    let loop_start = script.len();
    script.push(op::LDLOC4);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let loop_exit = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC2);
    script.push(op::DUP);
    script.push(op::LDLOC0);
    script.push(op::APPEND);
    script.push(op::STLOC2);

    script.push(op::LDLOC4);
    script.push(op::DEC);
    script.push(op::STLOC4);
    let loop_back = emit_jump_placeholder(script, "JMP_L")?;

    let exit_label = script.len();
    script.push(op::LDLOC3);
    script.push(RET);
    patch_jump(script, loop_exit, exit_label)?;
    patch_jump(script, loop_back, loop_start)?;

    let zero_label = script.len();
    script.push(op::LDLOC3);
    script.push(RET);
    patch_jump(script, zero_branch, zero_label)?;
    if let Some(exceed_jump) = exceed_jump {
        let fail_label = script.len();
        script.push(op::PUSHM1);
        script.push(RET);
        patch_jump(script, exceed_jump, fail_label)?;
    }
    Ok(())
}

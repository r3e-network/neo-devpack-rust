// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

pub(in super::super) fn emit_table_get_helper(
    script: &mut Vec<u8>,
    table_slot: usize,
    mask_u32_offset: Option<usize>,
) -> Result<()> {
    script.push(op::INITSLOT);
    script.push(2);
    script.push(0);

    script.push(op::STLOC0);
    emit_load_static(script, table_slot)?;
    script.push(op::STLOC1);

    script.push(op::LDLOC0);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(op::STLOC0);

    script.push(op::LDLOC0);
    script.push(op::LDLOC1);
    script.push(op::SIZE);
    script.push(op::SWAP);
    script.push(op::GT);
    let trap_oob = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    script.push(op::LDLOC1);
    script.push(op::LDLOC0);
    script.push(op::PICKITEM);
    script.push(RET);

    let trap_label = script.len();
    script.push(op::ABORT);
    patch_jump(script, trap_oob, trap_label)?;
    Ok(())
}

pub(in super::super) fn emit_table_set_helper(
    script: &mut Vec<u8>,
    table_slot: usize,
    mask_u32_offset: Option<usize>,
) -> Result<()> {
    script.push(op::INITSLOT);
    script.push(3);
    script.push(0);

    script.push(op::STLOC0);
    script.push(op::STLOC1);
    emit_load_static(script, table_slot)?;
    script.push(op::STLOC2);

    script.push(op::LDLOC1);
    if let Some(off) = mask_u32_offset {
        emit_call_to(script, off)?;
    } else {
        emit_mask_u32(script)?;
    }
    script.push(op::STLOC1);

    script.push(op::LDLOC1);
    script.push(op::LDLOC2);
    script.push(op::SIZE);
    script.push(op::SWAP);
    script.push(op::GT);
    let trap_oob = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    script.push(op::LDLOC2);
    script.push(op::LDLOC1);
    script.push(op::LDLOC0);
    script.push(op::SETITEM);
    script.push(RET);

    let trap_label = script.len();
    script.push(op::ABORT);
    patch_jump(script, trap_oob, trap_label)?;
    Ok(())
}

pub(in super::super) fn emit_table_size_helper(
    script: &mut Vec<u8>,
    table_slot: usize,
) -> Result<()> {
    emit_load_static(script, table_slot)?;
    script.push(op::SIZE);
    script.push(RET);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_access_helpers_do_not_drop_after_conditional_jump() {
        let mut script = Vec::new();
        emit_table_get_helper(&mut script, 0, None).expect("emit get helper");
        emit_table_set_helper(&mut script, 0, None).expect("emit set helper");

        let jmpifnot_l = lookup_opcode("JMPIFNOT_L").unwrap().byte;
        let drop = lookup_opcode("DROP").unwrap().byte;
        assert!(
            !script.windows(2).any(|window| window == [jmpifnot_l, drop]),
            "unexpected JMPIFNOT_L followed by DROP in table access helpers"
        );
    }
}

// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

pub(in super::super) fn emit_table_get_helper(
    script: &mut Vec<u8>,
    table_slot: usize,
) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(2);
    script.push(0);

    script.push(lookup_opcode("STLOC0")?.byte);
    emit_load_static(script, table_slot)?;
    script.push(lookup_opcode("STLOC1")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    emit_mask_u32(script)?;
    script.push(lookup_opcode("STLOC0")?.byte);

    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_oob = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("PICKITEM")?.byte);
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);
    patch_jump(script, trap_oob, trap_label)?;
    Ok(())
}

pub(in super::super) fn emit_table_set_helper(
    script: &mut Vec<u8>,
    table_slot: usize,
) -> Result<()> {
    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(3);
    script.push(0);

    script.push(lookup_opcode("STLOC0")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte);
    emit_load_static(script, table_slot)?;
    script.push(lookup_opcode("STLOC2")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    emit_mask_u32(script)?;
    script.push(lookup_opcode("STLOC1")?.byte);

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);
    script.push(lookup_opcode("GT")?.byte);
    let trap_oob = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("SETITEM")?.byte);
    script.push(RET);

    let trap_label = script.len();
    script.push(lookup_opcode("ABORT")?.byte);
    patch_jump(script, trap_oob, trap_label)?;
    Ok(())
}

pub(in super::super) fn emit_table_size_helper(
    script: &mut Vec<u8>,
    table_slot: usize,
) -> Result<()> {
    emit_load_static(script, table_slot)?;
    script.push(lookup_opcode("SIZE")?.byte);
    script.push(RET);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_access_helpers_do_not_drop_after_conditional_jump() {
        let mut script = Vec::new();
        emit_table_get_helper(&mut script, 0).expect("emit get helper");
        emit_table_set_helper(&mut script, 0).expect("emit set helper");

        let jmpifnot_l = lookup_opcode("JMPIFNOT_L").unwrap().byte;
        let drop = lookup_opcode("DROP").unwrap().byte;
        assert!(
            !script.windows(2).any(|window| window == [jmpifnot_l, drop]),
            "unexpected JMPIFNOT_L followed by DROP in table access helpers"
        );
    }
}

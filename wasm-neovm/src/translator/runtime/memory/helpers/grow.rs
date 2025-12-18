use super::super::super::*;

pub(in crate::translator::runtime) fn emit_memory_grow_helper(
    script: &mut Vec<u8>,
    _config: &MemoryConfig,
) -> Result<()> {
    let mask = (1u128 << 32) - 1;
    let _ = emit_push_int(script, mask as i128);
    script.push(lookup_opcode("AND")?.byte);

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("INITSLOT")?.byte);
    script.push(3);
    script.push(0);

    script.push(lookup_opcode("STLOC0")?.byte); // delta pages

    script.push(lookup_opcode("LDSFLD2")?.byte); // current pages
    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("STLOC1")?.byte); // save for return

    script.push(lookup_opcode("LDLOC1")?.byte);
    script.push(lookup_opcode("LDLOC0")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("STLOC2")?.byte); // new pages

    script.push(lookup_opcode("LDLOC2")?.byte); // new pages
    script.push(lookup_opcode("LDSFLD3")?.byte); // maximum
    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("PUSHM1")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let skip_limit = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("GT")?.byte);
    let fail_on_max = emit_jump_placeholder(script, "JMPIF_L")?;
    let after_normal = emit_jump_placeholder(script, "JMP_L")?;

    let skip_limit_label = script.len();
    script.push(lookup_opcode("DROP")?.byte); // drop max when unlimited
    script.push(lookup_opcode("DROP")?.byte); // drop duplicated new_pages to normalise stack

    let after_limit = script.len();
    patch_jump(script, skip_limit, skip_limit_label)?;
    patch_jump(script, after_normal, after_limit)?;

    script.push(lookup_opcode("LDLOC2")?.byte);
    let _ = emit_push_int(script, 16);
    script.push(lookup_opcode("SHL")?.byte); // new byte length
    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("NEWBUFFER")?.byte);
    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LDSFLD0")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("LDSFLD1")?.byte);
    script.push(lookup_opcode("MEMCPY")?.byte);

    script.push(lookup_opcode("STSFLD0")?.byte); // buffer
    script.push(lookup_opcode("STSFLD1")?.byte); // byte length
    script.push(lookup_opcode("LDLOC2")?.byte);
    script.push(lookup_opcode("STSFLD2")?.byte); // page count
    script.push(lookup_opcode("LDLOC1")?.byte); // return old pages
    script.push(RET);

    let zero_label = script.len();
    script.push(lookup_opcode("DROP")?.byte);
    script.push(lookup_opcode("LDSFLD2")?.byte);
    script.push(RET);

    let fail_label = script.len();
    script.push(lookup_opcode("PUSHM1")?.byte);
    script.push(RET);

    patch_jump(script, zero_branch, zero_label)?;
    patch_jump(script, fail_on_max, fail_label)?;
    Ok(())
}

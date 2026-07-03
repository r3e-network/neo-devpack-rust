// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::super::*;
use super::super::chunked::{emit_chunked_new_page, WASM_MEMORY_PAGE_BYTES};

pub(in crate::translator::runtime) fn emit_memory_grow_helper(
    script: &mut Vec<u8>,
    _config: &MemoryConfig,
) -> Result<()> {
    let mask = (1u128 << 32) - 1;
    let _ = emit_push_int(script, mask as i128);
    script.push(op::AND);

    script.push(op::DUP);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::INITSLOT);
    script.push(3);
    script.push(0);

    script.push(op::STLOC0); // delta pages

    script.push(op::LDSFLD2); // current pages
    script.push(op::DUP);
    script.push(op::STLOC1); // save for return

    script.push(op::LDLOC1);
    script.push(op::LDLOC0);
    script.push(op::ADD);
    script.push(op::DUP);
    script.push(op::STLOC2); // new pages

    script.push(op::LDLOC2); // new pages
    script.push(op::LDSFLD3); // maximum
    script.push(op::DUP);
    script.push(op::PUSHM1);
    script.push(op::EQUAL);
    let skip_limit = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::GT);
    let fail_on_max = emit_jump_placeholder(script, "JMPIF_L")?;
    let after_normal = emit_jump_placeholder(script, "JMP_L")?;

    let skip_limit_label = script.len();
    script.push(op::DROP); // drop max when unlimited
    script.push(op::DROP); // drop duplicated new_pages to normalise stack

    let after_limit = script.len();
    patch_jump(script, skip_limit, skip_limit_label)?;
    patch_jump(script, after_normal, after_limit)?;

    script.push(op::LDLOC2);
    let _ = emit_push_int(script, 16);
    script.push(op::SHL); // new byte length
    script.push(op::DUP);
    script.push(op::NEWBUFFER);
    script.push(op::DUP);
    script.push(op::PUSH0);
    script.push(op::LDSFLD0);
    script.push(op::PUSH0);
    script.push(op::LDSFLD1);
    script.push(op::MEMCPY);

    script.push(op::STSFLD0); // buffer
    script.push(op::STSFLD1); // byte length
    script.push(op::LDLOC2);
    script.push(op::STSFLD2); // page count
    script.push(op::LDLOC1); // return old pages
    script.push(RET);

    let zero_label = script.len();
    script.push(op::DROP);
    script.push(op::LDSFLD2);
    script.push(RET);

    let fail_label = script.len();
    script.push(op::PUSHM1);
    script.push(RET);

    patch_jump(script, zero_branch, zero_label)?;
    patch_jump(script, fail_on_max, fail_label)?;
    Ok(())
}

pub(in crate::translator::runtime) fn emit_chunked_memory_grow_helper(
    script: &mut Vec<u8>,
    _config: &MemoryConfig,
) -> Result<()> {
    let mask = (1u128 << 32) - 1;
    let _ = emit_push_int(script, mask as i128);
    script.push(op::AND);

    script.push(op::DUP);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::INITSLOT);
    script.push(4);
    script.push(0);

    script.push(op::STLOC0); // delta pages

    script.push(op::LDSFLD2); // current pages
    script.push(op::DUP);
    script.push(op::STLOC1); // save for return

    script.push(op::LDLOC1);
    script.push(op::LDLOC0);
    script.push(op::ADD);
    script.push(op::DUP);
    script.push(op::STLOC2); // new pages

    script.push(op::LDLOC2); // new pages
    script.push(op::LDSFLD3); // maximum
    script.push(op::DUP);
    script.push(op::PUSHM1);
    script.push(op::EQUAL);
    let skip_limit = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::GT);
    let fail_on_max = emit_jump_placeholder(script, "JMPIF_L")?;
    let after_normal = emit_jump_placeholder(script, "JMP_L")?;

    let skip_limit_label = script.len();
    script.push(op::DROP);
    script.push(op::DROP);

    let after_limit = script.len();
    patch_jump(script, skip_limit, skip_limit_label)?;
    patch_jump(script, after_normal, after_limit)?;

    script.push(op::LDLOC0);
    script.push(op::STLOC3); // remaining pages to append

    let grow_loop = script.len();
    script.push(op::LDLOC3);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let grow_exit = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDSFLD0);
    script.push(op::DUP);
    emit_chunked_new_page(script)?;
    script.push(op::APPEND);
    script.push(op::STSFLD0);

    script.push(op::LDLOC3);
    script.push(op::DEC);
    script.push(op::STLOC3);
    let grow_back = emit_jump_placeholder(script, "JMP_L")?;

    let grow_exit_label = script.len();
    script.push(op::LDLOC2);
    let _ = emit_push_int(script, WASM_MEMORY_PAGE_BYTES);
    script.push(op::MUL);
    script.push(op::STSFLD1);
    script.push(op::LDLOC2);
    script.push(op::STSFLD2);
    script.push(op::LDLOC1);
    script.push(RET);
    patch_jump(script, grow_exit, grow_exit_label)?;
    patch_jump(script, grow_back, grow_loop)?;

    let zero_label = script.len();
    script.push(op::DROP);
    script.push(op::LDSFLD2);
    script.push(RET);

    let fail_label = script.len();
    script.push(op::PUSHM1);
    script.push(RET);

    patch_jump(script, zero_branch, zero_label)?;
    patch_jump(script, fail_on_max, fail_label)?;
    Ok(())
}

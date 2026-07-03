// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::super::*;
use super::super::chunked::{emit_chunked_load_byte_at_local, emit_chunked_store_byte_at_local};

pub(in crate::translator::runtime) fn emit_memory_load_helper(
    script: &mut Vec<u8>,
    bytes: u32,
) -> Result<()> {
    let bytes_i128 = bytes as i128;

    script.push(op::DUP);
    script.push(op::PUSH0);
    script.push(op::LT);
    let trap_negative = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::DUP);
    let _ = emit_push_int(script, bytes_i128);
    script.push(op::ADD);
    script.push(op::LDSFLD1);
    script.push(op::GT);
    let trap_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDSFLD0);
    script.push(op::SWAP);
    let _ = emit_push_int(script, bytes_i128);
    script.push(op::SUBSTR);
    script.push(CONVERT);
    script.push(STACKITEMTYPE_INTEGER);
    script.push(RET);

    let trap_label = script.len();
    script.push(op::ABORT);

    patch_jump(script, trap_negative, trap_label)?;
    patch_jump(script, trap_oob, trap_label)?;

    Ok(())
}

pub(in crate::translator::runtime) fn emit_memory_store_helper(
    script: &mut Vec<u8>,
    bytes: u32,
) -> Result<()> {
    let bytes_i128 = bytes as i128;

    script.push(op::SWAP);

    script.push(op::DUP);
    script.push(op::PUSH0);
    script.push(op::LT);
    let trap_negative = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::DUP);
    let _ = emit_push_int(script, bytes_i128);
    script.push(op::ADD);
    script.push(op::LDSFLD1);
    script.push(op::GT);
    let trap_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::SWAP);
    // Mask value to byte width: compute (1 << (bytes*8)) - 1 inline
    let bit_width = bytes * 8;
    let _ = emit_push_int(script, 1);
    let _ = emit_push_int(script, bit_width as i128);
    script.push(op::SHL);
    script.push(op::DEC);
    script.push(op::AND);
    script.push(op::SWAP);

    for i in 0..bytes {
        script.push(op::OVER);
        let shift = i * 8;
        if shift > 0 {
            let _ = emit_push_int(script, shift as i128);
            script.push(op::SHR);
        }
        let _ = emit_push_int(script, 0xFF);
        script.push(op::AND);
        script.push(op::OVER);
        if i > 0 {
            let _ = emit_push_int(script, i as i128);
            script.push(op::ADD);
        }
        // Stack (bottom->top): [.., masked_value, addr, byte_i, addr+i].
        // NeoVM SETITEM consumes [collection, index, value] (collection deepest,
        // value on top) — same convention SUBSTR uses. We need
        // [memBuf, addr+i, byte_i] before SETITEM.
        script.push(op::SWAP); // -> [.., addr+i, byte_i]
        script.push(op::LDSFLD0); // push memBuf -> [.., addr+i, byte_i, memBuf]
                                                     // Two ROTs cycle the top three [addr+i, byte_i, memBuf] ->
                                                     // [byte_i, memBuf, addr+i] -> [memBuf, addr+i, byte_i], the exact
                                                     // (collection, index, value) order SETITEM requires. (A single ROT left
                                                     // byte_i as the "collection", which faults at runtime — T1.)
        script.push(op::ROT);
        script.push(op::ROT);
        script.push(op::SETITEM);
    }

    script.push(op::DROP);
    script.push(op::DROP);
    script.push(RET);

    let trap_label = script.len();
    script.push(op::ABORT);
    patch_jump(script, trap_negative, trap_label)?;
    patch_jump(script, trap_oob, trap_label)?;

    Ok(())
}

pub(in crate::translator::runtime) fn emit_chunked_memory_load_helper(
    script: &mut Vec<u8>,
    bytes: u32,
) -> Result<()> {
    let bytes_i128 = bytes as i128;

    script.push(op::INITSLOT);
    script.push(5);
    script.push(0);
    script.push(op::STLOC0); // address

    script.push(op::LDLOC0);
    script.push(op::PUSH0);
    script.push(op::LT);
    let trap_negative = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC0);
    let _ = emit_push_int(script, bytes_i128);
    script.push(op::ADD);
    script.push(op::LDSFLD1);
    script.push(op::GT);
    let trap_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::PUSH0);
    script.push(op::STLOC1); // result
    script.push(op::PUSH0);
    script.push(op::STLOC2); // byte index

    let loop_start = script.len();
    script.push(op::LDLOC2);
    let _ = emit_push_int(script, bytes_i128);
    script.push(op::EQUAL);
    let loop_exit = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC0);
    script.push(op::LDLOC2);
    script.push(op::ADD);
    script.push(op::STLOC3);

    emit_chunked_load_byte_at_local(script, 3)?;
    script.push(op::STLOC4);

    script.push(op::LDLOC4);
    script.push(op::LDLOC2);
    let _ = emit_push_int(script, 3);
    script.push(op::SHL);
    script.push(op::SHL);
    script.push(op::LDLOC1);
    script.push(op::OR);
    script.push(op::STLOC1);

    script.push(op::LDLOC2);
    script.push(op::INC);
    script.push(op::STLOC2);
    let loop_back = emit_jump_placeholder(script, "JMP_L")?;

    let exit_label = script.len();
    script.push(op::LDLOC1);
    // The loop above accumulates bytes via `byte << (index*8)` + OR, which
    // produces an UNSIGNED integer. Wasm's full-width loads (`i32.load`/
    // `i64.load`) require a two's-complement sign-extended result, otherwise
    // every downstream signed op (lt_s, shr_s, div_s, ...) sees the wrong
    // magnitude. Recover the sign with the XOR-SUB idiom:
    //   u in [0, 2^W)  ->  (u XOR sign_bit) - sign_bit
    // which yields u for non-negative values and u - 2^W when the top bit is
    // set. Sub-word `_s` loads re-sign-extend at the call site (idempotent)
    // and `_u` loads re-mask via `emit_zero_extend`, so this is safe for all
    // load flavors while costing nothing for the (already-signed) compact
    // `SUBSTR`+`CONVERT` helper used in non-chunked mode.
    let sign_bit = 1i128 << ((bytes * 8) - 1);
    let _ = emit_push_int(script, sign_bit);
    script.push(op::XOR);
    let _ = emit_push_int(script, sign_bit);
    script.push(op::SUB);
    script.push(RET);

    let trap_label = script.len();
    script.push(op::ABORT);

    patch_jump(script, trap_negative, trap_label)?;
    patch_jump(script, trap_oob, trap_label)?;
    patch_jump(script, loop_exit, exit_label)?;
    patch_jump(script, loop_back, loop_start)?;
    Ok(())
}

pub(in crate::translator::runtime) fn emit_chunked_memory_store_helper(
    script: &mut Vec<u8>,
    bytes: u32,
) -> Result<()> {
    let bytes_i128 = bytes as i128;

    script.push(op::INITSLOT);
    script.push(5);
    script.push(0);

    script.push(op::SWAP);
    script.push(op::STLOC0); // address
    script.push(op::STLOC1); // value

    script.push(op::LDLOC0);
    script.push(op::PUSH0);
    script.push(op::LT);
    let trap_negative = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::LDLOC0);
    let _ = emit_push_int(script, bytes_i128);
    script.push(op::ADD);
    script.push(op::LDSFLD1);
    script.push(op::GT);
    let trap_oob = emit_jump_placeholder(script, "JMPIF_L")?;

    let bit_width = bytes * 8;
    let _ = emit_push_int(script, 1);
    let _ = emit_push_int(script, bit_width as i128);
    script.push(op::SHL);
    script.push(op::DEC);
    script.push(op::LDLOC1);
    script.push(op::AND);
    script.push(op::STLOC1);

    for i in 0..bytes {
        script.push(op::LDLOC1);
        let shift = i * 8;
        if shift > 0 {
            let _ = emit_push_int(script, shift as i128);
            script.push(op::SHR);
        }
        let _ = emit_push_int(script, 0xFF);
        script.push(op::AND);
        script.push(op::STLOC2);

        script.push(op::LDLOC0);
        if i > 0 {
            let _ = emit_push_int(script, i as i128);
            script.push(op::ADD);
        }
        script.push(op::STLOC3);
        emit_chunked_store_byte_at_local(script, 3, 2)?;
    }

    script.push(RET);

    let trap_label = script.len();
    script.push(op::ABORT);
    patch_jump(script, trap_negative, trap_label)?;
    patch_jump(script, trap_oob, trap_label)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_no_conditional_jump_drop_pair(script: &[u8], jump_opcode: u8, drop_opcode: u8) {
        // Walk instruction boundaries to check if a conditional jump is followed by DROP.
        // JMPIF_L is 5 bytes (opcode + 4-byte offset), so we skip the operand.
        let mut pc = 0usize;
        while pc < script.len() {
            let op = script[pc];
            let info = crate::opcodes::lookup_by_byte(op);
            let size = match info {
                Some(i) if i.operand_size_prefix == 0 => 1 + i.operand_size as usize,
                _ => 1,
            };
            if op == jump_opcode && pc + size < script.len() && script[pc + size] == drop_opcode {
                panic!(
                    "conditional jump 0x{jump_opcode:02x} at offset {pc} followed by DROP at offset {}",
                    pc + size
                );
            }
            pc += size;
        }
    }

    #[test]
    fn memory_load_helper_does_not_drop_after_jump() {
        let mut script = Vec::new();
        emit_memory_load_helper(&mut script, 4).expect("emit helper");

        let jmpif_l = lookup_opcode("JMPIF_L").unwrap().byte;
        let drop = lookup_opcode("DROP").unwrap().byte;
        assert_no_conditional_jump_drop_pair(&script, jmpif_l, drop);
    }

    #[test]
    fn memory_store_helper_does_not_drop_after_jump() {
        let mut script = Vec::new();
        emit_memory_store_helper(&mut script, 4).expect("emit helper");

        let jmpif_l = lookup_opcode("JMPIF_L").unwrap().byte;
        let drop = lookup_opcode("DROP").unwrap().byte;
        assert_no_conditional_jump_drop_pair(&script, jmpif_l, drop);
    }
}

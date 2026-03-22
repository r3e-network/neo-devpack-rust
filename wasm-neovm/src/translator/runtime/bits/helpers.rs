// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::*;
use super::util::mask_top_bits;

pub(in crate::translator::runtime) fn emit_popcnt_helper(
    script: &mut Vec<u8>,
    bits: u32,
) -> Result<()> {
    mask_top_bits(script, bits)?;
    emit_popcnt_core(script, bits)?;
    script.push(RET);
    Ok(())
}

pub(in crate::translator::runtime) fn emit_ctz_helper(
    script: &mut Vec<u8>,
    bits: u32,
) -> Result<()> {
    mask_top_bits(script, bits)?;

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("NEGATE")?.byte);
    script.push(lookup_opcode("AND")?.byte);
    let _ = emit_push_int(script, 1);
    script.push(lookup_opcode("SUB")?.byte);
    mask_top_bits(script, bits)?;
    emit_popcnt_core(script, bits)?;
    script.push(RET);

    let zero_label = script.len();
    script.push(lookup_opcode("DROP")?.byte);
    let _ = emit_push_int(script, bits as i128);
    script.push(RET);

    patch_jump(script, zero_branch, zero_label)?;
    Ok(())
}

pub(in crate::translator::runtime) fn emit_clz_helper(
    script: &mut Vec<u8>,
    bits: u32,
) -> Result<()> {
    mask_top_bits(script, bits)?;

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("PUSH0")?.byte);
    script.push(lookup_opcode("EQUAL")?.byte);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    let shifts: &[u32] = match bits {
        32 => &[1, 2, 4, 8, 16],
        64 => &[1, 2, 4, 8, 16, 32],
        _ => bail!("unsupported bit-width {} for clz helper", bits),
    };

    for &shift in shifts {
        script.push(lookup_opcode("DUP")?.byte);
        let _ = emit_push_int(script, shift as i128);
        script.push(lookup_opcode("SHR")?.byte);
        script.push(lookup_opcode("OR")?.byte);
    }

    script.push(lookup_opcode("INVERT")?.byte);
    mask_top_bits(script, bits)?;
    emit_popcnt_core(script, bits)?;
    script.push(RET);

    let zero_label = script.len();
    script.push(lookup_opcode("DROP")?.byte);
    let _ = emit_push_int(script, bits as i128);
    script.push(RET);

    patch_jump(script, zero_branch, zero_label)?;
    Ok(())
}

fn emit_popcnt_core(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    let (mask1, mask2, mask4, h01, shift) = match bits {
        32 => (
            0x5555_5555u64 as i128,
            0x3333_3333u64 as i128,
            0x0F0F_0F0Fu64 as i128,
            0x0101_0101u64 as i128,
            24,
        ),
        64 => (
            0x5555_5555_5555_5555u64 as i128,
            0x3333_3333_3333_3333u64 as i128,
            0x0F0F_0F0F_0F0F_0F0Fu64 as i128,
            0x0101_0101_0101_0101u64 as i128,
            56,
        ),
        _ => bail!("unsupported bit-width {} for popcnt helper", bits),
    };

    script.push(lookup_opcode("DUP")?.byte);
    let _ = emit_push_int(script, 1);
    script.push(lookup_opcode("SHR")?.byte);
    let _ = emit_push_int(script, mask1);
    script.push(lookup_opcode("AND")?.byte);
    script.push(lookup_opcode("SUB")?.byte);

    script.push(lookup_opcode("DUP")?.byte);
    let _ = emit_push_int(script, mask2);
    script.push(lookup_opcode("AND")?.byte);
    script.push(lookup_opcode("OVER")?.byte);
    let _ = emit_push_int(script, 2);
    script.push(lookup_opcode("SHR")?.byte);
    let _ = emit_push_int(script, mask2);
    script.push(lookup_opcode("AND")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    script.push(lookup_opcode("SWAP")?.byte);
    script.push(lookup_opcode("DROP")?.byte);

    script.push(lookup_opcode("DUP")?.byte);
    let _ = emit_push_int(script, 4);
    script.push(lookup_opcode("SHR")?.byte);
    script.push(lookup_opcode("ADD")?.byte);
    let _ = emit_push_int(script, mask4);
    script.push(lookup_opcode("AND")?.byte);

    let _ = emit_push_int(script, h01);
    script.push(lookup_opcode("MUL")?.byte);
    let _ = emit_push_int(script, shift as i128);
    script.push(lookup_opcode("SHR")?.byte);
    Ok(())
}

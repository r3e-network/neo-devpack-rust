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

    script.push(op::DUP);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(op::DUP);
    script.push(op::NEGATE);
    script.push(op::AND);
    let _ = emit_push_int(script, 1);
    script.push(op::SUB);
    mask_top_bits(script, bits)?;
    emit_popcnt_core(script, bits)?;
    script.push(RET);

    let zero_label = script.len();
    script.push(op::DROP);
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

    script.push(op::DUP);
    script.push(op::PUSH0);
    script.push(op::EQUAL);
    let zero_branch = emit_jump_placeholder(script, "JMPIF_L")?;

    let shifts: &[u32] = match bits {
        32 => &[1, 2, 4, 8, 16],
        64 => &[1, 2, 4, 8, 16, 32],
        _ => bail!("unsupported bit-width {} for clz helper", bits),
    };

    for &shift in shifts {
        script.push(op::DUP);
        let _ = emit_push_int(script, shift as i128);
        script.push(op::SHR);
        script.push(op::OR);
    }

    script.push(op::INVERT);
    mask_top_bits(script, bits)?;
    emit_popcnt_core(script, bits)?;
    script.push(RET);

    let zero_label = script.len();
    script.push(op::DROP);
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

    script.push(op::DUP);
    let _ = emit_push_int(script, 1);
    script.push(op::SHR);
    let _ = emit_push_int(script, mask1);
    script.push(op::AND);
    script.push(op::SUB);

    script.push(op::DUP);
    let _ = emit_push_int(script, mask2);
    script.push(op::AND);
    script.push(op::OVER);
    let _ = emit_push_int(script, 2);
    script.push(op::SHR);
    let _ = emit_push_int(script, mask2);
    script.push(op::AND);
    script.push(op::ADD);
    script.push(op::SWAP);
    script.push(op::DROP);

    script.push(op::DUP);
    let _ = emit_push_int(script, 4);
    script.push(op::SHR);
    script.push(op::ADD);
    let _ = emit_push_int(script, mask4);
    script.push(op::AND);

    let _ = emit_push_int(script, h01);
    script.push(op::MUL);
    // The SWAR popcount trick `(x * h01) >> shift` extracts the byte-sum from
    // the TOP byte of a `bits`-wide product, relying on the multiply wrapping
    // modulo 2^bits. NeoVM `BigInteger` is arbitrary-precision and does NOT
    // wrap, so the full product carries higher-order byte sums that pollute the
    // shifted result. Mask the product back to `bits` width to emulate the
    // fixed-width wraparound before extracting the top byte.
    mask_top_bits(script, bits)?;
    let _ = emit_push_int(script, shift as i128);
    script.push(op::SHR);
    Ok(())
}

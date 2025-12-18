use super::super::*;

pub(super) fn mask_top_bits(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    if bits >= 128 {
        return Ok(());
    }
    // Compute mask = (1 << bits) - 1 without materialising large immediates.
    emit_pow2(script, bits)?;
    let _ = emit_push_int(script, 1);
    script.push(lookup_opcode("SUB")?.byte);
    script.push(lookup_opcode("AND")?.byte);
    Ok(())
}

pub(super) fn emit_pow2(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    let _ = emit_push_int(script, 1);
    let _ = emit_push_int(script, bits as i128);
    script.push(lookup_opcode("SHL")?.byte);
    Ok(())
}

pub(super) fn truncate_to_bits(value: i128, bits: u32) -> i128 {
    if bits >= 128 {
        value
    } else {
        let mask = (1i128 << bits) - 1;
        value & mask
    }
}

pub(super) fn sign_extend_const(value: i128, bits: u32) -> i128 {
    if bits == 0 || bits >= 128 {
        value
    } else {
        let shift = 128 - bits;
        let masked = truncate_to_bits(value, bits);
        (masked << shift) >> shift
    }
}

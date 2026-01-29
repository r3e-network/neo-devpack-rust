use super::super::*;

/// Branch prediction macros (Round 85)
#[allow(unused_macros)]
macro_rules! likely {
    ($e:expr) => {
        $e
    };
}
#[allow(unused_macros)]
macro_rules! unlikely {
    ($e:expr) => {
        $e
    };
}

/// Mask top bits with const evaluation (Rounds 81, 82)
///
/// Round 81: Inline hot function
/// Round 82: Pre-computed masks for common bit widths
#[inline]
pub(super) fn mask_top_bits(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    // Round 82: Compile-time mask table for common bit widths (0-127)
    // Note: 1i128 << 127 would overflow, so we compute differently
    const MASKS: [i128; 128] = {
        let mut masks = [0i128; 128];
        let mut i = 0u32;
        while i < 127 {
            masks[i as usize] = (1i128 << i) - 1;
            i += 1;
        }
        // Special case for 127: all bits set except sign
        masks[127] = i128::MAX;
        masks
    };

    // Handle edge case
    if bits >= 128 {
        return Ok(());
    }

    // Round 87: Use const-evaluated mask from lookup table
    let mask = MASKS[bits as usize];
    let _ = emit_push_int(script, mask);
    script.push(lookup_opcode("AND")?.byte);
    Ok(())
}

/// Emit power of 2 as constant (Rounds 81, 82)
///
/// Round 82: Pre-computed powers of 2 for common values
#[inline]
pub(super) fn emit_pow2(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    // Round 82: Const table for common power-of-2 values
    const POW2_TABLE: [i128; 65] = {
        let mut table = [0i128; 65];
        let mut i = 0;
        while i <= 64 {
            table[i] = 1i128 << i;
            i += 1;
        }
        table
    };

    // Round 85: Common case is bits <= 64
    if likely!(bits <= 64) {
        let _ = emit_push_int(script, POW2_TABLE[bits as usize]);
    } else {
        // Fallback: compute at runtime
        let _ = emit_push_int(script, 1);
        let _ = emit_push_int(script, bits as i128);
        script.push(lookup_opcode("SHL")?.byte);
    }
    Ok(())
}

/// Truncate value to specified bits (Rounds 81, 82, 87)
///
/// Round 81: Inline always (hot const eval path)
/// Round 82: Compile-time mask table
/// Round 87: Bit manipulation for masking
#[inline(always)]
pub(super) fn truncate_to_bits(value: i128, bits: u32) -> i128 {
    // Round 82: Pre-computed masks
    const MASKS: [u128; 129] = {
        let mut masks = [0u128; 129];
        let mut i = 0;
        while i < 128 {
            masks[i] = (1u128 << i).wrapping_sub(1);
            i += 1;
        }
        masks[128] = u128::MAX;
        masks
    };

    // Round 85: bits >= 128 is unlikely
    if unlikely!(bits >= 128) {
        value
    } else {
        // Round 87: Bitwise AND with pre-computed mask
        let mask = MASKS[bits as usize];
        (value as u128 & mask) as i128
    }
}

/// Sign extend constant value (Rounds 81, 82, 87)
///
/// Round 81: Inline always (hot const eval path)
/// Round 87: Bit manipulation for sign extension
#[inline(always)]
pub(super) fn sign_extend_const(value: i128, bits: u32) -> i128 {
    // Round 85: Edge cases are unlikely
    if unlikely!(bits == 0 || bits >= 128) {
        return value;
    }

    // Round 87: Use arithmetic shift for sign extension
    let shift = 128 - bits;
    let masked = truncate_to_bits(value, bits) as u128;
    // Sign extend by shifting left then right (arithmetic)
    ((masked << shift) as i128) >> shift
}

// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

/// Comparison operations (Round 82 - Const evaluation optimization)
#[derive(Clone, Copy)]
pub(in super::super) enum CompareOp {
    Lt,
    Le,
    Gt,
    Ge,
}

impl CompareOp {
    /// Get the opcode name for this comparison (Round 81 - inline)
    #[inline(always)]
    fn opcode_name(self) -> &'static str {
        match self {
            CompareOp::Lt => "LT",
            CompareOp::Le => "LE",
            CompareOp::Gt => "GT",
            CompareOp::Ge => "GE",
        }
    }

    /// Evaluate signed comparison with compile-time constant folding (Round 82)
    #[inline]
    fn evaluate_signed(self, lhs: i128, rhs: i128, bits: u32) -> bool {
        // Simple cast-based sign extension (more reliable than bit manipulation)
        match bits {
            32 => {
                let lhs = (lhs as i32) as i128;
                let rhs = (rhs as i32) as i128;
                self.evaluate_order(lhs, rhs)
            }
            64 => {
                let lhs = (lhs as i64) as i128;
                let rhs = (rhs as i64) as i128;
                self.evaluate_order(lhs, rhs)
            }
            other => unreachable!("unsupported signed comparison width {}", other),
        }
    }

    /// Evaluate unsigned comparison (Round 81 - inline)
    #[inline(always)]
    fn evaluate_unsigned(self, lhs: u128, rhs: u128) -> bool {
        self.evaluate_order(lhs, rhs)
    }

    /// Generic order evaluation (Round 81 - inline hot path)
    #[inline(always)]
    fn evaluate_order<T: PartialOrd>(self, lhs: T, rhs: T) -> bool {
        match self {
            CompareOp::Lt => lhs < rhs,
            CompareOp::Le => lhs <= rhs,
            CompareOp::Gt => lhs > rhs,
            CompareOp::Ge => lhs >= rhs,
        }
    }
}

pub(in super::super) fn emit_signed_compare(
    script: &mut Vec<u8>,
    lhs: StackValue,
    rhs: StackValue,
    bits: u32,
    kind: CompareOp,
) -> Result<StackValue> {
    emit_binary_op(script, kind.opcode_name(), lhs, rhs, |a, b| {
        let cmp = kind.evaluate_signed(a, b, bits);
        Some(if cmp { 1 } else { 0 })
    })
}

pub(in super::super) fn emit_unsigned_compare(
    script: &mut Vec<u8>,
    lhs: StackValue,
    rhs: StackValue,
    bits: u32,
    kind: CompareOp,
) -> Result<StackValue> {
    super::divrem::mask_unsigned_operands(script, bits)?;
    let mask = (1u128 << bits) - 1;
    emit_binary_op(script, kind.opcode_name(), lhs, rhs, |a, b| {
        let lhs = (a as u128) & mask;
        let rhs = (b as u128) & mask;
        let cmp = kind.evaluate_unsigned(lhs, rhs);
        Some(if cmp { 1 } else { 0 })
    })
}

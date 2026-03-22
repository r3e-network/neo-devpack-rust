// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

/// Emit binary operation with constant folding (Round 81, 82 optimizations)
///
/// Round 81: `#[inline]` for hot function
/// Round 82: Compile-time constant evaluation
#[inline]
pub(crate) fn emit_binary_op(
    script: &mut Vec<u8>,
    opcode_name: &str,
    lhs: StackValue,
    rhs: StackValue,
    combine: impl FnOnce(i128, i128) -> Option<i128>,
) -> Result<StackValue> {
    let opcode = lookup_opcode(opcode_name)?;
    script.push(opcode.byte);

    // Round 82: Constant folding when both values known at compile time
    let const_value = match (lhs.const_value, rhs.const_value) {
        (Some(a), Some(b)) => combine(a, b),
        _ => None,
    };

    Ok(StackValue {
        const_value,
        bytecode_start: None,
    })
}

/// Emit EQZ (equal to zero) with constant folding (Round 81 - inline)
#[inline]
pub(in super::super) fn emit_eqz(script: &mut Vec<u8>, value: StackValue) -> Result<StackValue> {
    // Round 82: Const evaluation for EQZ
    if let (Some(constant), Some(_start)) = (value.const_value, value.bytecode_start) {
        script.push(lookup_opcode("DROP")?.byte);
        let result = if constant == 0 { 1 } else { 0 };
        return Ok(emit_push_int(script, result));
    }

    let push0 = lookup_opcode("PUSH0")?;
    script.push(push0.byte);
    let equal = lookup_opcode("EQUAL")?;
    script.push(equal.byte);
    Ok(StackValue::unknown())
}

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
        pending_sign_extend: None,
    })
}

/// Emit EQZ (equal to zero) with constant folding (Round 81 - inline)
#[inline]
pub(in super::super) fn emit_eqz(script: &mut Vec<u8>, value: StackValue) -> Result<StackValue> {
    // Round 82: Const evaluation for EQZ
    if let (Some(constant), Some(_start)) = (value.const_value, value.bytecode_start) {
        script.push(op::DROP);
        let result = if constant == 0 { 1 } else { 0 };
        return Ok(emit_push_int(script, result));
    }

    // Lower `eqz` to a single `NOT`, NOT `PUSH0; EQUAL`.
    //
    // NeoVM's `EQUAL` (0x97) is type-strict: it compares stack-item types, so
    // `Boolean(false) EQUAL Integer(0)` returns FALSE. Wasm `eqz` is frequently
    // chained by LLVM — e.g. `if x == 0 { .. } else { .. }` lowers to
    // `(x == 0) == 0` — which feeds the *Boolean* result of the first compare
    // into the second `eqz`. With `PUSH0; EQUAL` that Boolean never equalled
    // Integer 0, so the guard silently always fell through (this corrupted
    // div_u/rem_u and num-bigint's negative `>>`, found by the differential
    // fuzzer). `NOT` (0xAA) coerces its operand to a boolean first
    // (0/false -> false, otherwise true) and negates it, which is exactly
    // `== 0` for both Integer and Boolean operands.
    let not = lookup_opcode("NOT")?;
    script.push(not.byte);
    Ok(StackValue::unknown())
}

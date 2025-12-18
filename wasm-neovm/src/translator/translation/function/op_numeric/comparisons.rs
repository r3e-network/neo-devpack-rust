use super::*;

pub(super) fn try_handle(
    op: &Operator,
    script: &mut Vec<u8>,
    _runtime: &mut RuntimeHelpers,
    value_stack: &mut Vec<StackValue>,
) -> Result<bool> {
    match op {
        Operator::I32Eq => {
            let rhs = super::pop_value(value_stack, "i32.eq rhs")?;
            let lhs = super::pop_value(value_stack, "i32.eq lhs")?;
            let result = emit_binary_op(script, "EQUAL", lhs, rhs, |a, b| {
                Some(if a == b { 1 } else { 0 })
            })?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32Ne => {
            let rhs = super::pop_value(value_stack, "i32.ne rhs")?;
            let lhs = super::pop_value(value_stack, "i32.ne lhs")?;
            let result = emit_binary_op(script, "NOTEQUAL", lhs, rhs, |a, b| {
                Some(if a != b { 1 } else { 0 })
            })?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32LtS => {
            let rhs = super::pop_value(value_stack, "i32.lt_s rhs")?;
            let lhs = super::pop_value(value_stack, "i32.lt_s lhs")?;
            let result = emit_signed_compare(script, lhs, rhs, 32, CompareOp::Lt)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32LtU => {
            let rhs = super::pop_value(value_stack, "i32.lt_u rhs")?;
            let lhs = super::pop_value(value_stack, "i32.lt_u lhs")?;
            let result = emit_unsigned_compare(script, lhs, rhs, 32, CompareOp::Lt)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32LeS => {
            let rhs = super::pop_value(value_stack, "i32.le_s rhs")?;
            let lhs = super::pop_value(value_stack, "i32.le_s lhs")?;
            let result = emit_signed_compare(script, lhs, rhs, 32, CompareOp::Le)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32LeU => {
            let rhs = super::pop_value(value_stack, "i32.le_u rhs")?;
            let lhs = super::pop_value(value_stack, "i32.le_u lhs")?;
            let result = emit_unsigned_compare(script, lhs, rhs, 32, CompareOp::Le)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32GtS => {
            let rhs = super::pop_value(value_stack, "i32.gt_s rhs")?;
            let lhs = super::pop_value(value_stack, "i32.gt_s lhs")?;
            let result = emit_signed_compare(script, lhs, rhs, 32, CompareOp::Gt)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32GtU => {
            let rhs = super::pop_value(value_stack, "i32.gt_u rhs")?;
            let lhs = super::pop_value(value_stack, "i32.gt_u lhs")?;
            let result = emit_unsigned_compare(script, lhs, rhs, 32, CompareOp::Gt)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32GeS => {
            let rhs = super::pop_value(value_stack, "i32.ge_s rhs")?;
            let lhs = super::pop_value(value_stack, "i32.ge_s lhs")?;
            let result = emit_signed_compare(script, lhs, rhs, 32, CompareOp::Ge)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32GeU => {
            let rhs = super::pop_value(value_stack, "i32.ge_u rhs")?;
            let lhs = super::pop_value(value_stack, "i32.ge_u lhs")?;
            let result = emit_unsigned_compare(script, lhs, rhs, 32, CompareOp::Ge)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Eq => {
            let rhs = super::pop_value(value_stack, "i64.eq rhs")?;
            let lhs = super::pop_value(value_stack, "i64.eq lhs")?;
            let result = emit_binary_op(script, "EQUAL", lhs, rhs, |a, b| {
                Some(if a == b { 1 } else { 0 })
            })?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Ne => {
            let rhs = super::pop_value(value_stack, "i64.ne rhs")?;
            let lhs = super::pop_value(value_stack, "i64.ne lhs")?;
            let result = emit_binary_op(script, "NOTEQUAL", lhs, rhs, |a, b| {
                Some(if a != b { 1 } else { 0 })
            })?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64LtS => {
            let rhs = super::pop_value(value_stack, "i64.lt_s rhs")?;
            let lhs = super::pop_value(value_stack, "i64.lt_s lhs")?;
            let result = emit_signed_compare(script, lhs, rhs, 64, CompareOp::Lt)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64LtU => {
            let rhs = super::pop_value(value_stack, "i64.lt_u rhs")?;
            let lhs = super::pop_value(value_stack, "i64.lt_u lhs")?;
            let result = emit_unsigned_compare(script, lhs, rhs, 64, CompareOp::Lt)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64LeS => {
            let rhs = super::pop_value(value_stack, "i64.le_s rhs")?;
            let lhs = super::pop_value(value_stack, "i64.le_s lhs")?;
            let result = emit_signed_compare(script, lhs, rhs, 64, CompareOp::Le)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64LeU => {
            let rhs = super::pop_value(value_stack, "i64.le_u rhs")?;
            let lhs = super::pop_value(value_stack, "i64.le_u lhs")?;
            let result = emit_unsigned_compare(script, lhs, rhs, 64, CompareOp::Le)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64GtS => {
            let rhs = super::pop_value(value_stack, "i64.gt_s rhs")?;
            let lhs = super::pop_value(value_stack, "i64.gt_s lhs")?;
            let result = emit_signed_compare(script, lhs, rhs, 64, CompareOp::Gt)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64GtU => {
            let rhs = super::pop_value(value_stack, "i64.gt_u rhs")?;
            let lhs = super::pop_value(value_stack, "i64.gt_u lhs")?;
            let result = emit_unsigned_compare(script, lhs, rhs, 64, CompareOp::Gt)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64GeS => {
            let rhs = super::pop_value(value_stack, "i64.ge_s rhs")?;
            let lhs = super::pop_value(value_stack, "i64.ge_s lhs")?;
            let result = emit_signed_compare(script, lhs, rhs, 64, CompareOp::Ge)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64GeU => {
            let rhs = super::pop_value(value_stack, "i64.ge_u rhs")?;
            let lhs = super::pop_value(value_stack, "i64.ge_u lhs")?;
            let result = emit_unsigned_compare(script, lhs, rhs, 64, CompareOp::Ge)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32Eqz => {
            let value = super::pop_value(value_stack, "i32.eqz operand")?;
            let result = emit_eqz(script, value)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Eqz => {
            let value = super::pop_value(value_stack, "i64.eqz operand")?;
            let result = emit_eqz(script, value)?;
            value_stack.push(result);
            Ok(true)
        }
        _ => Ok(false),
    }
}

use super::*;

pub(super) fn try_handle(
    op: &Operator,
    script: &mut Vec<u8>,
    _runtime: &mut RuntimeHelpers,
    value_stack: &mut Vec<StackValue>,
) -> Result<bool> {
    match op {
        Operator::I32And => {
            let rhs = super::pop_value(value_stack, "i32.and rhs")?;
            let lhs = super::pop_value(value_stack, "i32.and lhs")?;
            let result = emit_binary_op(script, "AND", lhs, rhs, |a, b| {
                Some(((a as i32) & (b as i32)) as i128)
            })?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32Or => {
            let rhs = super::pop_value(value_stack, "i32.or rhs")?;
            let lhs = super::pop_value(value_stack, "i32.or lhs")?;
            let result = emit_binary_op(script, "OR", lhs, rhs, |a, b| {
                Some(((a as i32) | (b as i32)) as i128)
            })?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32Xor => {
            let rhs = super::pop_value(value_stack, "i32.xor rhs")?;
            let lhs = super::pop_value(value_stack, "i32.xor lhs")?;
            let result = emit_binary_op(script, "XOR", lhs, rhs, |a, b| {
                Some(((a as i32) ^ (b as i32)) as i128)
            })?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64And => {
            let rhs = super::pop_value(value_stack, "i64.and rhs")?;
            let lhs = super::pop_value(value_stack, "i64.and lhs")?;
            let result = emit_binary_op(script, "AND", lhs, rhs, |a, b| {
                Some(((a as i64) & (b as i64)) as i128)
            })?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Or => {
            let rhs = super::pop_value(value_stack, "i64.or rhs")?;
            let lhs = super::pop_value(value_stack, "i64.or lhs")?;
            let result = emit_binary_op(script, "OR", lhs, rhs, |a, b| {
                Some(((a as i64) | (b as i64)) as i128)
            })?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Xor => {
            let rhs = super::pop_value(value_stack, "i64.xor rhs")?;
            let lhs = super::pop_value(value_stack, "i64.xor lhs")?;
            let result = emit_binary_op(script, "XOR", lhs, rhs, |a, b| {
                Some(((a as i64) ^ (b as i64)) as i128)
            })?;
            value_stack.push(result);
            Ok(true)
        }
        _ => Ok(false),
    }
}

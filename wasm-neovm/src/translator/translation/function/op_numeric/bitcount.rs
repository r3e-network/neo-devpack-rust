use super::*;

pub(super) fn try_handle(
    op: &Operator,
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    value_stack: &mut Vec<StackValue>,
) -> Result<bool> {
    match op {
        Operator::I32Clz => {
            let value = super::pop_value(value_stack, "i32.clz operand")?;
            let result = emit_bit_count(script, runtime, value, BitHelperKind::Clz(32))?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32Ctz => {
            let value = super::pop_value(value_stack, "i32.ctz operand")?;
            let result = emit_bit_count(script, runtime, value, BitHelperKind::Ctz(32))?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I32Popcnt => {
            let value = super::pop_value(value_stack, "i32.popcnt operand")?;
            let result = emit_bit_count(script, runtime, value, BitHelperKind::Popcnt(32))?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Clz => {
            let value = super::pop_value(value_stack, "i64.clz operand")?;
            let result = emit_bit_count(script, runtime, value, BitHelperKind::Clz(64))?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Ctz => {
            let value = super::pop_value(value_stack, "i64.ctz operand")?;
            let result = emit_bit_count(script, runtime, value, BitHelperKind::Ctz(64))?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::I64Popcnt => {
            let value = super::pop_value(value_stack, "i64.popcnt operand")?;
            let result = emit_bit_count(script, runtime, value, BitHelperKind::Popcnt(64))?;
            value_stack.push(result);
            Ok(true)
        }
        _ => Ok(false),
    }
}

use super::*;

pub(super) fn try_handle(
    op: &Operator,
    script: &mut Vec<u8>,
    _runtime: &mut RuntimeHelpers,
    value_stack: &mut Vec<StackValue>,
) -> Result<bool> {
    match op {
        Operator::Nop => Ok(true),
        Operator::I32Const { value } => {
            let entry = emit_push_int(script, (*value) as i128);
            value_stack.push(entry);
            Ok(true)
        }
        Operator::I64Const { value } => {
            let entry = emit_push_int(script, (*value) as i128);
            value_stack.push(entry);
            Ok(true)
        }
        _ => Ok(false),
    }
}

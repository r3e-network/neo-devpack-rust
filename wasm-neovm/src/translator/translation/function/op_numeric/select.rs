use super::*;

pub(super) fn try_handle(
    op: &Operator,
    script: &mut Vec<u8>,
    _runtime: &mut RuntimeHelpers,
    value_stack: &mut Vec<StackValue>,
) -> Result<bool> {
    match op {
        Operator::Select => {
            let condition = super::pop_value(value_stack, "select condition")?;
            let false_value = super::pop_value(value_stack, "select false value")?;
            let true_value = super::pop_value(value_stack, "select true value")?;
            let result = emit_select(script, true_value, false_value, condition)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::TypedSelect { ty } => {
            ensure_select_type_supported(std::slice::from_ref(ty))?;
            let condition = super::pop_value(value_stack, "typed select condition")?;
            let false_value = super::pop_value(value_stack, "typed select false value")?;
            let true_value = super::pop_value(value_stack, "typed select true value")?;
            let result = emit_select(script, true_value, false_value, condition)?;
            value_stack.push(result);
            Ok(true)
        }
        Operator::TypedSelectMulti { tys } => {
            ensure_select_type_supported(tys)?;
            let condition = super::pop_value(value_stack, "typed select condition")?;
            let false_value = super::pop_value(value_stack, "typed select false value")?;
            let true_value = super::pop_value(value_stack, "typed select true value")?;
            let result = emit_select(script, true_value, false_value, condition)?;
            value_stack.push(result);
            Ok(true)
        }
        _ => Ok(false),
    }
}

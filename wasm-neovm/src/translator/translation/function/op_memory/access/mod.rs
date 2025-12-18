use super::*;

mod load;
mod store;

pub(super) fn try_handle(
    op: &Operator,
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    value_stack: &mut Vec<StackValue>,
) -> Result<bool> {
    if load::try_handle(op, script, runtime, value_stack)? {
        return Ok(true);
    }
    if store::try_handle(op, script, runtime, value_stack)? {
        return Ok(true);
    }
    Ok(false)
}

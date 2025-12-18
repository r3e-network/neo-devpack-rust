use super::*;

mod arithmetic;
mod bitcount;
mod bitwise;
mod comparisons;
mod consts;
mod conversions;
mod divrem;
mod select;
mod shifts;

pub(super) fn try_handle(
    op: &Operator,
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    value_stack: &mut Vec<StackValue>,
) -> Result<bool> {
    if consts::try_handle(op, script, runtime, value_stack)? {
        return Ok(true);
    }
    if bitcount::try_handle(op, script, runtime, value_stack)? {
        return Ok(true);
    }
    if arithmetic::try_handle(op, script, runtime, value_stack)? {
        return Ok(true);
    }
    if bitwise::try_handle(op, script, runtime, value_stack)? {
        return Ok(true);
    }
    if shifts::try_handle(op, script, runtime, value_stack)? {
        return Ok(true);
    }
    if divrem::try_handle(op, script, runtime, value_stack)? {
        return Ok(true);
    }
    if conversions::try_handle(op, script, runtime, value_stack)? {
        return Ok(true);
    }
    if comparisons::try_handle(op, script, runtime, value_stack)? {
        return Ok(true);
    }
    if select::try_handle(op, script, runtime, value_stack)? {
        return Ok(true);
    }

    Ok(false)
}

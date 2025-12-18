use super::*;

pub(crate) fn emit_binary_op(
    script: &mut Vec<u8>,
    opcode_name: &str,
    lhs: StackValue,
    rhs: StackValue,
    combine: impl FnOnce(i128, i128) -> Option<i128>,
) -> Result<StackValue> {
    let opcode = lookup_opcode(opcode_name)?;
    script.push(opcode.byte);
    let const_value = match (lhs.const_value, rhs.const_value) {
        (Some(a), Some(b)) => combine(a, b),
        _ => None,
    };
    Ok(StackValue {
        const_value,
        bytecode_start: None,
    })
}

pub(in super::super) fn emit_eqz(script: &mut Vec<u8>, value: StackValue) -> Result<StackValue> {
    if let (Some(constant), Some(start)) = (value.const_value, value.bytecode_start) {
        script.truncate(start);
        let result = if constant == 0 { 1 } else { 0 };
        return Ok(emit_push_int(script, result));
    }

    let push0 = lookup_opcode("PUSH0")?;
    script.push(push0.byte);
    let equal = lookup_opcode("EQUAL")?;
    script.push(equal.byte);
    Ok(StackValue {
        const_value: None,
        bytecode_start: None,
    })
}

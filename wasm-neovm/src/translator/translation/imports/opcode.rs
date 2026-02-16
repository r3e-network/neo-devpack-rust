use super::*;

pub(super) fn emit_opcode_call(
    import: &FunctionImport,
    func_type: &FuncType,
    params: &[StackValue],
    script: &mut Vec<u8>,
) -> Result<()> {
    if !func_type.results().is_empty() {
        bail!(
            "imported opcode '{}' must have signature (param ..., result void)",
            import.name
        );
    }

    let expected_params = func_type.params().len();
    if expected_params != params.len() {
        bail!(
            "imported opcode '{}' expects {} parameter(s) but {} were provided",
            import.name,
            expected_params,
            params.len()
        );
    }

    if import.name.eq_ignore_ascii_case("raw") {
        ensure_param_count(import, params, 1)?;
        let param = params
            .last()
            .ok_or_else(|| anyhow!("import '{}' missing operand", import.name))?;
        let value = truncate_literal(param, script, 1)? as u8;
        script.push(value);
        return Ok(());
    }

    if import.name.eq_ignore_ascii_case("raw4") {
        ensure_param_count(import, params, 1)?;
        let param = params
            .last()
            .ok_or_else(|| anyhow!("import '{}' missing operand", import.name))?;
        let value = truncate_literal(param, script, 4)? as i64;
        script.extend_from_slice(&(value as u32).to_le_bytes());
        return Ok(());
    }

    let info = opcodes::lookup(&import.name)
        .ok_or_else(|| anyhow!("unknown NeoVM opcode '{}'", import.name))?;

    if info.operand_size_prefix != 0 {
        bail!(
            "opcode '{}' has a variable-size operand; emit it manually via opcode.raw/raw4",
            import.name
        );
    }

    if info.operand_size == 0 {
        if !params.is_empty() {
            bail!("opcode '{}' does not take immediate operands", import.name);
        }
        script.push(info.byte);
        return Ok(());
    }

    ensure_param_count(import, params, 1)?;
    let param = params
        .last()
        .ok_or_else(|| anyhow!("import '{}' missing operand", import.name))?;
    let immediate = truncate_literal(param, script, info.operand_size as usize)?;

    script.push(info.byte);
    match info.operand_size {
        1 => script.push(immediate as u8),
        2 => script.extend_from_slice(&(immediate as i16).to_le_bytes()),
        4 => script.extend_from_slice(&(immediate as i32).to_le_bytes()),
        8 => script.extend_from_slice(&(immediate as i64).to_le_bytes()),
        16 => script.extend_from_slice(&immediate.to_le_bytes()),
        32 => script.extend_from_slice(&sign_extend_i128_to_32(immediate)),
        other => {
            bail!(
                "unsupported operand size {} for opcode '{}'; use opcode.raw/raw4",
                other,
                import.name
            );
        }
    }

    Ok(())
}

fn ensure_param_count(
    import: &FunctionImport,
    params: &[StackValue],
    expected: usize,
) -> Result<()> {
    if params.len() != expected {
        bail!(
            "import '{}' expects {} parameter(s) but {} were provided",
            import.name,
            expected,
            params.len()
        );
    }
    Ok(())
}

fn literal_instruction_len(script: &[u8], start: usize) -> Result<usize> {
    if start >= script.len() {
        bail!(
            "invalid literal start {} for script of length {}",
            start,
            script.len()
        );
    }

    let opcode = script[start];
    let len = if opcode == PUSHM1
        || opcode == PUSH0
        || (PUSH_BASE + 1..=PUSH_BASE + 16).contains(&opcode)
    {
        1usize
    } else if opcode == PUSHINT8 {
        1usize + 1
    } else if opcode == PUSHINT16 {
        1usize + 2
    } else if opcode == PUSHINT32 {
        1usize + 4
    } else if opcode == PUSHINT64 {
        1usize + 8
    } else if opcode == PUSHINT128 {
        1usize + 16
    } else {
        bail!(
            "unable to determine literal length for opcode 0x{:02X}",
            opcode
        );
    };

    if start + len > script.len() {
        bail!("literal extends beyond script bounds");
    }

    Ok(len)
}

fn sign_extend_i128_to_32(value: i128) -> [u8; 32] {
    let mut bytes = [if value < 0 { 0xFF } else { 0x00 }; 32];
    bytes[..16].copy_from_slice(&value.to_le_bytes());
    bytes
}

fn truncate_literal(param: &StackValue, script: &mut Vec<u8>, max_bytes: usize) -> Result<i128> {
    let value = param
        .const_value
        .ok_or_else(|| anyhow!("import argument must be a compile-time constant"))?;
    // Treat the literal as signed; validate it fits within the requested bytes.
    if max_bytes == 0 || max_bytes > 32 {
        bail!("unsupported immediate width {} bytes", max_bytes);
    }

    if max_bytes < 16 {
        let bits = max_bytes * 8;
        let min = -(1i128 << (bits - 1));
        let max_signed = (1i128 << (bits - 1)) - 1;
        let max_unsigned = (1i128 << bits) - 1;
        if !(min <= value && value <= max_signed || 0 <= value && value <= max_unsigned) {
            bail!(
                "literal value {} does not fit in {} byte(s) for opcode immediate",
                value,
                max_bytes
            );
        }
    }

    let Some(start) = param.bytecode_start else {
        bail!("import argument cannot be materialised as an immediate; ensure it is a literal");
    };

    if start >= script.len() {
        bail!("internal error: literal start beyond current script length");
    }

    let literal_len = literal_instruction_len(script, start)?;
    let literal_end = start + literal_len;
    if literal_end == script.len() {
        script.truncate(start);
    } else {
        let drop = lookup_opcode("DROP")?;
        script.push(drop.byte);
    }

    Ok(value)
}

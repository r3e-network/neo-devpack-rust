use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
struct OpcodeEntry {
    name: String,
    byte: u8,
    operand_size: u8,
    operand_size_prefix: u8,
}

fn parse_neovm_opcodes() -> Result<Vec<OpcodeEntry>, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .expect("wasm-neovm crate should live one level below repo root");
    let opcodes_path = repo_root.join("neo/src/Neo.VM/OpCode.cs");
    let contents = fs::read_to_string(&opcodes_path)?;

    let mut entries = Vec::new();
    let mut current_size: u8 = 0;
    let mut current_prefix: u8 = 0;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[OperandSize(") {
            current_size = 0;
            current_prefix = 0;
            let body = trimmed
                .trim_start_matches("[OperandSize(")
                .trim_end_matches(")]");
            for part in body.split(',') {
                let part = part.trim();
                if let Some(value) = part.strip_prefix("SizePrefix") {
                    if let Some(value) = value.split('=').nth(1) {
                        current_prefix = value.trim().parse::<u16>()? as u8;
                    }
                } else if let Some(value) = part.strip_prefix("Size") {
                    if let Some(value) = value.split('=').nth(1) {
                        current_size = value.trim().parse::<u16>()? as u8;
                    }
                }
            }
            continue;
        }

        if trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with("*")
            || trimmed.starts_with("///")
        {
            continue;
        }
        if let Some(eq_index) = trimmed.find('=') {
            let name = trimmed[..eq_index].trim();
            if name.is_empty() {
                continue;
            }
            let mut value = trimmed[eq_index + 1..].trim();
            if let Some(idx) = value.find(',') {
                value = value[..idx].trim();
            }
            if let Some(idx) = value.find("//") {
                value = value[..idx].trim();
            }
            let byte = if let Some(hex) = value
                .strip_prefix("0x")
                .or_else(|| value.strip_prefix("0X"))
            {
                u8::from_str_radix(hex, 16)?
            } else {
                value.parse::<u16>()? as u8
            };

            entries.push(OpcodeEntry {
                name: name.to_string(),
                byte,
                operand_size: current_size,
                operand_size_prefix: current_prefix,
            });

            current_size = 0;
            current_prefix = 0;
        }
    }

    Ok(entries)
}

#[test]
fn neo_opcodes_match_reference() -> Result<(), Box<dyn std::error::Error>> {
    let mut expected = parse_neovm_opcodes()?;
    expected.sort_by_key(|entry| entry.byte);

    let mut actual: Vec<OpcodeEntry> = wasm_neovm::opcodes::all()
        .iter()
        .map(|info| OpcodeEntry {
            name: info.name.to_string(),
            byte: info.byte,
            operand_size: info.operand_size,
            operand_size_prefix: info.operand_size_prefix,
        })
        .collect();
    actual.sort_by_key(|entry| entry.byte);

    assert_eq!(expected.len(), actual.len(), "opcode count mismatch");

    for (exp, act) in expected.iter().zip(actual.iter()) {
        assert_eq!(
            exp.byte, act.byte,
            "opcode byte mismatch for {} vs {}",
            exp.name, act.name
        );
        assert_eq!(
            exp.operand_size, act.operand_size,
            "operand size mismatch for opcode {}",
            exp.name
        );
        assert_eq!(
            exp.operand_size_prefix, act.operand_size_prefix,
            "operand size prefix mismatch for opcode {}",
            exp.name
        );
        assert!(
            exp.name.eq_ignore_ascii_case(&act.name),
            "opcode name mismatch: expected {} got {}",
            exp.name,
            act.name
        );
    }

    Ok(())
}

use anyhow::{bail, Result};

use super::reader::BytecodeReader;
use super::types::{AbilitySet, BytecodeVersion, FunctionDef, MoveModule, MoveOpcode, StructDef};
use super::MOVE_MAGIC;

/// Parse Move bytecode into a module
pub fn parse_move_bytecode(bytes: &[u8]) -> Result<MoveModule> {
    if bytes.len() < 8 {
        bail!("Move bytecode too short: {} bytes", bytes.len());
    }

    // Check magic
    if bytes[0..4] != MOVE_MAGIC {
        bail!(
            "Invalid Move bytecode magic: expected {:02x?}, got {:02x?}",
            MOVE_MAGIC,
            &bytes[0..4]
        );
    }

    let mut reader = BytecodeReader::new(bytes);

    // Skip magic
    reader.read_bytes(4)?;

    // Read version
    let version = reader.read_u32()?;

    // Parse a minimal module header (table count only). This is not a full parser
    // but avoids rejecting valid Move artefacts while the lowering remains experimental.
    let mut module = parse_module_tables(&mut reader, version)?;
    let code_start = reader.position() as usize;

    // Resolve identifiers table for names if present
    let identifiers = read_identifiers(
        &mut reader,
        module.identifiers_offset,
        module.identifiers_count,
    )?;
    module.name = identifiers
        .first()
        .cloned()
        .unwrap_or_else(|| "MoveModule".to_string());
    module.structs = parse_structs(
        &mut reader,
        module.struct_defs_offset,
        module.struct_defs_count,
        &identifiers,
    )?;
    module.functions = parse_functions(
        &mut reader,
        module.function_defs_offset,
        module.function_defs_count,
        module.identifiers_offset,
        module.identifiers_count,
        &identifiers,
    )?;

    // If no functions are present, treat remaining bytes as a single entry function.
    if module.functions.is_empty() {
        let bytes = reader.bytes();
        let opcode_bytes = if code_start < bytes.len() {
            &bytes[code_start..]
        } else {
            &[]
        };

        let mut code_reader = BytecodeReader::new(opcode_bytes);
        let mut code = Vec::new();
        while code_reader.remaining() > 0 {
            code.push(parse_opcode(&mut code_reader)?);
        }
        if !code.is_empty() {
            module.functions.push(FunctionDef {
                name: "main".to_string(),
                is_public: true,
                is_entry: true,
                parameters: Vec::new(),
                returns: Vec::new(),
                locals: Vec::new(),
                code,
            });
        }
    }

    Ok(module)
}

fn parse_module_tables(reader: &mut BytecodeReader<'_>, version: u32) -> Result<MoveModule> {
    // Move bytecode structure:
    // - Table count (ULEB128)
    // - Table headers: [kind: u8, offset: u32, count: u32]
    // - Table contents

    let table_count = reader.read_uleb128()? as usize;

    let mut _module_handles_offset = 0u32;
    let mut _module_handles_count = 0u32;
    let mut _struct_handles_offset = 0u32;
    let mut _struct_handles_count = 0u32;
    let mut _function_handles_offset = 0u32;
    let mut _function_handles_count = 0u32;
    let mut identifiers_offset = 0u32;
    let mut identifiers_count = 0u32;
    let mut struct_defs_offset = 0u32;
    let mut struct_defs_count = 0u32;
    let mut function_defs_offset = 0u32;
    let mut function_defs_count = 0u32;

    // Read table headers
    for _ in 0..table_count {
        let kind = reader.read_u8()?;
        let offset = reader.read_u32()?;
        let count = reader.read_u32()?;

        match kind {
            0x01 => {
                _module_handles_offset = offset;
                _module_handles_count = count;
            }
            0x02 => {
                _struct_handles_offset = offset;
                _struct_handles_count = count;
            }
            0x03 => {
                _function_handles_offset = offset;
                _function_handles_count = count;
            }
            0x05 => {
                identifiers_offset = offset;
                identifiers_count = count;
            }
            0x08 => {
                struct_defs_offset = offset;
                struct_defs_count = count;
            }
            0x09 => {
                function_defs_offset = offset;
                function_defs_count = count;
            }
            _ => {} // Skip other tables
        }
    }

    Ok(MoveModule {
        version: BytecodeVersion(version),
        name: "MoveModule".to_string(),
        identifiers_offset,
        identifiers_count,
        struct_defs_offset,
        struct_defs_count,
        _function_handles_offset,
        _function_handles_count,
        function_defs_offset,
        function_defs_count,
        structs: Vec::new(),
        functions: Vec::new(),
    })
}

fn read_identifiers(
    reader: &mut BytecodeReader<'_>,
    identifiers_offset: u32,
    identifiers_count: u32,
) -> Result<Vec<String>> {
    if identifiers_count == 0 {
        return Ok(Vec::new());
    }
    let bytes = reader.bytes();
    let start = identifiers_offset as usize;
    if start > bytes.len() {
        bail!(
            "identifiers offset {} out of range for bytecode length {}",
            start,
            bytes.len()
        );
    }
    let mut names = Vec::new();
    let mut cursor = BytecodeReader::new(&bytes[start..]);
    for _ in 0..identifiers_count {
        names.push(cursor.read_string()?);
    }
    Ok(names)
}

fn parse_structs(
    _reader: &mut BytecodeReader<'_>,
    _offset: u32,
    count: u32,
    identifiers: &[String],
) -> Result<Vec<StructDef>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    let mut structs = Vec::new();
    for idx in 0..count {
        let name = identifiers
            .get(idx as usize)
            .cloned()
            .unwrap_or_else(|| format!("Struct{}", idx));
        structs.push(StructDef {
            name,
            abilities: AbilitySet::default(),
            fields: Vec::new(),
        });
    }
    Ok(structs)
}

fn parse_functions(
    reader: &mut BytecodeReader<'_>,
    offset: u32,
    count: u32,
    id_offset: u32,
    id_count: u32,
    identifiers: &[String],
) -> Result<Vec<FunctionDef>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    let mut funcs = Vec::new();
    let names = if id_count == identifiers.len() as u32 {
        identifiers.to_vec()
    } else {
        read_identifiers(reader, id_offset, id_count)?
    };
    let bytes = reader.bytes();
    let base = offset as usize;
    if base > bytes.len() {
        bail!(
            "function defs offset {} out of range for bytecode length {}",
            base,
            bytes.len()
        );
    }
    let mut cursor = BytecodeReader::new(&bytes[base..]);

    for _ in 0..count {
        let name_idx = cursor.read_uleb128()? as usize;
        let _flags = cursor.read_u8()?; // visibility/entry flags
        let _params_count = cursor.read_uleb128()?;
        let _returns_count = cursor.read_uleb128()?;
        let _code_offset = cursor.read_u32()?;
        let _locals_count = cursor.read_uleb128()?;

        let name = names
            .get(name_idx)
            .cloned()
            .unwrap_or_else(|| "fn".to_string());
        funcs.push(FunctionDef {
            name,
            is_public: true,
            is_entry: true,
            parameters: Vec::new(),
            returns: Vec::new(),
            locals: Vec::new(),
            code: Vec::new(),
        });
    }

    Ok(funcs)
}

fn parse_opcode(reader: &mut BytecodeReader<'_>) -> Result<MoveOpcode> {
    let op = reader.read_u8()?;

    let opcode = match op {
        0x00 => MoveOpcode::Nop,
        0x01 => MoveOpcode::Pop,
        0x02 => MoveOpcode::Ret,
        0x03 => MoveOpcode::BrTrue(reader.read_u16()?),
        0x04 => MoveOpcode::BrFalse(reader.read_u16()?),
        0x05 => MoveOpcode::Branch(reader.read_u16()?),
        0x06 => MoveOpcode::LdU8(reader.read_u8()?),
        0x07 => MoveOpcode::LdU64(reader.read_u64()?),
        0x08 => MoveOpcode::LdU128(reader.read_u128()?),
        0x09 => MoveOpcode::CastU8,
        0x0A => MoveOpcode::CastU64,
        0x0B => MoveOpcode::CastU128,
        0x0C => MoveOpcode::LdConst(reader.read_u16()?),
        0x0D => MoveOpcode::LdTrue,
        0x0E => MoveOpcode::LdFalse,
        0x0F => MoveOpcode::CopyLoc(reader.read_u8()?),
        0x10 => MoveOpcode::MoveLoc(reader.read_u8()?),
        0x11 => MoveOpcode::StLoc(reader.read_u8()?),
        0x12 => MoveOpcode::MutBorrowLoc(reader.read_u8()?),
        0x13 => MoveOpcode::ImmBorrowLoc(reader.read_u8()?),
        0x14 => MoveOpcode::MutBorrowField(reader.read_u16()?),
        0x15 => MoveOpcode::BorrowField(reader.read_u16()?),
        0x16 => MoveOpcode::Call(reader.read_u16()?),
        0x17 => MoveOpcode::Pack(reader.read_u16()?),
        0x18 => MoveOpcode::Unpack(reader.read_u16()?),
        0x22 => MoveOpcode::Add,
        0x23 => MoveOpcode::Sub,
        0x24 => MoveOpcode::Mul,
        0x25 => MoveOpcode::Mod,
        0x26 => MoveOpcode::Div,
        0x32 => MoveOpcode::Lt,
        0x33 => MoveOpcode::Gt,
        0x34 => MoveOpcode::Le,
        0x35 => MoveOpcode::Ge,
        0x40 => MoveOpcode::And,
        0x41 => MoveOpcode::Or,
        0x42 => MoveOpcode::Not,
        0x43 => MoveOpcode::Eq,
        0x44 => MoveOpcode::Neq,
        0x45 => MoveOpcode::Abort,
        0x50 => MoveOpcode::Exists(reader.read_u16()?),
        0x51 => MoveOpcode::BorrowGlobal(reader.read_u16()?),
        0x52 => MoveOpcode::MutBorrowGlobal(reader.read_u16()?),
        0x53 => MoveOpcode::MoveFrom(reader.read_u16()?),
        0x54 => MoveOpcode::MoveTo(reader.read_u16()?),
        other => bail!("unsupported Move opcode 0x{other:02X}"),
    };

    Ok(opcode)
}

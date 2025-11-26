//! Move bytecode parsing module
//!
//! This module handles parsing of Move compiled bytecode.
//!
//! # Move Bytecode Format
//!
//! Move bytecode consists of:
//! - Module handles (references to other modules)
//! - Struct definitions (including resources)
//! - Function definitions
//! - Code sections with instructions

use anyhow::{bail, Context, Result};
use std::io::{Cursor, Read};

/// Move bytecode magic bytes: 0xa1, 0x1c, 0xeb, 0x0b
const MOVE_MAGIC: [u8; 4] = [0xa1, 0x1c, 0xeb, 0x0b];

/// Move bytecode version
#[derive(Debug, Clone, Copy)]
pub struct BytecodeVersion(pub u32);

/// A parsed Move module
#[derive(Debug, Clone)]
pub struct MoveModule {
    /// Module version
    pub version: BytecodeVersion,
    /// Module name
    pub name: String,
    /// Struct definitions
    pub structs: Vec<StructDef>,
    /// Function definitions
    pub functions: Vec<FunctionDef>,
}

/// A struct (or resource) definition
#[derive(Debug, Clone)]
pub struct StructDef {
    /// Struct name
    pub name: String,
    /// Is this a resource type?
    pub is_resource: bool,
    /// Field definitions
    pub fields: Vec<FieldDef>,
}

/// A field definition
#[derive(Debug, Clone)]
pub struct FieldDef {
    /// Field name
    pub name: String,
    /// Field type
    pub type_tag: TypeTag,
}

/// A function definition
#[derive(Debug, Clone)]
pub struct FunctionDef {
    /// Function name
    pub name: String,
    /// Is this a public function?
    pub is_public: bool,
    /// Is this an entry function?
    pub is_entry: bool,
    /// Parameter types
    pub parameters: Vec<TypeTag>,
    /// Return types
    pub returns: Vec<TypeTag>,
    /// Function body (opcodes)
    pub code: Vec<MoveOpcode>,
}

/// Move type tags
#[derive(Debug, Clone)]
pub enum TypeTag {
    Bool,
    U8,
    U64,
    U128,
    U256,
    Address,
    Signer,
    Vector(Box<TypeTag>),
    Struct(String),
    Reference(Box<TypeTag>),
    MutableReference(Box<TypeTag>),
}

impl TypeTag {
    /// Convert to WASM-compatible representation
    pub fn to_wasm_type(&self) -> &'static str {
        match self {
            TypeTag::Bool => "i32",
            TypeTag::U8 => "i32",
            TypeTag::U64 => "i64",
            TypeTag::U128 => "i64", // Requires multi-word handling
            TypeTag::U256 => "i64", // Requires multi-word handling
            TypeTag::Address => "i32", // Pointer to 32-byte array
            TypeTag::Signer => "i32", // Pointer to signer data
            TypeTag::Vector(_) => "i32", // Pointer to vector
            TypeTag::Struct(_) => "i32", // Pointer to struct
            TypeTag::Reference(_) => "i32",
            TypeTag::MutableReference(_) => "i32",
        }
    }
}

/// Move VM opcodes (simplified subset)
#[derive(Debug, Clone)]
pub enum MoveOpcode {
    // Constants
    LdU8(u8),
    LdU64(u64),
    LdU128(u128),
    LdTrue,
    LdFalse,
    LdConst(u16), // Constant pool index

    // Local operations
    CopyLoc(u8),
    MoveLoc(u8),
    StLoc(u8),
    MutBorrowLoc(u8),
    ImmBorrowLoc(u8),

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Neq,

    // Logical
    And,
    Or,
    Not,

    // Control flow
    Branch(u16),
    BrTrue(u16),
    BrFalse(u16),
    Call(u16), // Function index
    Ret,
    Abort,

    // Resource operations
    Pack(u16),      // Struct index
    Unpack(u16),
    BorrowField(u16),
    MutBorrowField(u16),
    MoveFrom(u16),  // Struct index
    MoveTo(u16),
    Exists(u16),
    BorrowGlobal(u16),
    MutBorrowGlobal(u16),

    // Stack
    Pop,

    // Vector operations
    VecPack(u16, u64),
    VecLen(u16),
    VecImmBorrow(u16),
    VecMutBorrow(u16),
    VecPushBack(u16),
    VecPopBack(u16),

    // Casting
    CastU8,
    CastU64,
    CastU128,

    // Nop (placeholder for unsupported)
    Nop,
}

impl MoveOpcode {
    /// Get opcode byte value
    pub fn opcode_byte(&self) -> u8 {
        match self {
            MoveOpcode::Pop => 0x01,
            MoveOpcode::Ret => 0x02,
            MoveOpcode::BrTrue(_) => 0x03,
            MoveOpcode::BrFalse(_) => 0x04,
            MoveOpcode::Branch(_) => 0x05,
            MoveOpcode::LdU8(_) => 0x06,
            MoveOpcode::LdU64(_) => 0x07,
            MoveOpcode::LdU128(_) => 0x08,
            MoveOpcode::CastU8 => 0x09,
            MoveOpcode::CastU64 => 0x0A,
            MoveOpcode::CastU128 => 0x0B,
            MoveOpcode::LdConst(_) => 0x0C,
            MoveOpcode::LdTrue => 0x0D,
            MoveOpcode::LdFalse => 0x0E,
            MoveOpcode::CopyLoc(_) => 0x0F,
            MoveOpcode::MoveLoc(_) => 0x10,
            MoveOpcode::StLoc(_) => 0x11,
            MoveOpcode::MutBorrowLoc(_) => 0x12,
            MoveOpcode::ImmBorrowLoc(_) => 0x13,
            MoveOpcode::MutBorrowField(_) => 0x14,
            MoveOpcode::BorrowField(_) => 0x15,
            MoveOpcode::Call(_) => 0x16,
            MoveOpcode::Pack(_) => 0x17,
            MoveOpcode::Unpack(_) => 0x18,
            MoveOpcode::Add => 0x22,
            MoveOpcode::Sub => 0x23,
            MoveOpcode::Mul => 0x24,
            MoveOpcode::Mod => 0x25,
            MoveOpcode::Div => 0x26,
            MoveOpcode::Lt => 0x32,
            MoveOpcode::Gt => 0x33,
            MoveOpcode::Le => 0x34,
            MoveOpcode::Ge => 0x35,
            MoveOpcode::And => 0x40,
            MoveOpcode::Or => 0x41,
            MoveOpcode::Not => 0x42,
            MoveOpcode::Eq => 0x43,
            MoveOpcode::Neq => 0x44,
            MoveOpcode::Abort => 0x45,
            MoveOpcode::Exists(_) => 0x50,
            MoveOpcode::BorrowGlobal(_) => 0x51,
            MoveOpcode::MutBorrowGlobal(_) => 0x52,
            MoveOpcode::MoveFrom(_) => 0x53,
            MoveOpcode::MoveTo(_) => 0x54,
            MoveOpcode::VecPack(_, _) => 0x60,
            MoveOpcode::VecLen(_) => 0x61,
            MoveOpcode::VecImmBorrow(_) => 0x62,
            MoveOpcode::VecMutBorrow(_) => 0x63,
            MoveOpcode::VecPushBack(_) => 0x64,
            MoveOpcode::VecPopBack(_) => 0x65,
            MoveOpcode::Nop => 0x00,
        }
    }
}

/// Bytecode reader helper
#[allow(dead_code)]
struct BytecodeReader<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> BytecodeReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(bytes),
        }
    }

    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.cursor.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.cursor.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_u64(&mut self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.cursor.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    fn read_u128(&mut self) -> Result<u128> {
        let mut buf = [0u8; 16];
        self.cursor.read_exact(&mut buf)?;
        Ok(u128::from_le_bytes(buf))
    }

    fn read_uleb128(&mut self) -> Result<u64> {
        let mut result: u64 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7f) as u64) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift >= 64 {
                bail!("ULEB128 overflow");
            }
        }
        Ok(result)
    }

    fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf)
    }

    #[allow(dead_code)]
    fn read_string(&mut self) -> Result<String> {
        let len = self.read_uleb128()? as usize;
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes).context("Invalid UTF-8 in string")
    }

    #[allow(dead_code)]
    fn position(&self) -> u64 {
        self.cursor.position()
    }

    #[allow(dead_code)]
    fn remaining(&self) -> usize {
        let pos = self.cursor.position() as usize;
        let len = self.cursor.get_ref().len();
        len.saturating_sub(pos)
    }
}

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

    // For now, parse a simplified structure
    // Full Move bytecode has complex table-based format
    let module = parse_module_tables(&mut reader, version)?;

    Ok(module)
}

fn parse_module_tables(reader: &mut BytecodeReader, version: u32) -> Result<MoveModule> {
    // Move bytecode structure:
    // - Table count (ULEB128)
    // - Table headers: [kind: u8, offset: u32, count: u32]
    // - Table contents

    let table_count = reader.read_uleb128()? as usize;

    let mut module_handles_offset = 0u32;
    let mut module_handles_count = 0u32;
    let mut struct_handles_offset = 0u32;
    let mut struct_handles_count = 0u32;
    let mut function_handles_offset = 0u32;
    let mut function_handles_count = 0u32;
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
            0x01 => { module_handles_offset = offset; module_handles_count = count; }
            0x02 => { struct_handles_offset = offset; struct_handles_count = count; }
            0x03 => { function_handles_offset = offset; function_handles_count = count; }
            0x05 => { identifiers_offset = offset; identifiers_count = count; }
            0x08 => { struct_defs_offset = offset; struct_defs_count = count; }
            0x09 => { function_defs_offset = offset; function_defs_count = count; }
            _ => {} // Skip other tables
        }
    }

    // For now, create a basic module structure
    // Full parsing would read each table based on offsets
    let module = MoveModule {
        version: BytecodeVersion(version),
        name: "MoveModule".to_string(),
        structs: Vec::new(),
        functions: Vec::new(),
    };

    // TODO: Parse actual table contents using offsets
    let _ = (module_handles_offset, module_handles_count);
    let _ = (struct_handles_offset, struct_handles_count);
    let _ = (function_handles_offset, function_handles_count);
    let _ = (identifiers_offset, identifiers_count);
    let _ = (struct_defs_offset, struct_defs_count);
    let _ = (function_defs_offset, function_defs_count);

    Ok(module)
}

/// Validate Move bytecode without full parsing
pub fn validate_move_bytecode(bytes: &[u8]) -> bool {
    // Check magic bytes
    if bytes.len() < 4 {
        return false;
    }
    // Move module magic: 0xa1, 0x1c, 0xeb, 0x0b
    bytes[0..4] == MOVE_MAGIC
}

/// Parse a single opcode from bytecode (internal use)
#[allow(dead_code)]
fn parse_opcode(reader: &mut BytecodeReader) -> Result<MoveOpcode> {
    let op = reader.read_u8()?;

    let opcode = match op {
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
        _ => MoveOpcode::Nop,
    };

    Ok(opcode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_move_bytecode() {
        // Valid magic
        let valid = [0xa1, 0x1c, 0xeb, 0x0b, 0x00, 0x00, 0x00, 0x00];
        assert!(validate_move_bytecode(&valid));

        // Invalid magic
        let invalid = [0x00, 0x61, 0x73, 0x6d];
        assert!(!validate_move_bytecode(&invalid));

        // Too short
        let short = [0xa1, 0x1c];
        assert!(!validate_move_bytecode(&short));
    }

    #[test]
    fn test_bytecode_reader() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut reader = BytecodeReader::new(&data);

        assert_eq!(reader.read_u8().unwrap(), 0x01);
        assert_eq!(reader.read_u16().unwrap(), 0x0302);
        assert_eq!(reader.position(), 3);
        assert_eq!(reader.remaining(), 5);
    }

    #[test]
    fn test_uleb128_decoding() {
        // Single byte: 0x7f = 127
        let mut reader = BytecodeReader::new(&[0x7f]);
        assert_eq!(reader.read_uleb128().unwrap(), 127);

        // Two bytes: 0x80 0x01 = 128
        let mut reader = BytecodeReader::new(&[0x80, 0x01]);
        assert_eq!(reader.read_uleb128().unwrap(), 128);

        // Three bytes: 0xe5 0x8e 0x26 = 624485
        let mut reader = BytecodeReader::new(&[0xe5, 0x8e, 0x26]);
        assert_eq!(reader.read_uleb128().unwrap(), 624485);
    }
}

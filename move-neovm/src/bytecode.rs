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

mod parser;
mod reader;
mod types;
mod validate;

pub use parser::parse_move_bytecode;
pub use types::{
    AbilitySet, BytecodeVersion, FieldDef, FunctionDef, MoveModule, MoveOpcode, StructDef, TypeTag,
};
pub use validate::validate_move_bytecode;

/// Move bytecode magic bytes: 0xa1, 0x1c, 0xeb, 0x0b
const MOVE_MAGIC: [u8; 4] = [0xa1, 0x1c, 0xeb, 0x0b];

#[cfg(test)]
mod tests {
    use super::reader::BytecodeReader;
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

    #[test]
    fn parses_minimal_move_module_with_code() {
        // Magic + version + one opcode (LdU8 7) with no tables
        let bytes = [
            0xa1, 0x1c, 0xeb, 0x0b, // magic
            0x06, 0x00, 0x00, 0x00, // version
            0x00, // table count = 0
            0x06, 0x07, // LdU8 7
            0x02, // Ret
        ];

        let module = parse_move_bytecode(&bytes).expect("parse minimal move");
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "main");
        assert!(!module.functions[0].code.is_empty());
    }
}

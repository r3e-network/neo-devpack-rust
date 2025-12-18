//! Move to WASM/NeoVM translator
//!
//! This module lowers a parsed `MoveModule` into a plain WASM module that can
//! then be consumed by the `wasm-neovm` pipeline. The lowering is deliberately
//! conservative: it performs basic stack/type analysis, rejects ability
//! violations, and encodes control flow using a dispatch loop so that branch
//! depths remain valid for arbitrary Move bytecode.

mod analysis;
mod lowering;
mod resources;

pub use lowering::{translate_to_wasm, ImportLayout, ValueKind};
// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{AbilitySet, FunctionDef, MoveModule, MoveOpcode, StructDef, TypeTag};

    fn validate_wasm(bytes: &[u8]) -> Result<(), wasmparser::BinaryReaderError> {
        wasmparser::Validator::new().validate_all(bytes).map(|_| ())
    }

    #[test]
    fn translates_simple_add() {
        let module = MoveModule {
            version: crate::bytecode::BytecodeVersion(6),
            name: "Test".into(),
            identifiers_offset: 0,
            identifiers_count: 0,
            struct_defs_offset: 0,
            struct_defs_count: 0,
            _function_handles_offset: 0,
            _function_handles_count: 0,
            function_defs_offset: 0,
            function_defs_count: 0,
            structs: vec![],
            functions: vec![FunctionDef {
                name: "add".into(),
                is_public: true,
                is_entry: false,
                parameters: vec![TypeTag::U64, TypeTag::U64],
                returns: vec![TypeTag::U64],
                locals: vec![],
                code: vec![
                    MoveOpcode::CopyLoc(0),
                    MoveOpcode::CopyLoc(1),
                    MoveOpcode::Add,
                    MoveOpcode::Ret,
                ],
            }],
        };

        let bytes = translate_to_wasm(&module).unwrap();
        if std::env::var("DUMP_WASM").is_ok() {
            std::fs::write("/tmp/move_add.wasm", &bytes).unwrap();
        }
        validate_wasm(&bytes).expect("valid wasm for add");
    }

    #[test]
    fn handles_branch_control_flow() {
        let module = MoveModule {
            version: crate::bytecode::BytecodeVersion(6),
            name: "Test".into(),
            identifiers_offset: 0,
            identifiers_count: 0,
            struct_defs_offset: 0,
            struct_defs_count: 0,
            _function_handles_offset: 0,
            _function_handles_count: 0,
            function_defs_offset: 0,
            function_defs_count: 0,
            structs: vec![],
            functions: vec![FunctionDef {
                name: "abs".into(),
                is_public: true,
                is_entry: false,
                parameters: vec![TypeTag::U64],
                returns: vec![TypeTag::U64],
                locals: vec![TypeTag::U64],
                code: vec![
                    MoveOpcode::CopyLoc(0),
                    MoveOpcode::LdU64(0),
                    MoveOpcode::Lt,
                    MoveOpcode::BrFalse(4),
                    MoveOpcode::CopyLoc(0),
                    MoveOpcode::Ret,
                    MoveOpcode::CopyLoc(0),
                    MoveOpcode::LdU64(0),
                    MoveOpcode::Sub,
                    MoveOpcode::Ret,
                ],
            }],
        };

        let bytes = translate_to_wasm(&module).unwrap();
        validate_wasm(&bytes).expect("valid wasm for branches");
    }

    #[test]
    fn rejects_copy_of_resource_without_ability() {
        let module = MoveModule {
            version: crate::bytecode::BytecodeVersion(6),
            name: "Test".into(),
            identifiers_offset: 0,
            identifiers_count: 0,
            struct_defs_offset: 0,
            struct_defs_count: 0,
            _function_handles_offset: 0,
            _function_handles_count: 0,
            function_defs_offset: 0,
            function_defs_count: 0,
            structs: vec![StructDef {
                name: "R".into(),
                abilities: AbilitySet {
                    key: true,
                    store: true,
                    copy: false,
                    drop: false,
                },
                fields: vec![],
            }],
            functions: vec![FunctionDef {
                name: "bad".into(),
                is_public: true,
                is_entry: false,
                parameters: vec![TypeTag::Struct("R".into())],
                returns: vec![],
                locals: vec![],
                code: vec![MoveOpcode::CopyLoc(0), MoveOpcode::Pop, MoveOpcode::Ret],
            }],
        };

        let err = translate_to_wasm(&module).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("copy of resource"), "unexpected error {err:?}");
    }

    #[test]
    fn lowers_resource_ops() {
        let module = MoveModule {
            version: crate::bytecode::BytecodeVersion(6),
            name: "Resource".into(),
            identifiers_offset: 0,
            identifiers_count: 0,
            struct_defs_offset: 0,
            struct_defs_count: 0,
            _function_handles_offset: 0,
            _function_handles_count: 0,
            function_defs_offset: 0,
            function_defs_count: 0,
            structs: vec![StructDef {
                name: "Coin".into(),
                abilities: AbilitySet {
                    key: true,
                    store: true,
                    copy: true,
                    drop: true,
                },
                fields: vec![],
            }],
            functions: vec![FunctionDef {
                name: "mint".into(),
                is_public: true,
                is_entry: true,
                parameters: vec![TypeTag::Address, TypeTag::U64],
                returns: vec![],
                locals: vec![TypeTag::Address, TypeTag::U64],
                code: vec![
                    MoveOpcode::CopyLoc(0),
                    MoveOpcode::CopyLoc(1),
                    MoveOpcode::MoveTo(0),
                    MoveOpcode::Ret,
                ],
            }],
        };

        let bytes = translate_to_wasm(&module).unwrap();
        validate_wasm(&bytes).expect("valid wasm for resource ops");
    }
}

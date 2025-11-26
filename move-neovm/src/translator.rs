//! Move to WASM/NeoVM translator
//!
//! Translates parsed Move modules to WASM bytecode that can then be processed
//! by wasm-neovm to generate NeoVM NEF artifacts.

use crate::bytecode::{FunctionDef, MoveModule, MoveOpcode, TypeTag};
use anyhow::Result;

/// WASM section codes
const WASM_SECTION_TYPE: u8 = 0x01;
const WASM_SECTION_IMPORT: u8 = 0x02;
const WASM_SECTION_FUNCTION: u8 = 0x03;
const WASM_SECTION_EXPORT: u8 = 0x07;
const WASM_SECTION_CODE: u8 = 0x0a;

/// WASM type codes
const WASM_TYPE_I32: u8 = 0x7f;
const WASM_TYPE_I64: u8 = 0x7e;
const WASM_TYPE_FUNC: u8 = 0x60;

/// WASM instruction opcodes
#[allow(dead_code)]
mod wasm_op {
    pub const UNREACHABLE: u8 = 0x00;
    pub const NOP: u8 = 0x01;
    pub const BLOCK: u8 = 0x02;
    pub const LOOP: u8 = 0x03;
    pub const IF: u8 = 0x04;
    pub const ELSE: u8 = 0x05;
    pub const END: u8 = 0x0b;
    pub const BR: u8 = 0x0c;
    pub const BR_IF: u8 = 0x0d;
    pub const RETURN: u8 = 0x0f;
    pub const CALL: u8 = 0x10;
    pub const DROP: u8 = 0x1a;
    pub const LOCAL_GET: u8 = 0x20;
    pub const LOCAL_SET: u8 = 0x21;
    pub const LOCAL_TEE: u8 = 0x22;
    pub const I32_CONST: u8 = 0x41;
    pub const I64_CONST: u8 = 0x42;
    pub const I32_EQZ: u8 = 0x45;
    pub const I32_EQ: u8 = 0x46;
    pub const I32_NE: u8 = 0x47;
    pub const I32_LT_S: u8 = 0x48;
    pub const I32_GT_S: u8 = 0x4a;
    pub const I32_LE_S: u8 = 0x4c;
    pub const I32_GE_S: u8 = 0x4e;
    pub const I64_EQZ: u8 = 0x50;
    pub const I64_EQ: u8 = 0x51;
    pub const I64_NE: u8 = 0x52;
    pub const I64_LT_S: u8 = 0x53;
    pub const I64_GT_S: u8 = 0x55;
    pub const I64_LE_S: u8 = 0x57;
    pub const I64_GE_S: u8 = 0x59;
    pub const I32_ADD: u8 = 0x6a;
    pub const I32_SUB: u8 = 0x6b;
    pub const I32_MUL: u8 = 0x6c;
    pub const I32_DIV_S: u8 = 0x6d;
    pub const I32_REM_S: u8 = 0x6f;
    pub const I32_AND: u8 = 0x71;
    pub const I32_OR: u8 = 0x72;
    pub const I64_ADD: u8 = 0x7c;
    pub const I64_SUB: u8 = 0x7d;
    pub const I64_MUL: u8 = 0x7e;
    pub const I64_DIV_S: u8 = 0x7f;
    pub const I64_REM_S: u8 = 0x81;
    pub const I64_AND: u8 = 0x83;
    pub const I64_OR: u8 = 0x84;
    pub const I32_WRAP_I64: u8 = 0xa7;
    pub const I64_EXTEND_I32_S: u8 = 0xac;
}

/// Translation context
pub struct TranslationContext {
    /// Output buffer
    pub output: Vec<u8>,
    /// Local variable mappings
    pub locals: Vec<LocalMapping>,
    /// Label/offset mappings for branches
    pub labels: Vec<usize>,
    /// Current function being translated
    pub current_func: Option<String>,
    /// Import function count (offset for local function indices)
    pub import_count: u32,
}

/// Mapping from Move local to WASM local slot
#[derive(Debug, Clone)]
pub struct LocalMapping {
    /// Move local index
    pub move_idx: u8,
    /// Target local index
    pub target_idx: u32,
    /// Type information
    pub type_tag: TypeTag,
}

impl TranslationContext {
    pub fn new() -> Self {
        Self {
            output: Vec::new(),
            locals: Vec::new(),
            labels: Vec::new(),
            current_func: None,
            import_count: 0,
        }
    }

    /// Write a byte
    fn emit(&mut self, byte: u8) {
        self.output.push(byte);
    }

    /// Write bytes
    fn emit_bytes(&mut self, bytes: &[u8]) {
        self.output.extend_from_slice(bytes);
    }

    /// Write unsigned LEB128
    fn emit_uleb128(&mut self, mut value: u64) {
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                self.emit(byte);
                break;
            } else {
                self.emit(byte | 0x80);
            }
        }
    }

    /// Write signed LEB128
    #[allow(dead_code)]
    fn emit_sleb128(&mut self, mut value: i64) {
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            let more = !((value == 0 && byte & 0x40 == 0) || (value == -1 && byte & 0x40 != 0));
            if more {
                self.emit(byte | 0x80);
            } else {
                self.emit(byte);
                break;
            }
        }
    }

    /// Write a length-prefixed vector
    #[allow(dead_code)]
    fn emit_vec<F>(&mut self, count: u32, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.emit_uleb128(count as u64);
        f(self);
    }

    /// Write a section
    fn emit_section(&mut self, id: u8, content: Vec<u8>) {
        self.emit(id);
        self.emit_uleb128(content.len() as u64);
        self.emit_bytes(&content);
    }
}

impl Default for TranslationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Translate a Move module to WASM bytes
pub fn translate_to_wasm(module: &MoveModule) -> Result<Vec<u8>> {
    let mut ctx = TranslationContext::new();

    // WASM magic and version
    ctx.emit_bytes(&[0x00, 0x61, 0x73, 0x6d]); // "\0asm"
    ctx.emit_bytes(&[0x01, 0x00, 0x00, 0x00]); // version 1

    // Build type section
    let type_section = build_type_section(module)?;
    if !type_section.is_empty() {
        ctx.emit_section(WASM_SECTION_TYPE, type_section);
    }

    // Build import section (Neo syscalls)
    let import_section = build_import_section()?;
    if !import_section.is_empty() {
        ctx.emit_section(WASM_SECTION_IMPORT, import_section);
        ctx.import_count = 2; // storage_get, storage_put
    }

    // Build function section
    let func_section = build_function_section(module)?;
    if !func_section.is_empty() {
        ctx.emit_section(WASM_SECTION_FUNCTION, func_section);
    }

    // Build export section
    let export_section = build_export_section(module, ctx.import_count)?;
    if !export_section.is_empty() {
        ctx.emit_section(WASM_SECTION_EXPORT, export_section);
    }

    // Build code section
    let code_section = build_code_section(module)?;
    if !code_section.is_empty() {
        ctx.emit_section(WASM_SECTION_CODE, code_section);
    }

    Ok(ctx.output)
}

fn build_type_section(module: &MoveModule) -> Result<Vec<u8>> {
    let mut buf = Vec::new();

    // Count function signatures (deduplicated in real impl)
    let func_count = module.functions.len() + 2; // +2 for import types
    emit_uleb128_to(&mut buf, func_count as u64);

    // Import function types
    // storage_get: (i32, i32) -> i64
    buf.push(WASM_TYPE_FUNC);
    emit_uleb128_to(&mut buf, 2); // 2 params
    buf.push(WASM_TYPE_I32);
    buf.push(WASM_TYPE_I32);
    emit_uleb128_to(&mut buf, 1); // 1 result
    buf.push(WASM_TYPE_I64);

    // storage_put: (i32, i32, i32, i32) -> void
    buf.push(WASM_TYPE_FUNC);
    emit_uleb128_to(&mut buf, 4); // 4 params
    buf.push(WASM_TYPE_I32);
    buf.push(WASM_TYPE_I32);
    buf.push(WASM_TYPE_I32);
    buf.push(WASM_TYPE_I32);
    emit_uleb128_to(&mut buf, 0); // 0 results

    // Add types for each function
    for func in &module.functions {
        buf.push(WASM_TYPE_FUNC);

        // Parameters
        emit_uleb128_to(&mut buf, func.parameters.len() as u64);
        for param in &func.parameters {
            buf.push(type_tag_to_wasm(param));
        }

        // Returns
        emit_uleb128_to(&mut buf, func.returns.len() as u64);
        for ret in &func.returns {
            buf.push(type_tag_to_wasm(ret));
        }
    }

    Ok(buf)
}

fn build_import_section() -> Result<Vec<u8>> {
    let mut buf = Vec::new();

    // 2 imports
    emit_uleb128_to(&mut buf, 2);

    // Import: neo.storage_get
    emit_string_to(&mut buf, "neo");
    emit_string_to(&mut buf, "storage_get");
    buf.push(0x00); // func import
    emit_uleb128_to(&mut buf, 0); // type index 0

    // Import: neo.storage_put
    emit_string_to(&mut buf, "neo");
    emit_string_to(&mut buf, "storage_put");
    buf.push(0x00); // func import
    emit_uleb128_to(&mut buf, 1); // type index 1

    Ok(buf)
}

fn build_function_section(module: &MoveModule) -> Result<Vec<u8>> {
    let mut buf = Vec::new();

    emit_uleb128_to(&mut buf, module.functions.len() as u64);

    for (i, _func) in module.functions.iter().enumerate() {
        // Type index (offset by import types)
        emit_uleb128_to(&mut buf, (i + 2) as u64);
    }

    Ok(buf)
}

fn build_export_section(module: &MoveModule, import_count: u32) -> Result<Vec<u8>> {
    let mut buf = Vec::new();

    // Export public/entry functions
    let exports: Vec<_> = module
        .functions
        .iter()
        .enumerate()
        .filter(|(_, f)| f.is_public || f.is_entry)
        .collect();

    emit_uleb128_to(&mut buf, exports.len() as u64);

    for (i, func) in exports {
        emit_string_to(&mut buf, &func.name);
        buf.push(0x00); // func export
        emit_uleb128_to(&mut buf, (import_count + i as u32) as u64);
    }

    Ok(buf)
}

fn build_code_section(module: &MoveModule) -> Result<Vec<u8>> {
    let mut buf = Vec::new();

    emit_uleb128_to(&mut buf, module.functions.len() as u64);

    for func in &module.functions {
        let func_body = translate_function(func)?;
        emit_uleb128_to(&mut buf, func_body.len() as u64);
        buf.extend_from_slice(&func_body);
    }

    Ok(buf)
}

fn translate_function(func: &FunctionDef) -> Result<Vec<u8>> {
    let mut buf = Vec::new();

    // Local declarations (simplified: just count of i64 locals)
    emit_uleb128_to(&mut buf, 1); // 1 local group
    emit_uleb128_to(&mut buf, 4); // 4 locals
    buf.push(WASM_TYPE_I64);

    // Translate opcodes
    for opcode in &func.code {
        translate_opcode_to(&mut buf, opcode)?;
    }

    // End
    buf.push(wasm_op::END);

    Ok(buf)
}

/// Translate a single Move opcode to WASM instructions
fn translate_opcode_to(buf: &mut Vec<u8>, opcode: &MoveOpcode) -> Result<()> {
    match opcode {
        // Constants
        MoveOpcode::LdU8(v) => {
            buf.push(wasm_op::I32_CONST);
            emit_sleb128_to(buf, *v as i64);
        }
        MoveOpcode::LdU64(v) => {
            buf.push(wasm_op::I64_CONST);
            emit_sleb128_to(buf, *v as i64);
        }
        MoveOpcode::LdU128(v) => {
            // Simplified: just use lower 64 bits
            buf.push(wasm_op::I64_CONST);
            emit_sleb128_to(buf, *v as i64);
        }
        MoveOpcode::LdTrue => {
            buf.push(wasm_op::I32_CONST);
            emit_sleb128_to(buf, 1);
        }
        MoveOpcode::LdFalse => {
            buf.push(wasm_op::I32_CONST);
            emit_sleb128_to(buf, 0);
        }
        MoveOpcode::LdConst(idx) => {
            // Load from constant pool (simplified)
            buf.push(wasm_op::I64_CONST);
            emit_sleb128_to(buf, *idx as i64);
        }

        // Local operations
        MoveOpcode::CopyLoc(idx) | MoveOpcode::MoveLoc(idx) => {
            buf.push(wasm_op::LOCAL_GET);
            emit_uleb128_to(buf, *idx as u64);
        }
        MoveOpcode::StLoc(idx) => {
            buf.push(wasm_op::LOCAL_SET);
            emit_uleb128_to(buf, *idx as u64);
        }
        MoveOpcode::MutBorrowLoc(idx) | MoveOpcode::ImmBorrowLoc(idx) => {
            // References are pointers in WASM
            buf.push(wasm_op::LOCAL_GET);
            emit_uleb128_to(buf, *idx as u64);
        }

        // Arithmetic (i64)
        MoveOpcode::Add => buf.push(wasm_op::I64_ADD),
        MoveOpcode::Sub => buf.push(wasm_op::I64_SUB),
        MoveOpcode::Mul => buf.push(wasm_op::I64_MUL),
        MoveOpcode::Div => buf.push(wasm_op::I64_DIV_S),
        MoveOpcode::Mod => buf.push(wasm_op::I64_REM_S),

        // Comparison
        MoveOpcode::Lt => buf.push(wasm_op::I64_LT_S),
        MoveOpcode::Gt => buf.push(wasm_op::I64_GT_S),
        MoveOpcode::Le => buf.push(wasm_op::I64_LE_S),
        MoveOpcode::Ge => buf.push(wasm_op::I64_GE_S),
        MoveOpcode::Eq => buf.push(wasm_op::I64_EQ),
        MoveOpcode::Neq => buf.push(wasm_op::I64_NE),

        // Logical
        MoveOpcode::And => buf.push(wasm_op::I64_AND),
        MoveOpcode::Or => buf.push(wasm_op::I64_OR),
        MoveOpcode::Not => {
            buf.push(wasm_op::I64_EQZ);
        }

        // Control flow
        MoveOpcode::Branch(offset) => {
            buf.push(wasm_op::BR);
            emit_uleb128_to(buf, *offset as u64);
        }
        MoveOpcode::BrTrue(offset) => {
            buf.push(wasm_op::BR_IF);
            emit_uleb128_to(buf, *offset as u64);
        }
        MoveOpcode::BrFalse(offset) => {
            buf.push(wasm_op::I64_EQZ);
            buf.push(wasm_op::BR_IF);
            emit_uleb128_to(buf, *offset as u64);
        }
        MoveOpcode::Call(idx) => {
            buf.push(wasm_op::CALL);
            // Offset by import count (2)
            emit_uleb128_to(buf, (*idx + 2) as u64);
        }
        MoveOpcode::Ret => {
            buf.push(wasm_op::RETURN);
        }
        MoveOpcode::Abort => {
            buf.push(wasm_op::UNREACHABLE);
        }

        // Resource operations (map to storage syscalls)
        MoveOpcode::MoveTo(_) | MoveOpcode::MoveFrom(_) => {
            // These require runtime support - emit call to storage helper
            buf.push(wasm_op::CALL);
            emit_uleb128_to(buf, 1); // storage_put
        }
        MoveOpcode::Exists(_) | MoveOpcode::BorrowGlobal(_) | MoveOpcode::MutBorrowGlobal(_) => {
            // Map to storage_get
            buf.push(wasm_op::CALL);
            emit_uleb128_to(buf, 0); // storage_get
        }

        // Struct operations
        MoveOpcode::Pack(_) | MoveOpcode::Unpack(_) => {
            // Struct packing/unpacking is handled at compile time
            buf.push(wasm_op::NOP);
        }
        MoveOpcode::BorrowField(_) | MoveOpcode::MutBorrowField(_) => {
            // Field access is memory offset calculation
            buf.push(wasm_op::NOP);
        }

        // Stack
        MoveOpcode::Pop => {
            buf.push(wasm_op::DROP);
        }

        // Vector operations (simplified)
        MoveOpcode::VecPack(_, _)
        | MoveOpcode::VecLen(_)
        | MoveOpcode::VecImmBorrow(_)
        | MoveOpcode::VecMutBorrow(_)
        | MoveOpcode::VecPushBack(_)
        | MoveOpcode::VecPopBack(_) => {
            // Vector ops need runtime support
            buf.push(wasm_op::NOP);
        }

        // Casting
        MoveOpcode::CastU8 => {
            buf.push(wasm_op::I32_WRAP_I64);
        }
        MoveOpcode::CastU64 => {
            // Already i64
            buf.push(wasm_op::NOP);
        }
        MoveOpcode::CastU128 => {
            // Simplified: no-op
            buf.push(wasm_op::NOP);
        }

        MoveOpcode::Nop => {
            buf.push(wasm_op::NOP);
        }
    }

    Ok(())
}

/// Convert TypeTag to WASM type byte
fn type_tag_to_wasm(tag: &TypeTag) -> u8 {
    match tag {
        TypeTag::Bool | TypeTag::U8 => WASM_TYPE_I32,
        TypeTag::U64 | TypeTag::U128 | TypeTag::U256 => WASM_TYPE_I64,
        TypeTag::Address | TypeTag::Signer => WASM_TYPE_I32,
        TypeTag::Vector(_) | TypeTag::Struct(_) => WASM_TYPE_I32,
        TypeTag::Reference(_) | TypeTag::MutableReference(_) => WASM_TYPE_I32,
    }
}

/// Helper: emit ULEB128 to buffer
fn emit_uleb128_to(buf: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            buf.push(byte);
            break;
        } else {
            buf.push(byte | 0x80);
        }
    }
}

/// Helper: emit SLEB128 to buffer
fn emit_sleb128_to(buf: &mut Vec<u8>, mut value: i64) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        let more = !((value == 0 && byte & 0x40 == 0) || (value == -1 && byte & 0x40 != 0));
        if more {
            buf.push(byte | 0x80);
        } else {
            buf.push(byte);
            break;
        }
    }
}

/// Helper: emit string to buffer
fn emit_string_to(buf: &mut Vec<u8>, s: &str) {
    emit_uleb128_to(buf, s.len() as u64);
    buf.extend_from_slice(s.as_bytes());
}

/// Opcode mapping table for Move -> NeoVM
///
/// This table shows the conceptual mapping between Move and NeoVM operations.
/// Actual implementation requires careful handling of type semantics.
pub const OPCODE_MAP: &[(&str, &str)] = &[
    // Arithmetic
    ("Add", "ADD"),
    ("Sub", "SUB"),
    ("Mul", "MUL"),
    ("Div", "DIV"),
    ("Mod", "MOD"),
    // Comparison
    ("Lt", "LT"),
    ("Gt", "GT"),
    ("Le", "LE"),
    ("Ge", "GE"),
    ("Eq", "EQUAL"),
    ("Neq", "NOTEQUAL"),
    // Logical
    ("And", "AND"),
    ("Or", "OR"),
    ("Not", "NOT"),
    // Control flow
    ("Branch", "JMP_L"),
    ("BrTrue", "JMPIF_L"),
    ("BrFalse", "JMPIFNOT_L"),
    ("Ret", "RET"),
    ("Abort", "ABORT"),
    // Stack
    ("Pop", "DROP"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{BytecodeVersion, MoveModule};

    #[test]
    fn test_translate_empty_module() {
        let module = MoveModule {
            version: BytecodeVersion(6),
            name: "test".to_string(),
            structs: vec![],
            functions: vec![],
        };

        let wasm = translate_to_wasm(&module).unwrap();

        // Check WASM magic
        assert_eq!(&wasm[0..4], &[0x00, 0x61, 0x73, 0x6d]);
        // Check version
        assert_eq!(&wasm[4..8], &[0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_translate_simple_function() {
        let module = MoveModule {
            version: BytecodeVersion(6),
            name: "test".to_string(),
            structs: vec![],
            functions: vec![FunctionDef {
                name: "add".to_string(),
                is_public: true,
                is_entry: false,
                parameters: vec![TypeTag::U64, TypeTag::U64],
                returns: vec![TypeTag::U64],
                code: vec![
                    MoveOpcode::CopyLoc(0),
                    MoveOpcode::CopyLoc(1),
                    MoveOpcode::Add,
                    MoveOpcode::Ret,
                ],
            }],
        };

        let wasm = translate_to_wasm(&module).unwrap();

        // Should produce valid WASM
        assert!(wasm.len() > 8);
        assert_eq!(&wasm[0..4], &[0x00, 0x61, 0x73, 0x6d]);
    }

    #[test]
    fn test_uleb128_encoding() {
        let mut buf = Vec::new();
        emit_uleb128_to(&mut buf, 0);
        assert_eq!(buf, vec![0x00]);

        buf.clear();
        emit_uleb128_to(&mut buf, 127);
        assert_eq!(buf, vec![0x7f]);

        buf.clear();
        emit_uleb128_to(&mut buf, 128);
        assert_eq!(buf, vec![0x80, 0x01]);
    }

    #[test]
    fn test_sleb128_encoding() {
        let mut buf = Vec::new();
        emit_sleb128_to(&mut buf, 0);
        assert_eq!(buf, vec![0x00]);

        buf.clear();
        emit_sleb128_to(&mut buf, -1);
        assert_eq!(buf, vec![0x7f]);

        buf.clear();
        emit_sleb128_to(&mut buf, 64);
        assert_eq!(buf, vec![0xc0, 0x00]);
    }
}

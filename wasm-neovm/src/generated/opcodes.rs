// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

/// NeoVM opcode metadata.
#[derive(Debug, Copy, Clone)]
pub struct OpcodeInfo {
    /// Opcode mnemonic name.
    pub name: &'static str,
    /// Single-byte opcode value.
    pub byte: u8,
    /// Fixed operand size in bytes.
    pub operand_size: u8,
    /// Variable-length operand size prefix in bytes.
    pub operand_size_prefix: u8,
}

pub static OPCODES: &[OpcodeInfo] = &[
    OpcodeInfo { name: "PUSHINT8", byte: 0x00, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSHINT16", byte: 0x01, operand_size: 2, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSHINT32", byte: 0x02, operand_size: 4, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSHINT64", byte: 0x03, operand_size: 8, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSHINT128", byte: 0x04, operand_size: 16, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSHINT256", byte: 0x05, operand_size: 32, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSHT", byte: 0x08, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSHF", byte: 0x09, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSHA", byte: 0x0A, operand_size: 4, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSHNULL", byte: 0x0B, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSHDATA1", byte: 0x0C, operand_size: 0, operand_size_prefix: 1 },
    OpcodeInfo { name: "PUSHDATA2", byte: 0x0D, operand_size: 0, operand_size_prefix: 2 },
    OpcodeInfo { name: "PUSHDATA4", byte: 0x0E, operand_size: 0, operand_size_prefix: 4 },
    OpcodeInfo { name: "PUSHM1", byte: 0x0F, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH0", byte: 0x10, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH1", byte: 0x11, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH2", byte: 0x12, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH3", byte: 0x13, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH4", byte: 0x14, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH5", byte: 0x15, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH6", byte: 0x16, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH7", byte: 0x17, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH8", byte: 0x18, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH9", byte: 0x19, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH10", byte: 0x1A, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH11", byte: 0x1B, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH12", byte: 0x1C, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH13", byte: 0x1D, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH14", byte: 0x1E, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH15", byte: 0x1F, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PUSH16", byte: 0x20, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "NOP", byte: 0x21, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMP", byte: 0x22, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMP_L", byte: 0x23, operand_size: 4, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPIF", byte: 0x24, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPIF_L", byte: 0x25, operand_size: 4, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPIFNOT", byte: 0x26, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPIFNOT_L", byte: 0x27, operand_size: 4, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPEQ", byte: 0x28, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPEQ_L", byte: 0x29, operand_size: 4, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPNE", byte: 0x2A, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPNE_L", byte: 0x2B, operand_size: 4, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPGT", byte: 0x2C, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPGT_L", byte: 0x2D, operand_size: 4, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPGE", byte: 0x2E, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPGE_L", byte: 0x2F, operand_size: 4, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPLT", byte: 0x30, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPLT_L", byte: 0x31, operand_size: 4, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPLE", byte: 0x32, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "JMPLE_L", byte: 0x33, operand_size: 4, operand_size_prefix: 0 },
    OpcodeInfo { name: "CALL", byte: 0x34, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "CALL_L", byte: 0x35, operand_size: 4, operand_size_prefix: 0 },
    OpcodeInfo { name: "CALLA", byte: 0x36, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "CALLT", byte: 0x37, operand_size: 2, operand_size_prefix: 0 },
    OpcodeInfo { name: "ABORT", byte: 0x38, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "ASSERT", byte: 0x39, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "THROW", byte: 0x3A, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "TRY", byte: 0x3B, operand_size: 2, operand_size_prefix: 0 },
    OpcodeInfo { name: "TRY_L", byte: 0x3C, operand_size: 8, operand_size_prefix: 0 },
    OpcodeInfo { name: "ENDTRY", byte: 0x3D, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "ENDTRY_L", byte: 0x3E, operand_size: 4, operand_size_prefix: 0 },
    OpcodeInfo { name: "ENDFINALLY", byte: 0x3F, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "RET", byte: 0x40, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "SYSCALL", byte: 0x41, operand_size: 4, operand_size_prefix: 0 },
    OpcodeInfo { name: "DEPTH", byte: 0x43, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "DROP", byte: 0x45, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "NIP", byte: 0x46, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "XDROP", byte: 0x48, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "CLEAR", byte: 0x49, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "DUP", byte: 0x4A, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "OVER", byte: 0x4B, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PICK", byte: 0x4D, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "TUCK", byte: 0x4E, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "SWAP", byte: 0x50, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "ROT", byte: 0x51, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "ROLL", byte: 0x52, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "REVERSE3", byte: 0x53, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "REVERSE4", byte: 0x54, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "REVERSEN", byte: 0x55, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "INITSSLOT", byte: 0x56, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "INITSLOT", byte: 0x57, operand_size: 2, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDSFLD0", byte: 0x58, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDSFLD1", byte: 0x59, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDSFLD2", byte: 0x5A, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDSFLD3", byte: 0x5B, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDSFLD4", byte: 0x5C, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDSFLD5", byte: 0x5D, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDSFLD6", byte: 0x5E, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDSFLD", byte: 0x5F, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "STSFLD0", byte: 0x60, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STSFLD1", byte: 0x61, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STSFLD2", byte: 0x62, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STSFLD3", byte: 0x63, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STSFLD4", byte: 0x64, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STSFLD5", byte: 0x65, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STSFLD6", byte: 0x66, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STSFLD", byte: 0x67, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDLOC0", byte: 0x68, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDLOC1", byte: 0x69, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDLOC2", byte: 0x6A, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDLOC3", byte: 0x6B, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDLOC4", byte: 0x6C, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDLOC5", byte: 0x6D, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDLOC6", byte: 0x6E, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDLOC", byte: 0x6F, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "STLOC0", byte: 0x70, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STLOC1", byte: 0x71, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STLOC2", byte: 0x72, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STLOC3", byte: 0x73, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STLOC4", byte: 0x74, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STLOC5", byte: 0x75, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STLOC6", byte: 0x76, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STLOC", byte: 0x77, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDARG0", byte: 0x78, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDARG1", byte: 0x79, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDARG2", byte: 0x7A, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDARG3", byte: 0x7B, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDARG4", byte: 0x7C, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDARG5", byte: 0x7D, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDARG6", byte: 0x7E, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LDARG", byte: 0x7F, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "STARG0", byte: 0x80, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STARG1", byte: 0x81, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STARG2", byte: 0x82, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STARG3", byte: 0x83, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STARG4", byte: 0x84, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STARG5", byte: 0x85, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STARG6", byte: 0x86, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "STARG", byte: 0x87, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "NEWBUFFER", byte: 0x88, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "MEMCPY", byte: 0x89, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "CAT", byte: 0x8B, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "SUBSTR", byte: 0x8C, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LEFT", byte: 0x8D, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "RIGHT", byte: 0x8E, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "INVERT", byte: 0x90, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "AND", byte: 0x91, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "OR", byte: 0x92, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "XOR", byte: 0x93, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "EQUAL", byte: 0x97, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "NOTEQUAL", byte: 0x98, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "SIGN", byte: 0x99, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "ABS", byte: 0x9A, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "NEGATE", byte: 0x9B, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "INC", byte: 0x9C, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "DEC", byte: 0x9D, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "ADD", byte: 0x9E, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "SUB", byte: 0x9F, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "MUL", byte: 0xA0, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "DIV", byte: 0xA1, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "MOD", byte: 0xA2, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "POW", byte: 0xA3, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "SQRT", byte: 0xA4, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "MODMUL", byte: 0xA5, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "MODPOW", byte: 0xA6, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "SHL", byte: 0xA8, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "SHR", byte: 0xA9, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "NOT", byte: 0xAA, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "BOOLAND", byte: 0xAB, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "BOOLOR", byte: 0xAC, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "NZ", byte: 0xB1, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "NUMEQUAL", byte: 0xB3, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "NUMNOTEQUAL", byte: 0xB4, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LT", byte: 0xB5, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "LE", byte: 0xB6, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "GT", byte: 0xB7, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "GE", byte: 0xB8, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "MIN", byte: 0xB9, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "MAX", byte: 0xBA, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "WITHIN", byte: 0xBB, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PACKMAP", byte: 0xBE, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PACKSTRUCT", byte: 0xBF, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PACK", byte: 0xC0, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "UNPACK", byte: 0xC1, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "NEWARRAY0", byte: 0xC2, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "NEWARRAY", byte: 0xC3, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "NEWARRAY_T", byte: 0xC4, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "NEWSTRUCT0", byte: 0xC5, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "NEWSTRUCT", byte: 0xC6, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "NEWMAP", byte: 0xC8, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "SIZE", byte: 0xCA, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "HASKEY", byte: 0xCB, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "KEYS", byte: 0xCC, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "VALUES", byte: 0xCD, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "PICKITEM", byte: 0xCE, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "APPEND", byte: 0xCF, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "SETITEM", byte: 0xD0, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "REVERSEITEMS", byte: 0xD1, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "REMOVE", byte: 0xD2, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "CLEARITEMS", byte: 0xD3, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "POPITEM", byte: 0xD4, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "ISNULL", byte: 0xD8, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "ISTYPE", byte: 0xD9, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "CONVERT", byte: 0xDB, operand_size: 1, operand_size_prefix: 0 },
    OpcodeInfo { name: "ABORTMSG", byte: 0xE0, operand_size: 0, operand_size_prefix: 0 },
    OpcodeInfo { name: "ASSERTMSG", byte: 0xE1, operand_size: 0, operand_size_prefix: 0 },
];

/// Byte-value constants for each NeoVM opcode, keyed by mnemonic name.
pub mod op {
    /// `PUSHINT8` opcode byte.
    pub const PUSHINT8: u8 = 0x00;
    /// `PUSHINT16` opcode byte.
    pub const PUSHINT16: u8 = 0x01;
    /// `PUSHINT32` opcode byte.
    pub const PUSHINT32: u8 = 0x02;
    /// `PUSHINT64` opcode byte.
    pub const PUSHINT64: u8 = 0x03;
    /// `PUSHINT128` opcode byte.
    pub const PUSHINT128: u8 = 0x04;
    /// `PUSHINT256` opcode byte.
    pub const PUSHINT256: u8 = 0x05;
    /// `PUSHT` opcode byte.
    pub const PUSHT: u8 = 0x08;
    /// `PUSHF` opcode byte.
    pub const PUSHF: u8 = 0x09;
    /// `PUSHA` opcode byte.
    pub const PUSHA: u8 = 0x0A;
    /// `PUSHNULL` opcode byte.
    pub const PUSHNULL: u8 = 0x0B;
    /// `PUSHDATA1` opcode byte.
    pub const PUSHDATA1: u8 = 0x0C;
    /// `PUSHDATA2` opcode byte.
    pub const PUSHDATA2: u8 = 0x0D;
    /// `PUSHDATA4` opcode byte.
    pub const PUSHDATA4: u8 = 0x0E;
    /// `PUSHM1` opcode byte.
    pub const PUSHM1: u8 = 0x0F;
    /// `PUSH0` opcode byte.
    pub const PUSH0: u8 = 0x10;
    /// `PUSH1` opcode byte.
    pub const PUSH1: u8 = 0x11;
    /// `PUSH2` opcode byte.
    pub const PUSH2: u8 = 0x12;
    /// `PUSH3` opcode byte.
    pub const PUSH3: u8 = 0x13;
    /// `PUSH4` opcode byte.
    pub const PUSH4: u8 = 0x14;
    /// `PUSH5` opcode byte.
    pub const PUSH5: u8 = 0x15;
    /// `PUSH6` opcode byte.
    pub const PUSH6: u8 = 0x16;
    /// `PUSH7` opcode byte.
    pub const PUSH7: u8 = 0x17;
    /// `PUSH8` opcode byte.
    pub const PUSH8: u8 = 0x18;
    /// `PUSH9` opcode byte.
    pub const PUSH9: u8 = 0x19;
    /// `PUSH10` opcode byte.
    pub const PUSH10: u8 = 0x1A;
    /// `PUSH11` opcode byte.
    pub const PUSH11: u8 = 0x1B;
    /// `PUSH12` opcode byte.
    pub const PUSH12: u8 = 0x1C;
    /// `PUSH13` opcode byte.
    pub const PUSH13: u8 = 0x1D;
    /// `PUSH14` opcode byte.
    pub const PUSH14: u8 = 0x1E;
    /// `PUSH15` opcode byte.
    pub const PUSH15: u8 = 0x1F;
    /// `PUSH16` opcode byte.
    pub const PUSH16: u8 = 0x20;
    /// `NOP` opcode byte.
    pub const NOP: u8 = 0x21;
    /// `JMP` opcode byte.
    pub const JMP: u8 = 0x22;
    /// `JMP_L` opcode byte.
    pub const JMP_L: u8 = 0x23;
    /// `JMPIF` opcode byte.
    pub const JMPIF: u8 = 0x24;
    /// `JMPIF_L` opcode byte.
    pub const JMPIF_L: u8 = 0x25;
    /// `JMPIFNOT` opcode byte.
    pub const JMPIFNOT: u8 = 0x26;
    /// `JMPIFNOT_L` opcode byte.
    pub const JMPIFNOT_L: u8 = 0x27;
    /// `JMPEQ` opcode byte.
    pub const JMPEQ: u8 = 0x28;
    /// `JMPEQ_L` opcode byte.
    pub const JMPEQ_L: u8 = 0x29;
    /// `JMPNE` opcode byte.
    pub const JMPNE: u8 = 0x2A;
    /// `JMPNE_L` opcode byte.
    pub const JMPNE_L: u8 = 0x2B;
    /// `JMPGT` opcode byte.
    pub const JMPGT: u8 = 0x2C;
    /// `JMPGT_L` opcode byte.
    pub const JMPGT_L: u8 = 0x2D;
    /// `JMPGE` opcode byte.
    pub const JMPGE: u8 = 0x2E;
    /// `JMPGE_L` opcode byte.
    pub const JMPGE_L: u8 = 0x2F;
    /// `JMPLT` opcode byte.
    pub const JMPLT: u8 = 0x30;
    /// `JMPLT_L` opcode byte.
    pub const JMPLT_L: u8 = 0x31;
    /// `JMPLE` opcode byte.
    pub const JMPLE: u8 = 0x32;
    /// `JMPLE_L` opcode byte.
    pub const JMPLE_L: u8 = 0x33;
    /// `CALL` opcode byte.
    pub const CALL: u8 = 0x34;
    /// `CALL_L` opcode byte.
    pub const CALL_L: u8 = 0x35;
    /// `CALLA` opcode byte.
    pub const CALLA: u8 = 0x36;
    /// `CALLT` opcode byte.
    pub const CALLT: u8 = 0x37;
    /// `ABORT` opcode byte.
    pub const ABORT: u8 = 0x38;
    /// `ASSERT` opcode byte.
    pub const ASSERT: u8 = 0x39;
    /// `THROW` opcode byte.
    pub const THROW: u8 = 0x3A;
    /// `TRY` opcode byte.
    pub const TRY: u8 = 0x3B;
    /// `TRY_L` opcode byte.
    pub const TRY_L: u8 = 0x3C;
    /// `ENDTRY` opcode byte.
    pub const ENDTRY: u8 = 0x3D;
    /// `ENDTRY_L` opcode byte.
    pub const ENDTRY_L: u8 = 0x3E;
    /// `ENDFINALLY` opcode byte.
    pub const ENDFINALLY: u8 = 0x3F;
    /// `RET` opcode byte.
    pub const RET: u8 = 0x40;
    /// `SYSCALL` opcode byte.
    pub const SYSCALL: u8 = 0x41;
    /// `DEPTH` opcode byte.
    pub const DEPTH: u8 = 0x43;
    /// `DROP` opcode byte.
    pub const DROP: u8 = 0x45;
    /// `NIP` opcode byte.
    pub const NIP: u8 = 0x46;
    /// `XDROP` opcode byte.
    pub const XDROP: u8 = 0x48;
    /// `CLEAR` opcode byte.
    pub const CLEAR: u8 = 0x49;
    /// `DUP` opcode byte.
    pub const DUP: u8 = 0x4A;
    /// `OVER` opcode byte.
    pub const OVER: u8 = 0x4B;
    /// `PICK` opcode byte.
    pub const PICK: u8 = 0x4D;
    /// `TUCK` opcode byte.
    pub const TUCK: u8 = 0x4E;
    /// `SWAP` opcode byte.
    pub const SWAP: u8 = 0x50;
    /// `ROT` opcode byte.
    pub const ROT: u8 = 0x51;
    /// `ROLL` opcode byte.
    pub const ROLL: u8 = 0x52;
    /// `REVERSE3` opcode byte.
    pub const REVERSE3: u8 = 0x53;
    /// `REVERSE4` opcode byte.
    pub const REVERSE4: u8 = 0x54;
    /// `REVERSEN` opcode byte.
    pub const REVERSEN: u8 = 0x55;
    /// `INITSSLOT` opcode byte.
    pub const INITSSLOT: u8 = 0x56;
    /// `INITSLOT` opcode byte.
    pub const INITSLOT: u8 = 0x57;
    /// `LDSFLD0` opcode byte.
    pub const LDSFLD0: u8 = 0x58;
    /// `LDSFLD1` opcode byte.
    pub const LDSFLD1: u8 = 0x59;
    /// `LDSFLD2` opcode byte.
    pub const LDSFLD2: u8 = 0x5A;
    /// `LDSFLD3` opcode byte.
    pub const LDSFLD3: u8 = 0x5B;
    /// `LDSFLD4` opcode byte.
    pub const LDSFLD4: u8 = 0x5C;
    /// `LDSFLD5` opcode byte.
    pub const LDSFLD5: u8 = 0x5D;
    /// `LDSFLD6` opcode byte.
    pub const LDSFLD6: u8 = 0x5E;
    /// `LDSFLD` opcode byte.
    pub const LDSFLD: u8 = 0x5F;
    /// `STSFLD0` opcode byte.
    pub const STSFLD0: u8 = 0x60;
    /// `STSFLD1` opcode byte.
    pub const STSFLD1: u8 = 0x61;
    /// `STSFLD2` opcode byte.
    pub const STSFLD2: u8 = 0x62;
    /// `STSFLD3` opcode byte.
    pub const STSFLD3: u8 = 0x63;
    /// `STSFLD4` opcode byte.
    pub const STSFLD4: u8 = 0x64;
    /// `STSFLD5` opcode byte.
    pub const STSFLD5: u8 = 0x65;
    /// `STSFLD6` opcode byte.
    pub const STSFLD6: u8 = 0x66;
    /// `STSFLD` opcode byte.
    pub const STSFLD: u8 = 0x67;
    /// `LDLOC0` opcode byte.
    pub const LDLOC0: u8 = 0x68;
    /// `LDLOC1` opcode byte.
    pub const LDLOC1: u8 = 0x69;
    /// `LDLOC2` opcode byte.
    pub const LDLOC2: u8 = 0x6A;
    /// `LDLOC3` opcode byte.
    pub const LDLOC3: u8 = 0x6B;
    /// `LDLOC4` opcode byte.
    pub const LDLOC4: u8 = 0x6C;
    /// `LDLOC5` opcode byte.
    pub const LDLOC5: u8 = 0x6D;
    /// `LDLOC6` opcode byte.
    pub const LDLOC6: u8 = 0x6E;
    /// `LDLOC` opcode byte.
    pub const LDLOC: u8 = 0x6F;
    /// `STLOC0` opcode byte.
    pub const STLOC0: u8 = 0x70;
    /// `STLOC1` opcode byte.
    pub const STLOC1: u8 = 0x71;
    /// `STLOC2` opcode byte.
    pub const STLOC2: u8 = 0x72;
    /// `STLOC3` opcode byte.
    pub const STLOC3: u8 = 0x73;
    /// `STLOC4` opcode byte.
    pub const STLOC4: u8 = 0x74;
    /// `STLOC5` opcode byte.
    pub const STLOC5: u8 = 0x75;
    /// `STLOC6` opcode byte.
    pub const STLOC6: u8 = 0x76;
    /// `STLOC` opcode byte.
    pub const STLOC: u8 = 0x77;
    /// `LDARG0` opcode byte.
    pub const LDARG0: u8 = 0x78;
    /// `LDARG1` opcode byte.
    pub const LDARG1: u8 = 0x79;
    /// `LDARG2` opcode byte.
    pub const LDARG2: u8 = 0x7A;
    /// `LDARG3` opcode byte.
    pub const LDARG3: u8 = 0x7B;
    /// `LDARG4` opcode byte.
    pub const LDARG4: u8 = 0x7C;
    /// `LDARG5` opcode byte.
    pub const LDARG5: u8 = 0x7D;
    /// `LDARG6` opcode byte.
    pub const LDARG6: u8 = 0x7E;
    /// `LDARG` opcode byte.
    pub const LDARG: u8 = 0x7F;
    /// `STARG0` opcode byte.
    pub const STARG0: u8 = 0x80;
    /// `STARG1` opcode byte.
    pub const STARG1: u8 = 0x81;
    /// `STARG2` opcode byte.
    pub const STARG2: u8 = 0x82;
    /// `STARG3` opcode byte.
    pub const STARG3: u8 = 0x83;
    /// `STARG4` opcode byte.
    pub const STARG4: u8 = 0x84;
    /// `STARG5` opcode byte.
    pub const STARG5: u8 = 0x85;
    /// `STARG6` opcode byte.
    pub const STARG6: u8 = 0x86;
    /// `STARG` opcode byte.
    pub const STARG: u8 = 0x87;
    /// `NEWBUFFER` opcode byte.
    pub const NEWBUFFER: u8 = 0x88;
    /// `MEMCPY` opcode byte.
    pub const MEMCPY: u8 = 0x89;
    /// `CAT` opcode byte.
    pub const CAT: u8 = 0x8B;
    /// `SUBSTR` opcode byte.
    pub const SUBSTR: u8 = 0x8C;
    /// `LEFT` opcode byte.
    pub const LEFT: u8 = 0x8D;
    /// `RIGHT` opcode byte.
    pub const RIGHT: u8 = 0x8E;
    /// `INVERT` opcode byte.
    pub const INVERT: u8 = 0x90;
    /// `AND` opcode byte.
    pub const AND: u8 = 0x91;
    /// `OR` opcode byte.
    pub const OR: u8 = 0x92;
    /// `XOR` opcode byte.
    pub const XOR: u8 = 0x93;
    /// `EQUAL` opcode byte.
    pub const EQUAL: u8 = 0x97;
    /// `NOTEQUAL` opcode byte.
    pub const NOTEQUAL: u8 = 0x98;
    /// `SIGN` opcode byte.
    pub const SIGN: u8 = 0x99;
    /// `ABS` opcode byte.
    pub const ABS: u8 = 0x9A;
    /// `NEGATE` opcode byte.
    pub const NEGATE: u8 = 0x9B;
    /// `INC` opcode byte.
    pub const INC: u8 = 0x9C;
    /// `DEC` opcode byte.
    pub const DEC: u8 = 0x9D;
    /// `ADD` opcode byte.
    pub const ADD: u8 = 0x9E;
    /// `SUB` opcode byte.
    pub const SUB: u8 = 0x9F;
    /// `MUL` opcode byte.
    pub const MUL: u8 = 0xA0;
    /// `DIV` opcode byte.
    pub const DIV: u8 = 0xA1;
    /// `MOD` opcode byte.
    pub const MOD: u8 = 0xA2;
    /// `POW` opcode byte.
    pub const POW: u8 = 0xA3;
    /// `SQRT` opcode byte.
    pub const SQRT: u8 = 0xA4;
    /// `MODMUL` opcode byte.
    pub const MODMUL: u8 = 0xA5;
    /// `MODPOW` opcode byte.
    pub const MODPOW: u8 = 0xA6;
    /// `SHL` opcode byte.
    pub const SHL: u8 = 0xA8;
    /// `SHR` opcode byte.
    pub const SHR: u8 = 0xA9;
    /// `NOT` opcode byte.
    pub const NOT: u8 = 0xAA;
    /// `BOOLAND` opcode byte.
    pub const BOOLAND: u8 = 0xAB;
    /// `BOOLOR` opcode byte.
    pub const BOOLOR: u8 = 0xAC;
    /// `NZ` opcode byte.
    pub const NZ: u8 = 0xB1;
    /// `NUMEQUAL` opcode byte.
    pub const NUMEQUAL: u8 = 0xB3;
    /// `NUMNOTEQUAL` opcode byte.
    pub const NUMNOTEQUAL: u8 = 0xB4;
    /// `LT` opcode byte.
    pub const LT: u8 = 0xB5;
    /// `LE` opcode byte.
    pub const LE: u8 = 0xB6;
    /// `GT` opcode byte.
    pub const GT: u8 = 0xB7;
    /// `GE` opcode byte.
    pub const GE: u8 = 0xB8;
    /// `MIN` opcode byte.
    pub const MIN: u8 = 0xB9;
    /// `MAX` opcode byte.
    pub const MAX: u8 = 0xBA;
    /// `WITHIN` opcode byte.
    pub const WITHIN: u8 = 0xBB;
    /// `PACKMAP` opcode byte.
    pub const PACKMAP: u8 = 0xBE;
    /// `PACKSTRUCT` opcode byte.
    pub const PACKSTRUCT: u8 = 0xBF;
    /// `PACK` opcode byte.
    pub const PACK: u8 = 0xC0;
    /// `UNPACK` opcode byte.
    pub const UNPACK: u8 = 0xC1;
    /// `NEWARRAY0` opcode byte.
    pub const NEWARRAY0: u8 = 0xC2;
    /// `NEWARRAY` opcode byte.
    pub const NEWARRAY: u8 = 0xC3;
    /// `NEWARRAY_T` opcode byte.
    pub const NEWARRAY_T: u8 = 0xC4;
    /// `NEWSTRUCT0` opcode byte.
    pub const NEWSTRUCT0: u8 = 0xC5;
    /// `NEWSTRUCT` opcode byte.
    pub const NEWSTRUCT: u8 = 0xC6;
    /// `NEWMAP` opcode byte.
    pub const NEWMAP: u8 = 0xC8;
    /// `SIZE` opcode byte.
    pub const SIZE: u8 = 0xCA;
    /// `HASKEY` opcode byte.
    pub const HASKEY: u8 = 0xCB;
    /// `KEYS` opcode byte.
    pub const KEYS: u8 = 0xCC;
    /// `VALUES` opcode byte.
    pub const VALUES: u8 = 0xCD;
    /// `PICKITEM` opcode byte.
    pub const PICKITEM: u8 = 0xCE;
    /// `APPEND` opcode byte.
    pub const APPEND: u8 = 0xCF;
    /// `SETITEM` opcode byte.
    pub const SETITEM: u8 = 0xD0;
    /// `REVERSEITEMS` opcode byte.
    pub const REVERSEITEMS: u8 = 0xD1;
    /// `REMOVE` opcode byte.
    pub const REMOVE: u8 = 0xD2;
    /// `CLEARITEMS` opcode byte.
    pub const CLEARITEMS: u8 = 0xD3;
    /// `POPITEM` opcode byte.
    pub const POPITEM: u8 = 0xD4;
    /// `ISNULL` opcode byte.
    pub const ISNULL: u8 = 0xD8;
    /// `ISTYPE` opcode byte.
    pub const ISTYPE: u8 = 0xD9;
    /// `CONVERT` opcode byte.
    pub const CONVERT: u8 = 0xDB;
    /// `ABORTMSG` opcode byte.
    pub const ABORTMSG: u8 = 0xE0;
    /// `ASSERTMSG` opcode byte.
    pub const ASSERTMSG: u8 = 0xE1;

    /// Every opcode paired with its byte value, mirroring [`super::OPCODES`].
    pub static ALL: &[(&str, u8)] = &[
        ("PUSHINT8", PUSHINT8),
        ("PUSHINT16", PUSHINT16),
        ("PUSHINT32", PUSHINT32),
        ("PUSHINT64", PUSHINT64),
        ("PUSHINT128", PUSHINT128),
        ("PUSHINT256", PUSHINT256),
        ("PUSHT", PUSHT),
        ("PUSHF", PUSHF),
        ("PUSHA", PUSHA),
        ("PUSHNULL", PUSHNULL),
        ("PUSHDATA1", PUSHDATA1),
        ("PUSHDATA2", PUSHDATA2),
        ("PUSHDATA4", PUSHDATA4),
        ("PUSHM1", PUSHM1),
        ("PUSH0", PUSH0),
        ("PUSH1", PUSH1),
        ("PUSH2", PUSH2),
        ("PUSH3", PUSH3),
        ("PUSH4", PUSH4),
        ("PUSH5", PUSH5),
        ("PUSH6", PUSH6),
        ("PUSH7", PUSH7),
        ("PUSH8", PUSH8),
        ("PUSH9", PUSH9),
        ("PUSH10", PUSH10),
        ("PUSH11", PUSH11),
        ("PUSH12", PUSH12),
        ("PUSH13", PUSH13),
        ("PUSH14", PUSH14),
        ("PUSH15", PUSH15),
        ("PUSH16", PUSH16),
        ("NOP", NOP),
        ("JMP", JMP),
        ("JMP_L", JMP_L),
        ("JMPIF", JMPIF),
        ("JMPIF_L", JMPIF_L),
        ("JMPIFNOT", JMPIFNOT),
        ("JMPIFNOT_L", JMPIFNOT_L),
        ("JMPEQ", JMPEQ),
        ("JMPEQ_L", JMPEQ_L),
        ("JMPNE", JMPNE),
        ("JMPNE_L", JMPNE_L),
        ("JMPGT", JMPGT),
        ("JMPGT_L", JMPGT_L),
        ("JMPGE", JMPGE),
        ("JMPGE_L", JMPGE_L),
        ("JMPLT", JMPLT),
        ("JMPLT_L", JMPLT_L),
        ("JMPLE", JMPLE),
        ("JMPLE_L", JMPLE_L),
        ("CALL", CALL),
        ("CALL_L", CALL_L),
        ("CALLA", CALLA),
        ("CALLT", CALLT),
        ("ABORT", ABORT),
        ("ASSERT", ASSERT),
        ("THROW", THROW),
        ("TRY", TRY),
        ("TRY_L", TRY_L),
        ("ENDTRY", ENDTRY),
        ("ENDTRY_L", ENDTRY_L),
        ("ENDFINALLY", ENDFINALLY),
        ("RET", RET),
        ("SYSCALL", SYSCALL),
        ("DEPTH", DEPTH),
        ("DROP", DROP),
        ("NIP", NIP),
        ("XDROP", XDROP),
        ("CLEAR", CLEAR),
        ("DUP", DUP),
        ("OVER", OVER),
        ("PICK", PICK),
        ("TUCK", TUCK),
        ("SWAP", SWAP),
        ("ROT", ROT),
        ("ROLL", ROLL),
        ("REVERSE3", REVERSE3),
        ("REVERSE4", REVERSE4),
        ("REVERSEN", REVERSEN),
        ("INITSSLOT", INITSSLOT),
        ("INITSLOT", INITSLOT),
        ("LDSFLD0", LDSFLD0),
        ("LDSFLD1", LDSFLD1),
        ("LDSFLD2", LDSFLD2),
        ("LDSFLD3", LDSFLD3),
        ("LDSFLD4", LDSFLD4),
        ("LDSFLD5", LDSFLD5),
        ("LDSFLD6", LDSFLD6),
        ("LDSFLD", LDSFLD),
        ("STSFLD0", STSFLD0),
        ("STSFLD1", STSFLD1),
        ("STSFLD2", STSFLD2),
        ("STSFLD3", STSFLD3),
        ("STSFLD4", STSFLD4),
        ("STSFLD5", STSFLD5),
        ("STSFLD6", STSFLD6),
        ("STSFLD", STSFLD),
        ("LDLOC0", LDLOC0),
        ("LDLOC1", LDLOC1),
        ("LDLOC2", LDLOC2),
        ("LDLOC3", LDLOC3),
        ("LDLOC4", LDLOC4),
        ("LDLOC5", LDLOC5),
        ("LDLOC6", LDLOC6),
        ("LDLOC", LDLOC),
        ("STLOC0", STLOC0),
        ("STLOC1", STLOC1),
        ("STLOC2", STLOC2),
        ("STLOC3", STLOC3),
        ("STLOC4", STLOC4),
        ("STLOC5", STLOC5),
        ("STLOC6", STLOC6),
        ("STLOC", STLOC),
        ("LDARG0", LDARG0),
        ("LDARG1", LDARG1),
        ("LDARG2", LDARG2),
        ("LDARG3", LDARG3),
        ("LDARG4", LDARG4),
        ("LDARG5", LDARG5),
        ("LDARG6", LDARG6),
        ("LDARG", LDARG),
        ("STARG0", STARG0),
        ("STARG1", STARG1),
        ("STARG2", STARG2),
        ("STARG3", STARG3),
        ("STARG4", STARG4),
        ("STARG5", STARG5),
        ("STARG6", STARG6),
        ("STARG", STARG),
        ("NEWBUFFER", NEWBUFFER),
        ("MEMCPY", MEMCPY),
        ("CAT", CAT),
        ("SUBSTR", SUBSTR),
        ("LEFT", LEFT),
        ("RIGHT", RIGHT),
        ("INVERT", INVERT),
        ("AND", AND),
        ("OR", OR),
        ("XOR", XOR),
        ("EQUAL", EQUAL),
        ("NOTEQUAL", NOTEQUAL),
        ("SIGN", SIGN),
        ("ABS", ABS),
        ("NEGATE", NEGATE),
        ("INC", INC),
        ("DEC", DEC),
        ("ADD", ADD),
        ("SUB", SUB),
        ("MUL", MUL),
        ("DIV", DIV),
        ("MOD", MOD),
        ("POW", POW),
        ("SQRT", SQRT),
        ("MODMUL", MODMUL),
        ("MODPOW", MODPOW),
        ("SHL", SHL),
        ("SHR", SHR),
        ("NOT", NOT),
        ("BOOLAND", BOOLAND),
        ("BOOLOR", BOOLOR),
        ("NZ", NZ),
        ("NUMEQUAL", NUMEQUAL),
        ("NUMNOTEQUAL", NUMNOTEQUAL),
        ("LT", LT),
        ("LE", LE),
        ("GT", GT),
        ("GE", GE),
        ("MIN", MIN),
        ("MAX", MAX),
        ("WITHIN", WITHIN),
        ("PACKMAP", PACKMAP),
        ("PACKSTRUCT", PACKSTRUCT),
        ("PACK", PACK),
        ("UNPACK", UNPACK),
        ("NEWARRAY0", NEWARRAY0),
        ("NEWARRAY", NEWARRAY),
        ("NEWARRAY_T", NEWARRAY_T),
        ("NEWSTRUCT0", NEWSTRUCT0),
        ("NEWSTRUCT", NEWSTRUCT),
        ("NEWMAP", NEWMAP),
        ("SIZE", SIZE),
        ("HASKEY", HASKEY),
        ("KEYS", KEYS),
        ("VALUES", VALUES),
        ("PICKITEM", PICKITEM),
        ("APPEND", APPEND),
        ("SETITEM", SETITEM),
        ("REVERSEITEMS", REVERSEITEMS),
        ("REMOVE", REMOVE),
        ("CLEARITEMS", CLEARITEMS),
        ("POPITEM", POPITEM),
        ("ISNULL", ISNULL),
        ("ISTYPE", ISTYPE),
        ("CONVERT", CONVERT),
        ("ABORTMSG", ABORTMSG),
        ("ASSERTMSG", ASSERTMSG),
    ];
}

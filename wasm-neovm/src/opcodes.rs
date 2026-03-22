// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use once_cell::sync::Lazy;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/opcodes.rs"));
}

pub use generated::OpcodeInfo;

/// Fast O(1) opcode lookup table initialized lazily
static OPCODE_LOOKUP: Lazy<std::collections::HashMap<&'static str, &'static OpcodeInfo>> =
    Lazy::new(|| {
        let mut map = std::collections::HashMap::with_capacity(generated::OPCODES.len());
        for op in generated::OPCODES {
            map.insert(op.name, op);
        }
        map
    });

/// Fast O(1) opcode lookup by byte value.
static OPCODE_BYTE_LOOKUP: Lazy<[Option<&'static OpcodeInfo>; 256]> = Lazy::new(|| {
    let mut table = [None; 256];
    for op in generated::OPCODES {
        table[op.byte as usize] = Some(op);
    }
    table
});

/// Return all NeoVM opcodes.
pub fn all() -> &'static [OpcodeInfo] {
    generated::OPCODES
}

/// Fast O(1) opcode lookup using pre-built hash map (Round 61, 63, 66 optimizations)
pub fn lookup(name: &str) -> Option<&'static OpcodeInfo> {
    // Fast path: exact match (most common case)
    if let Some(op) = OPCODE_LOOKUP.get(name) {
        return Some(*op);
    }
    // Slow path: case-insensitive fallback for compatibility
    let upper_name = name.to_ascii_uppercase();
    OPCODE_LOOKUP.get(upper_name.as_str()).copied()
}

/// Look up an opcode by its byte value.
pub fn lookup_by_byte(byte: u8) -> Option<&'static OpcodeInfo> {
    OPCODE_BYTE_LOOKUP[byte as usize]
}

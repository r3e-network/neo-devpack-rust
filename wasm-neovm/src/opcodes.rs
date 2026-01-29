use std::sync::LazyLock;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/opcodes.rs"));
}

pub use generated::OpcodeInfo;

/// Fast O(1) opcode lookup table initialized lazily
static OPCODE_LOOKUP: LazyLock<std::collections::HashMap<&'static str, &'static OpcodeInfo>> =
    LazyLock::new(|| {
        let mut map = std::collections::HashMap::with_capacity(generated::OPCODES.len());
        for op in generated::OPCODES {
            map.insert(op.name, op);
        }
        map
    });

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

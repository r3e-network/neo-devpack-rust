mod generated {
    include!(concat!(env!("OUT_DIR"), "/opcodes.rs"));
}

pub use generated::OpcodeInfo;

pub fn all() -> &'static [OpcodeInfo] {
    generated::OPCODES
}

pub fn lookup(name: &str) -> Option<&'static OpcodeInfo> {
    generated::OPCODES
        .iter()
        .find(|op| op.name.eq_ignore_ascii_case(name))
}

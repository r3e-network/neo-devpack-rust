use anyhow::{anyhow, Result};

use crate::opcodes;

pub(crate) fn lookup_opcode(name: &str) -> Result<&'static opcodes::OpcodeInfo> {
    opcodes::lookup(name).ok_or_else(|| anyhow!("unknown NeoVM opcode '{}'", name))
}

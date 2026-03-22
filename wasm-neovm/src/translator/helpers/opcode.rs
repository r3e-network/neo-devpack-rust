// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use anyhow::{anyhow, Result};

use crate::opcodes;

/// Lookup NeoVM opcode by name (Round 81 - Inline small hot function)
///
/// This function is called extremely frequently during translation
/// and is marked `#[inline]` to avoid call overhead in hot paths.
#[inline(always)]
pub(crate) fn lookup_opcode(name: &str) -> Result<&'static opcodes::OpcodeInfo> {
    opcodes::lookup(name).ok_or_else(|| anyhow!("unknown NeoVM opcode '{}'", name))
}

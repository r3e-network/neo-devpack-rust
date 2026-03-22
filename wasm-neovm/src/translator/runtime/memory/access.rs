// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::*;

pub(crate) fn ensure_memory_access(runtime: &RuntimeHelpers, mem_index: u32) -> Result<()> {
    if mem_index != 0 {
        bail!(
            "only default memory index 0 is supported (NeoVM exposes a single linear memory; see docs/wasm-pipeline.md#9-unsupported-wasm-features)"
        );
    }
    if !runtime.memory_defined() {
        bail!("memory instructions require a defined memory section");
    }
    Ok(())
}

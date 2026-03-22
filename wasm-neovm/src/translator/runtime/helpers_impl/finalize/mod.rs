// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::*;

mod init_helper;
mod passive_data;
mod realize;

/// Bundled parameters for the `RuntimeHelpers::finalize` method.
pub(crate) struct FinalizeParams<'a> {
    pub script: &'a mut Vec<u8>,
    pub start: Option<&'a StartDescriptor>,
    pub imports: &'a [FunctionImport],
    pub types: &'a [FuncType],
    pub func_type_indices: &'a [u32],
    pub functions: Option<&'a mut FunctionRegistry>,
    pub features: &'a mut FeatureTracker,
    pub adapter: &'a dyn ChainAdapter,
}

impl RuntimeHelpers {
    pub(crate) fn emit_memory_init_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        if self.memory_init_suppressed {
            return Ok(());
        }
        emit_load_static(script, INIT_FLAG_SLOT)?;
        let skip_call = emit_jump_placeholder(script, "JMPIF_L")?;
        let call_pos = emit_call_placeholder(script)?;
        let after_call = script.len();
        patch_jump(script, skip_call, after_call)?;
        self.memory_init_calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn set_memory_init_suppressed(&mut self, suppressed: bool) -> bool {
        let previous = self.memory_init_suppressed;
        self.memory_init_suppressed = suppressed;
        previous
    }

    pub(crate) fn finalize(&mut self, params: FinalizeParams<'_>) -> Result<()> {
        self.prepare_init_helper(
            params.script,
            params.start,
            params.imports,
            params.types,
            params.adapter,
        )?;
        self.realize_helper_calls(
            params.script,
            params.imports,
            params.types,
            params.func_type_indices,
            params.functions,
            params.features,
            params.adapter,
        )?;
        self.emit_passive_data_helpers(params.script)?;
        Ok(())
    }

    pub(crate) fn patch_start_calls(&self, script: &mut [u8], target: usize) -> Result<()> {
        for &pos in &self.start_call_positions {
            patch_call(script, pos, target)?;
        }
        Ok(())
    }

    pub(crate) fn start_slot(&self) -> Option<usize> {
        self.start_slot
    }

    pub(crate) fn memory_init_offset(&self) -> Option<usize> {
        self.memory_init_offset
    }
}

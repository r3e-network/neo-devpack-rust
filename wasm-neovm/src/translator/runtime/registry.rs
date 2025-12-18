use anyhow::{anyhow, bail, Result};

use crate::translator::helpers::{emit_call_placeholder, patch_call};

pub(crate) struct FunctionRegistry {
    offsets: Vec<Option<usize>>,
    fixups: Vec<Vec<usize>>,
}

impl FunctionRegistry {
    pub(crate) fn new(total_functions: usize) -> Self {
        FunctionRegistry {
            offsets: vec![None; total_functions],
            fixups: vec![Vec::new(); total_functions],
        }
    }

    pub(crate) fn register_offset(
        &mut self,
        script: &mut [u8],
        function_index: usize,
        offset: usize,
    ) -> Result<()> {
        let entry = self
            .offsets
            .get_mut(function_index)
            .ok_or_else(|| anyhow!("function index {} out of range", function_index))?;
        if entry.is_some() {
            bail!(
                "function index {} registered multiple times",
                function_index
            );
        }
        *entry = Some(offset);

        if let Some(pending) = self.fixups.get_mut(function_index) {
            for call_pos in pending.drain(..) {
                patch_call(script, call_pos, offset)?;
            }
        }
        Ok(())
    }

    pub(crate) fn contains_index(&self, function_index: usize) -> bool {
        function_index < self.offsets.len()
    }

    pub(crate) fn emit_call(&mut self, script: &mut Vec<u8>, function_index: usize) -> Result<()> {
        if self.offsets.get(function_index).is_none() {
            bail!("function index {} out of range", function_index);
        }
        let call_pos = emit_call_placeholder(script)?;
        if let Some(offset) = self.offsets[function_index] {
            patch_call(script, call_pos, offset)?;
        } else {
            self.fixups[function_index].push(call_pos);
        }
        Ok(())
    }
}

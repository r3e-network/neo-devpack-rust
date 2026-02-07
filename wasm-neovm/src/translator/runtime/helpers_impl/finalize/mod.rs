use super::super::*;

mod init_helper;
mod passive_data;
mod realize;

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

    pub(crate) fn finalize(
        &mut self,
        script: &mut Vec<u8>,
        start: Option<&StartDescriptor>,
        imports: &[FunctionImport],
        types: &[FuncType],
        func_type_indices: &[u32],
        functions: Option<&mut FunctionRegistry>,
        features: &mut FeatureTracker,
        adapter: &dyn ChainAdapter,
    ) -> Result<()> {
        self.prepare_init_helper(script, start, imports, types, adapter)?;
        self.realize_helper_calls(
            script,
            imports,
            types,
            func_type_indices,
            functions,
            features,
            adapter,
        )?;
        self.emit_passive_data_helpers(script)?;
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

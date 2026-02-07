use super::super::*;

impl RuntimeHelpers {
    pub(crate) fn helper_record_mut(&mut self, kind: MemoryHelperKind) -> &mut HelperRecord {
        self.memory_helpers.entry(kind).or_default()
    }

    pub(crate) fn bit_helper_record_mut(&mut self, kind: BitHelperKind) -> &mut HelperRecord {
        self.bit_helpers.entry(kind).or_default()
    }

    pub(crate) fn table_helper_record_mut(&mut self, kind: TableHelperKind) -> &mut HelperRecord {
        self.table_helpers.entry(kind).or_default()
    }

    pub(crate) fn call_indirect_helper_record_mut(
        &mut self,
        key: CallIndirectHelperKey,
    ) -> &mut HelperRecord {
        self.call_indirect_helpers.entry(key).or_default()
    }

    pub(crate) fn emit_bit_helper(
        &mut self,
        script: &mut Vec<u8>,
        kind: BitHelperKind,
    ) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.bit_helper_record_mut(kind);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_table_helper(
        &mut self,
        script: &mut Vec<u8>,
        kind: TableHelperKind,
    ) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.table_helper_record_mut(kind);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_call_indirect_helper(
        &mut self,
        script: &mut Vec<u8>,
        table_index: usize,
        type_index: u32,
    ) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.call_indirect_helper_record_mut(CallIndirectHelperKey {
            table_index,
            type_index,
        });
        record.calls.push(call_pos);
        Ok(())
    }
}

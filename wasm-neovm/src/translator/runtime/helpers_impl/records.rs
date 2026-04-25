// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

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

    pub(crate) fn storage_helper_record_mut(
        &mut self,
        kind: StorageHelperKind,
    ) -> &mut HelperRecord {
        self.storage_helpers.entry(kind).or_default()
    }

    /// Emit a CALL placeholder for a `System.Storage.*` marshalling helper.
    pub(crate) fn emit_storage_helper(
        &mut self,
        script: &mut Vec<u8>,
        kind: StorageHelperKind,
    ) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.storage_helper_record_mut(kind);
        record.calls.push(call_pos);
        Ok(())
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

    /// Emit a CALL placeholder for the shared i32 sign-extension helper.
    /// The top-of-stack value will be sign-extended in place.
    pub(crate) fn emit_sign_extend_32_helper(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        self.sign_extend_32_helper.calls.push(call_pos);
        Ok(())
    }

    /// Emit a CALL placeholder for the shared i64 sign-extension helper.
    pub(crate) fn emit_sign_extend_64_helper(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        self.sign_extend_64_helper.calls.push(call_pos);
        Ok(())
    }

    /// Emit a CALL placeholder for the shared i32 param normalization helper.
    /// Top-of-stack value is normalized (null→0, ByteString→Integer, sign-extended).
    pub(crate) fn emit_param_normalize_i32_helper(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        self.param_normalize_i32_helper.calls.push(call_pos);
        Ok(())
    }

    /// Emit a CALL placeholder for the shared i64 param normalization helper.
    pub(crate) fn emit_param_normalize_i64_helper(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        self.param_normalize_i64_helper.calls.push(call_pos);
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

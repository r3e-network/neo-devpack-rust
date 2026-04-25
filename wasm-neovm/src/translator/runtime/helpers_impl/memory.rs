// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::*;

impl RuntimeHelpers {
    pub(crate) fn set_memory_config(
        &mut self,
        initial_pages: u32,
        maximum_pages: Option<u32>,
    ) -> Result<()> {
        if self.memory_defined {
            bail!(
                "multiple memories are not supported (NeoVM exposes a single linear memory; see docs/wasm-pipeline.md#9-unsupported-wasm-features)"
            );
        }
        if matches!(maximum_pages, Some(max) if max < initial_pages) {
            bail!("memory maximum smaller than initial size is invalid");
        }
        self.memory_config = MemoryConfig {
            initial_pages,
            maximum_pages,
        };
        self.memory_defined = true;
        Ok(())
    }

    pub(crate) fn memory_defined(&self) -> bool {
        self.memory_defined
    }

    pub(crate) fn uses_chunked_memory(&self) -> bool {
        self.memory_defined
            && (self.memory_config.initial_pages > 1
                || self.memory_helpers.contains_key(&MemoryHelperKind::Grow))
    }

    pub(crate) fn runtime_state_requires_entry_init(&self) -> bool {
        !self.tables.is_empty() || !self.element_segments.is_empty()
    }

    pub(crate) fn active_data_slice(&self, offset: usize, len: usize) -> Option<&[u8]> {
        for segment in &self.data_segments {
            let DataSegmentKind::Active {
                offset: segment_offset,
            } = segment.kind
            else {
                continue;
            };
            let segment_start = segment_offset as usize;
            let segment_end = segment_start.checked_add(segment.bytes.len())?;
            let requested_end = offset.checked_add(len)?;
            if offset >= segment_start && requested_end <= segment_end {
                let relative = offset - segment_start;
                return Some(&segment.bytes[relative..relative + len]);
            }
        }
        None
    }

    pub(crate) fn emit_memory_load_call(&mut self, script: &mut Vec<u8>, bytes: u32) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::Load(bytes));
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_memory_store_call(
        &mut self,
        script: &mut Vec<u8>,
        bytes: u32,
    ) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::Store(bytes));
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_memory_grow_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::Grow);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_memory_fill_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::Fill);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_memory_copy_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::Copy);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_env_memcpy_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::EnvMemcpy);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_env_memmove_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::EnvMemmove);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_env_memset_call(&mut self, script: &mut Vec<u8>) -> Result<()> {
        let call_pos = emit_call_placeholder(script)?;
        let record = self.helper_record_mut(MemoryHelperKind::EnvMemset);
        record.calls.push(call_pos);
        Ok(())
    }

    pub(crate) fn emit_data_init_call(
        &mut self,
        script: &mut Vec<u8>,
        segment_index: u32,
    ) -> Result<()> {
        let idx = segment_index as usize;
        let segment = self.ensure_passive_segment(idx)?;
        if let DataSegmentKind::Passive { init_record, .. } = &mut segment.kind {
            let call_pos = emit_call_placeholder(script)?;
            init_record.calls.push(call_pos);
            Ok(())
        } else {
            bail!("memory.init references active data segment {}", idx)
        }
    }

    pub(crate) fn emit_data_drop_call(
        &mut self,
        script: &mut Vec<u8>,
        segment_index: u32,
    ) -> Result<()> {
        let idx = segment_index as usize;
        let segment = self.ensure_passive_segment(idx)?;
        if let DataSegmentKind::Passive { drop_record, .. } = &mut segment.kind {
            let call_pos = emit_call_placeholder(script)?;
            drop_record.calls.push(call_pos);
            Ok(())
        } else {
            bail!("data.drop references active data segment {}", idx)
        }
    }

    pub(crate) fn register_passive_segment(&mut self, bytes: Vec<u8>) -> Result<()> {
        let index = self.next_data_index;
        self.next_data_index += 1;
        let segment = self.ensure_passive_segment(index)?;
        segment.bytes = bytes;
        segment.defined = true;
        Ok(())
    }

    pub(crate) fn register_active_segment(
        &mut self,
        _memory: u32,
        offset: u64,
        bytes: Vec<u8>,
    ) -> Result<()> {
        let index = self.next_data_index;
        self.next_data_index += 1;
        if self.data_segments.len() <= index {
            self.data_segments.push(DataSegmentInfo {
                bytes,
                kind: DataSegmentKind::Active { offset },
                defined: true,
            });
            return Ok(());
        }

        let segment = &mut self.data_segments[index];
        if segment.defined {
            bail!("data segment {} defined multiple times", index);
        }
        if let DataSegmentKind::Passive {
            init_record,
            drop_record,
            ..
        } = &segment.kind
        {
            if !init_record.calls.is_empty() || !drop_record.calls.is_empty() {
                bail!(
                    "data segment {} is active but referenced by memory.init/data.drop; only passive data segments are supported for bulk-memory operations",
                    index
                );
            }
        }
        segment.bytes = bytes;
        segment.kind = DataSegmentKind::Active { offset };
        segment.defined = true;
        Ok(())
    }

    pub(crate) fn ensure_passive_segment(&mut self, index: usize) -> Result<&mut DataSegmentInfo> {
        while self.data_segments.len() <= index {
            self.data_segments.push(DataSegmentInfo {
                bytes: Vec::new(),
                kind: DataSegmentKind::Passive {
                    init_record: HelperRecord::default(),
                    drop_record: HelperRecord::default(),
                    byte_slot: None,
                    drop_slot: None,
                },
                defined: false,
            });
        }

        let segment = &mut self.data_segments[index];
        match &segment.kind {
            DataSegmentKind::Passive { .. } => Ok(segment),
            DataSegmentKind::Active { .. } => bail!("data segment {} is active", index),
        }
    }
}

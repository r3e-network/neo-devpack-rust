use super::super::*;

impl RuntimeHelpers {
    pub(crate) fn register_table(&mut self, initial_len: usize, maximum: Option<u32>) -> usize {
        let mut entries = Vec::with_capacity(initial_len);
        for _ in 0..initial_len {
            entries.push(FUNCREF_NULL as i32);
        }
        self.tables.push(TableDescriptor {
            initial_entries: entries,
            maximum: maximum.map(|m| m as usize),
            slot: None,
        });
        self.tables.len() - 1
    }

    pub(crate) fn table_descriptor_mut(&mut self, index: usize) -> Result<&mut TableDescriptor> {
        self.tables
            .get_mut(index)
            .ok_or_else(|| anyhow!("table index {} out of range", index))
    }

    pub(crate) fn table_descriptor_const(&self, index: usize) -> Result<&TableDescriptor> {
        self.tables
            .get(index)
            .ok_or_else(|| anyhow!("table index {} out of range", index))
    }

    pub(crate) fn passive_element_slots_const(&self, index: usize) -> Result<(usize, usize)> {
        let segment = self
            .element_segments
            .get(index)
            .ok_or_else(|| anyhow!("element segment {} out of range", index))?;
        match &segment.kind {
            ElementSegmentKind::Passive {
                value_slot: Some(value_slot),
                drop_slot: Some(drop_slot),
            } => Ok((*value_slot, *drop_slot)),
            ElementSegmentKind::Passive { .. } => {
                bail!("passive element segment {} missing slot assignment", index)
            }
            ElementSegmentKind::Active { .. } => bail!("element segment {} is active", index),
        }
    }

    pub(crate) fn passive_element_drop_slot_const(&self, index: usize) -> Result<usize> {
        let (_, drop_slot) = self.passive_element_slots_const(index)?;
        Ok(drop_slot)
    }

    pub(crate) fn table_slot(&mut self, index: usize) -> Result<usize> {
        let base = BASE_STATIC_SLOTS + self.globals.len();
        let table = self.table_descriptor_mut(index)?;
        if let Some(slot) = table.slot {
            return Ok(slot);
        }
        let slot = base + index;
        table.slot = Some(slot);
        Ok(slot)
    }

    pub(crate) fn register_active_element(
        &mut self,
        table_index: usize,
        offset: usize,
        values: Vec<i32>,
    ) -> Result<usize> {
        let index = self.next_element_index;
        self.next_element_index += 1;

        if self.element_segments.len() <= index {
            self.element_segments.push(ElementSegmentInfo {
                values: values.clone(),
                kind: ElementSegmentKind::Active {
                    _table_index: table_index,
                    _offset: offset,
                },
                defined: true,
            });
        } else {
            let entry = &mut self.element_segments[index];
            if entry.defined {
                bail!("element segment {} defined multiple times", index);
            }
            entry.values = values.clone();
            entry.kind = ElementSegmentKind::Active {
                _table_index: table_index,
                _offset: offset,
            };
            entry.defined = true;
        }

        self.apply_active_element(table_index, offset, &values)?;
        Ok(index)
    }

    pub(crate) fn register_passive_element(&mut self, values: Vec<i32>) -> usize {
        let index = self.next_element_index;
        self.next_element_index += 1;

        if self.element_segments.len() <= index {
            self.element_segments.push(ElementSegmentInfo {
                values,
                kind: ElementSegmentKind::Passive {
                    value_slot: None,
                    drop_slot: None,
                },
                defined: true,
            });
        } else {
            let entry = &mut self.element_segments[index];
            entry.values = values;
            entry.kind = ElementSegmentKind::Passive {
                value_slot: None,
                drop_slot: None,
            };
            entry.defined = true;
        }

        index
    }

    pub(crate) fn ensure_passive_element(
        &mut self,
        index: usize,
    ) -> Result<&mut ElementSegmentInfo> {
        while self.element_segments.len() <= index {
            self.element_segments.push(ElementSegmentInfo {
                values: Vec::new(),
                kind: ElementSegmentKind::Passive {
                    value_slot: None,
                    drop_slot: None,
                },
                defined: false,
            });
        }

        let segment = &mut self.element_segments[index];
        match &segment.kind {
            ElementSegmentKind::Passive { .. } => Ok(segment),
            ElementSegmentKind::Active { .. } => bail!("element segment {} is active", index),
        }
    }

    pub(crate) fn apply_active_element(
        &mut self,
        table_index: usize,
        offset: usize,
        values: &[i32],
    ) -> Result<()> {
        let table = self.table_descriptor_mut(table_index)?;
        let end = offset
            .checked_add(values.len())
            .ok_or_else(|| anyhow!("element segment offset overflow"))?;
        if end > table.initial_entries.len() {
            bail!(
                "element segment writes past table bounds (offset {}, length {}, table size {})",
                offset,
                values.len(),
                table.initial_entries.len()
            );
        }
        table.initial_entries[offset..end].copy_from_slice(values);
        Ok(())
    }
}

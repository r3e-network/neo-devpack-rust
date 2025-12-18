use super::super::*;

impl RuntimeHelpers {
    pub(crate) fn register_global(&mut self, mutable: bool, initial_value: i128) -> usize {
        let slot = BASE_STATIC_SLOTS + self.globals.len();
        let const_value = if mutable { None } else { Some(initial_value) };
        self.globals.push(GlobalDescriptor {
            slot,
            mutable,
            initial_value,
            const_value,
        });
        self.globals.len() - 1
    }

    pub(crate) fn global_slot(&self, index: usize) -> Result<usize> {
        self.globals
            .get(index)
            .map(|g| g.slot)
            .ok_or_else(|| anyhow!("global index {} out of range", index))
    }

    pub(crate) fn global_mutable(&self, index: usize) -> Result<bool> {
        self.globals
            .get(index)
            .map(|g| g.mutable)
            .ok_or_else(|| anyhow!("global index {} out of range", index))
    }

    pub(crate) fn global_const_value(&self, index: usize) -> Result<Option<i128>> {
        self.globals
            .get(index)
            .map(|g| g.const_value)
            .ok_or_else(|| anyhow!("global index {} out of range", index))
    }

    pub(crate) fn clear_global_const(&mut self, index: usize) -> Result<()> {
        let global = self
            .globals
            .get_mut(index)
            .ok_or_else(|| anyhow!("global index {} out of range", index))?;
        global.const_value = None;
        Ok(())
    }
}

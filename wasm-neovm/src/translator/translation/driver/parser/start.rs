// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::*;

impl DriverState {
    pub(super) fn handle_start_section(&mut self, func: u32) -> Result<()> {
        if self.start_function.is_some() {
            bail!("module contains multiple start sections");
        }
        self.start_function = Some(func);
        Ok(())
    }
}

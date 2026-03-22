// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

#[derive(Debug, Default)]
pub(super) struct ExportedFunction {
    pub(super) names: Vec<ExportAlias>,
}

#[derive(Debug)]
pub(super) struct ExportAlias {
    pub(super) name: String,
    pub(super) processed: bool,
}

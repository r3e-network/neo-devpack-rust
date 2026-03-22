// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use anyhow::{anyhow, Result};

use crate::adapters::ChainAdapter;
use crate::manifest::ManifestBuilder;
use crate::neo_syscalls;
use crate::syscalls;

use crate::translator::FunctionImport;

#[derive(Default)]
pub(crate) struct FeatureTracker {
    storage: bool,
    payable: bool,
}

impl FeatureTracker {
    pub(crate) fn register_syscall(&mut self, descriptor: &str) {
        let lower = descriptor.to_ascii_lowercase();
        if lower.starts_with("system.storage") {
            self.storage = true;
        }
    }

    pub(crate) fn register_export(&mut self, export: &str) {
        let mut lowered = export.to_ascii_lowercase();
        lowered.retain(|c| c != '_');
        if matches!(
            lowered.as_str(),
            "onpayment" | "onnep17payment" | "onnep11payment"
        ) {
            self.payable = true;
        }
    }

    pub(crate) fn apply(&self, manifest: &mut ManifestBuilder) -> Result<()> {
        if self.storage {
            manifest.enable_feature("storage")?;
        }
        if self.payable {
            manifest.enable_feature("payable")?;
        }
        Ok(())
    }
}

pub(super) fn register_import_features(
    adapter: &dyn ChainAdapter,
    import: &FunctionImport,
    features: &mut FeatureTracker,
) -> Result<()> {
    if let Some(descriptor) = adapter.resolve_syscall(&import.module, &import.name) {
        features.register_syscall(descriptor);
        return Ok(());
    }

    let module = import.module.to_ascii_lowercase();
    match module.as_str() {
        "syscall" => {
            let syscall = syscalls::lookup_extended(&import.name)
                .ok_or_else(|| anyhow!("unknown syscall '{}'", import.name))?;
            features.register_syscall(syscall.name);
        }
        "neo" => {
            let descriptor = neo_syscalls::lookup_neo_syscall(&import.name)
                .ok_or_else(|| anyhow!("unknown Neo syscall import '{}'", import.name))?;
            features.register_syscall(descriptor);
        }
        _ => {}
    }
    Ok(())
}

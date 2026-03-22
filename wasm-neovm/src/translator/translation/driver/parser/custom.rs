// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::overlay::collect_safe_methods;
use super::super::*;

impl DriverState {
    pub(super) fn handle_custom_section(
        &mut self,
        section: wasmparser::CustomSectionReader<'_>,
    ) -> Result<()> {
        match classify_custom_section(section.name()) {
            Some(CustomSectionKind::Manifest) => {
                for overlay in parse_concatenated_json(section.data(), "neo.manifest")? {
                    collect_safe_methods(&overlay, &mut self.overlay_safe_methods);
                    if let Some(existing) = self.manifest_overlay.as_mut() {
                        merge_manifest(existing, &overlay);
                    } else {
                        self.manifest_overlay = Some(overlay);
                    }
                }
            }
            Some(CustomSectionKind::MethodTokens) => {
                let fragments = parse_concatenated_json(section.data(), "neo.methodtokens")?;
                for fragment in fragments {
                    let bytes = serde_json::to_vec(&fragment)
                        .context("failed to serialize neo.methodtokens fragment")?;
                    let metadata = parse_method_token_section(&bytes)
                        .context("failed to parse neo.methodtokens custom section fragment")?;
                    if self.section_source.is_none() {
                        self.section_source = metadata.source.clone();
                    }
                    self.section_method_tokens.extend(metadata.method_tokens);
                }
            }
            None => {}
        }

        Ok(())
    }
}

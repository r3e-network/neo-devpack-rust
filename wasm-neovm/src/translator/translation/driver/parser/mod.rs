// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

mod code;
mod custom;
mod data;
mod elements;
mod sections;
mod start;

impl DriverState {
    pub(super) fn parse_payloads(&mut self, bytes: &[u8]) -> Result<()> {
        #[cfg(feature = "profile")]
        let start = std::time::Instant::now();

        let parser = Parser::new(0);

        for payload in parser.parse_all(bytes) {
            match payload? {
                Payload::Version { .. } => {}
                Payload::TypeSection(reader) => self.handle_type_section(reader)?,
                Payload::ImportSection(reader) => self.handle_import_section(reader)?,
                Payload::FunctionSection(reader) => self.handle_function_section(reader)?,
                Payload::TableSection(reader) => self.handle_table_section(reader)?,
                Payload::GlobalSection(reader) => self.handle_global_section(reader)?,
                Payload::ExportSection(reader) => self.handle_export_section(reader)?,
                Payload::MemorySection(reader) => self.handle_memory_section(reader)?,
                Payload::CodeSectionStart { .. } => self.handle_code_section_start()?,
                Payload::CodeSectionEntry(body) => self.handle_code_section_entry(body)?,
                Payload::ElementSection(reader) => self.handle_element_section(reader)?,
                Payload::DataSection(reader) => self.handle_data_section(reader)?,
                Payload::CustomSection(section) => self.handle_custom_section(section)?,
                Payload::StartSection { func, .. } => self.handle_start_section(func)?,
                Payload::TagSection(_) => {
                    bail!(
                        "exception tags are not supported (exception-handling proposal; {})",
                        UNSUPPORTED_FEATURE_DOC
                    );
                }
                Payload::DataCountSection { .. } | Payload::UnknownSection { .. } => {}
                Payload::End(_) => break,
                _ => {}
            }
        }

        #[cfg(feature = "profile")]
        crate::translator::profiling::PROFILE.record_parse(start.elapsed().as_nanos() as u64);

        Ok(())
    }
}

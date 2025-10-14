use anyhow::{Context, Result};
use serde_json::{Deserializer, Value};
use std::str;

/// Sentinel value for null function references in tables
pub(crate) const FUNCREF_NULL: i128 = -1;

/// RET opcode byte value
pub(crate) const RET: u8 = 0x40;

/// Push integer opcode constants
pub(crate) const PUSHM1: u8 = 0x0F;
pub(crate) const PUSH0: u8 = 0x10;
pub(crate) const PUSH_BASE: u8 = 0x10; // PUSH1-PUSH16 are 0x11-0x20
pub(crate) const PUSHINT8: u8 = 0x00;
pub(crate) const PUSHINT16: u8 = 0x01;
pub(crate) const PUSHINT32: u8 = 0x02;
pub(crate) const PUSHINT64: u8 = 0x03;
pub(crate) const PUSHINT128: u8 = 0x04;

/// CONVERT opcode for type conversions
pub(crate) const CONVERT: u8 = 0xD3;

/// Stack item type constants for CONVERT
pub(crate) const STACKITEMTYPE_INTEGER: u8 = 0x21;

/// Documentation hint for unsupported features
pub(crate) const UNSUPPORTED_FEATURE_DOC: &str =
    "Refer to docs/wasm-neovm-status.md for current coverage.";

pub(crate) const CUSTOM_SECTION_PREFIX: &str = ".custom_section.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CustomSectionKind {
    Manifest,
    MethodTokens,
}

pub(crate) fn classify_custom_section(name: &str) -> Option<CustomSectionKind> {
    let stripped = name.strip_prefix(CUSTOM_SECTION_PREFIX).unwrap_or(name);
    if stripped == "neo.manifest" || stripped.starts_with("neo.manifest.") {
        Some(CustomSectionKind::Manifest)
    } else if stripped == "neo.methodtokens" || stripped.starts_with("neo.methodtokens.") {
        Some(CustomSectionKind::MethodTokens)
    } else {
        None
    }
}

pub(crate) fn parse_concatenated_json(data: &[u8], context: &str) -> Result<Vec<Value>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let text = str::from_utf8(data)
        .with_context(|| format!("{context} custom section must contain valid UTF-8 data"))?;

    let stream = Deserializer::from_str(text).into_iter::<Value>();
    let mut values = Vec::new();
    for fragment in stream {
        let value = fragment.with_context(|| format!("failed to parse {context} JSON fragment"))?;
        values.push(value);
    }

    Ok(values)
}

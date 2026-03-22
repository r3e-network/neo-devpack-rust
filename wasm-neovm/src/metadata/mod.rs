// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! NEF metadata extraction and manipulation
//!
//! This module handles metadata embedded in NEF files, including method tokens
//! for cross-contract calls and source information.

mod extract;
mod parse;
mod tokens;
mod update;

pub use extract::extract_nef_metadata;
pub use parse::parse_method_token_section;
pub use tokens::{dedup_method_tokens, method_tokens_to_json};
pub use update::update_manifest_metadata;

/// JSON key for the method tokens collection in manifest extra.
pub const TOKEN_COLLECTION_KEY: &str = "nefMethodTokens";
/// JSON key for contract hash in a method token entry.
pub const TOKEN_HASH_KEY: &str = "hash";
/// JSON key for method name in a method token entry.
pub const TOKEN_METHOD_KEY: &str = "method";
/// JSON key for parameter count in a method token entry.
pub const TOKEN_PARAMCOUNT_KEY: &str = "paramcount";
/// JSON key for the return value flag in a method token entry.
pub const TOKEN_HAS_RETURN_KEY: &str = "hasreturnvalue";
/// JSON key for call flags in a method token entry.
pub const TOKEN_CALLFLAGS_KEY: &str = "callflags";
/// JSON key for the top-level source field.
pub const SOURCE_TOP_LEVEL_KEY: &str = "source";
/// JSON key for the source field inside `extra`.
pub const SOURCE_EXTRA_KEY: &str = "nefSource";

/// Metadata extracted from a NEF manifest or custom section.
#[derive(Debug, Default, Clone)]
pub struct NefMetadata {
    /// Optional source URL for the contract.
    pub source: Option<String>,
    /// Method tokens for cross-contract calls.
    pub method_tokens: Vec<crate::nef::MethodToken>,
}

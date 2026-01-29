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

pub const TOKEN_COLLECTION_KEY: &str = "nefMethodTokens";
pub const TOKEN_HASH_KEY: &str = "hash";
pub const TOKEN_METHOD_KEY: &str = "method";
pub const TOKEN_PARAMCOUNT_KEY: &str = "paramcount";
pub const TOKEN_HAS_RETURN_KEY: &str = "hasreturnvalue";
pub const TOKEN_CALLFLAGS_KEY: &str = "callflags";
pub const SOURCE_TOP_LEVEL_KEY: &str = "source";
pub const SOURCE_EXTRA_KEY: &str = "nefSource";

#[derive(Debug, Default, Clone)]
pub struct NefMetadata {
    pub source: Option<String>,
    pub method_tokens: Vec<crate::nef::MethodToken>,
}

// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Shared helpers for cargo-fuzz targets.

use wasm_neovm::{
    api::TranslationStats, encode_nef, encode_nef_with_metadata, extract_nef_metadata,
};
use wasm_neovm::{MethodToken, SourceChain, Translation};

/// Sanitize arbitrary input into a stable contract name.
pub fn sanitize_contract_name(input: &str) -> String {
    let mut value: String = input
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(*ch, '_' | '-'))
        .take(32)
        .collect();
    if value.is_empty() {
        value.push_str("FuzzContract");
    }
    value
}

/// Sanitize arbitrary input into a compact WAT-safe symbolic name.
pub fn sanitize_symbol(input: &str, fallback: &str) -> String {
    let value: String = input
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(*ch, '_' | '-' | '.' | ':'))
        .take(48)
        .collect();
    if value.is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

/// Sanitize arbitrary input into a bounded metadata source string.
pub fn sanitize_source_url(input: Option<&str>) -> Option<String> {
    input.and_then(|raw| {
        let value: String = raw
            .chars()
            .filter(|ch| {
                ch.is_ascii_alphanumeric()
                    || matches!(*ch, ':' | '/' | '.' | '_' | '-' | '?' | '&' | '=' | '#')
            })
            .take(128)
            .collect();
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    })
}

/// Map an arbitrary byte to a source chain.
pub fn choose_chain(tag: u8) -> SourceChain {
    match tag % 3 {
        0 => SourceChain::Neo,
        1 => SourceChain::Solana,
        _ => SourceChain::Move,
    }
}

/// Assert strong post-translation invariants that should hold for all successful translations.
pub fn assert_translation_invariants(translation: &Translation) {
    assert!(
        !translation.script.is_empty(),
        "successful translation must produce a non-empty script"
    );
    assert!(
        translation.manifest.value.is_object(),
        "rendered manifest must be a JSON object"
    );

    let stats = TranslationStats::from_translation(translation);
    assert_eq!(stats.script_size, translation.script_size());
    assert_eq!(stats.token_count, translation.token_count());

    let manifest_name = translation
        .manifest
        .value
        .get("name")
        .and_then(|value| value.as_str());
    assert_eq!(manifest_name, Some(translation.contract_name.as_ref()));

    let metadata = extract_nef_metadata(&translation.manifest.value)
        .expect("translation manifests must expose extractable metadata");
    assert_eq!(
        metadata.source.as_deref(),
        translation.source_url.as_deref()
    );
    assert_method_tokens_match(&translation.method_tokens, &metadata.method_tokens);

    let plain_nef = encode_nef(&translation.script).expect("plain NEF encoding must succeed");
    let metadata_nef = encode_nef_with_metadata(
        &translation.script,
        translation.source_url.as_deref(),
        &translation.method_tokens,
    )
    .expect("metadata NEF encoding must succeed");
    assert!(
        metadata_nef.len() >= plain_nef.len(),
        "metadata-bearing NEF must not be smaller than the plain encoding"
    );
}

fn assert_method_tokens_match(expected: &[MethodToken], actual: &[MethodToken]) {
    assert_eq!(
        expected.len(),
        actual.len(),
        "manifest metadata tokens must match translation tokens"
    );
    for (lhs, rhs) in expected.iter().zip(actual.iter()) {
        assert_eq!(lhs.contract_hash, rhs.contract_hash);
        assert_eq!(lhs.method, rhs.method);
        assert_eq!(lhs.parameters_count, rhs.parameters_count);
        assert_eq!(lhs.has_return_value, rhs.has_return_value);
        assert_eq!(lhs.call_flags, rhs.call_flags);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_neovm::api::TranslationBuilder;

    #[test]
    fn sanitize_contract_name_filters_and_truncates() {
        let sanitized = sanitize_contract_name("bad name!*with spaces and punctuation___1234567890");
        assert!(sanitized.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'));
        assert!(sanitized.len() <= 32);

        assert_eq!(sanitize_contract_name("!!!"), "FuzzContract");
    }

    #[test]
    fn sanitize_symbol_preserves_allowed_characters_and_falls_back() {
        assert_eq!(sanitize_symbol("user:op-name.v1", "fallback"), "user:op-name.v1");
        assert_eq!(sanitize_symbol("   ", "fallback"), "fallback");
    }

    #[test]
    fn sanitize_source_url_filters_and_bounds_output() {
        let raw = format!("https://example.invalid/path?ok=1#frag{}", "!".repeat(200));
        let sanitized = sanitize_source_url(Some(&raw)).expect("sanitized source URL");
        assert!(sanitized.starts_with("https://example.invalid/path?ok=1#frag"));
        assert!(sanitized.len() <= 128);
        assert!(sanitize_source_url(Some("%%%")).is_none());
    }

    #[test]
    fn choose_chain_maps_bytes_across_all_supported_chains() {
        assert_eq!(choose_chain(0), SourceChain::Neo);
        assert_eq!(choose_chain(1), SourceChain::Solana);
        assert_eq!(choose_chain(2), SourceChain::Move);
        assert_eq!(choose_chain(5), SourceChain::Move);
    }

    #[test]
    fn translation_invariants_hold_for_simple_module_with_source_url() {
        let wasm = wat::parse_str(
            r#"(module
                (func (export "main") (result i32)
                    i32.const 7
                )
            )"#,
        )
        .expect("valid WAT");

        let translation = TranslationBuilder::new("FuzzInvariant")
            .with_wasm(wasm)
            .with_source_url("https://example.invalid/contracts/fuzz-invariant.wat")
            .translate()
            .expect("translation succeeds");

        assert_translation_invariants(&translation);
    }
}

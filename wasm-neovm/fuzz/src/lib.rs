// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Shared helpers for cargo-fuzz targets.

use arbitrary::Arbitrary;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use wasm_neovm::api::TranslationBuilder;
use wasm_neovm::{
    api::TranslationStats, encode_nef, encode_nef_with_metadata, extract_nef_metadata,
};
use wasm_neovm::{BehaviorConfig, MethodToken, SourceChain, Translation};

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

/// Shared structured input for Rust contract compiler/devpack fuzzing.
#[derive(Debug, Clone, Arbitrary)]
pub struct RustContractFuzzInput<'a> {
    pub contract_name: &'a str,
    pub method_seed: &'a str,
    pub overlay_tag: &'a str,
    pub flags: u16,
    pub method_count: u8,
    pub base_value: i64,
    pub alternate_value: i64,
    pub shape_bytes: Vec<u8>,
    pub strict_validation: bool,
    pub aggressive_optimization: bool,
    pub enable_bulk_memory: bool,
}

#[derive(Debug, Clone)]
pub struct GeneratedRustContract {
    pub contract_name: String,
    pub source: String,
    pub source_url: String,
    pub behavior: BehaviorConfig,
    pub expected_methods: Vec<ExpectedRustMethod>,
    pub expected_event: Option<String>,
    pub expected_standard: Option<String>,
    pub expected_overlay_tag: String,
    pub expects_permissions: bool,
    pub expects_trusts: bool,
    pub expects_entry: bool,
    pub expects_storage_feature: bool,
}

#[derive(Debug, Clone)]
pub struct ExpectedRustMethod {
    pub name: String,
    pub safe: bool,
    pub has_last_error: bool,
}

#[derive(Debug, Clone)]
pub struct CompiledRustContract {
    pub wasm: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum RustContractCompileOutcome {
    Compiled(CompiledRustContract),
    Rejected,
}

/// Sanitize arbitrary input into a Rust identifier suitable for function names.
pub fn sanitize_rust_ident(input: &str, fallback: &str) -> String {
    let mut output = String::with_capacity(input.len().min(64));
    let mut previous_was_underscore = false;

    for ch in input.chars().take(64) {
        if ch.is_ascii_alphanumeric() {
            if output.is_empty() && ch.is_ascii_digit() {
                output.push('f');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_underscore = false;
        } else if !output.is_empty() && !previous_was_underscore {
            output.push('_');
            previous_was_underscore = true;
        }
    }

    while output.ends_with('_') {
        output.pop();
    }

    if output.is_empty() {
        output.push_str(fallback);
    }

    if is_rust_keyword(&output) {
        output.insert_str(0, "neo_");
    }

    output
}

/// Sanitize arbitrary input into a Rust type name.
pub fn sanitize_rust_type_name(input: &str, fallback: &str) -> String {
    let mut output = String::with_capacity(input.len().min(64));
    let mut uppercase_next = true;

    for ch in input.chars().take(64) {
        if ch.is_ascii_alphanumeric() {
            if output.is_empty() && ch.is_ascii_digit() {
                output.push('F');
            }
            if uppercase_next {
                output.push(ch.to_ascii_uppercase());
            } else {
                output.push(ch.to_ascii_lowercase());
            }
            uppercase_next = false;
        } else if !output.is_empty() {
            uppercase_next = true;
        }
    }

    if output.is_empty() {
        fallback.to_string()
    } else {
        output
    }
}

/// Convert a snake_case method name into the export name emitted by `#[neo_method]`.
pub fn snake_to_camel_case(name: &str) -> String {
    let mut output = String::with_capacity(name.len());
    let mut uppercase_next = false;

    for ch in name.chars() {
        if ch == '_' {
            uppercase_next = true;
            continue;
        }

        if uppercase_next {
            output.push(ch.to_ascii_uppercase());
            uppercase_next = false;
        } else {
            output.push(ch);
        }
    }

    output
}

/// Render a structured Rust devpack contract that stays inside the supported wrapper/macro surface.
pub fn render_structured_rust_contract(input: &RustContractFuzzInput<'_>) -> GeneratedRustContract {
    let contract_name = sanitize_rust_type_name(input.contract_name, "FuzzContract");
    let method_seed = sanitize_rust_ident(input.method_seed, "method");
    let overlay_tag = sanitize_contract_name(input.overlay_tag);
    let include_event = input.flags & (1 << 0) != 0;
    let include_storage = input.flags & (1 << 1) != 0;
    let include_entry = input.flags & (1 << 2) != 0;
    let include_permissions = input.flags & (1 << 3) != 0;
    let include_standards = input.flags & (1 << 4) != 0;
    let include_trusts = input.flags & (1 << 5) != 0;
    let include_overlay_safe_methods = input.flags & (1 << 6) != 0;
    let use_custom_export_names = input.flags & (1 << 7) != 0;
    let use_legacy_export_name = input.flags & (1 << 8) != 0;

    let bounded_base = input.base_value.wrapping_rem(4_096);
    let bounded_alt = input.alternate_value.wrapping_rem(4_096);
    let offset_value = 1 + i64::from((input.flags % 97) as u8);
    let method_total = 2 + usize::from(input.method_count % 5);
    let event_name = format!(
        "{}Event",
        sanitize_rust_type_name(input.method_seed, "Fuzz")
    );
    let storage_name = format!(
        "{}Store",
        sanitize_rust_type_name(input.overlay_tag, "Fuzz")
    );
    let source_url = format!(
        "https://fuzz.invalid/rust/{}/{}.rs",
        sanitize_contract_name(&contract_name.to_ascii_lowercase()),
        overlay_tag
    );
    let expected_standard = if include_standards {
        Some(if input.flags & (1 << 9) != 0 {
            "NEP-11".to_string()
        } else {
            "NEP-17".to_string()
        })
    } else {
        None
    };

    let mut expected_methods = Vec::with_capacity(method_total);
    let mut safe_overlay_methods = Vec::with_capacity(method_total);
    let mut method_tokens = String::new();

    for index in 0..method_total {
        let shape = if index < 4 {
            index as u8
        } else {
            input
                .shape_bytes
                .get(index - 4)
                .copied()
                .unwrap_or_else(|| (index as u8).wrapping_add(input.flags as u8))
                % 6
        };
        let method_ident = format!("{method_seed}_{index}");
        let default_export_name = snake_to_camel_case(&method_ident);
        let export_name = if use_custom_export_names && index % 2 == 1 {
            format!(
                "{}Alias{}",
                sanitize_rust_type_name(input.method_seed, "Method"),
                index
            )
        } else {
            default_export_name
        };
        let attr_safe = matches!(shape, 0 | 1 | 3 | 4) && index % 2 == 0;
        let overlay_safe =
            include_overlay_safe_methods && matches!(shape, 0 | 1 | 3 | 4) && index % 2 == 1;
        if overlay_safe {
            safe_overlay_methods.push(export_name.clone());
        }

        let method_attribute = render_method_attribute(
            attr_safe,
            &export_name,
            use_custom_export_names && index % 2 == 1,
            use_legacy_export_name && index % 4 == 3,
        );

        let (body, has_last_error) = render_method_body(
            shape,
            &method_ident,
            include_storage,
            include_event,
            &event_name,
            &storage_name,
            bounded_base,
            bounded_alt,
            offset_value + index as i64,
        );
        method_tokens.push_str(&method_attribute);
        method_tokens.push('\n');
        method_tokens.push_str(&body);
        method_tokens.push('\n');

        expected_methods.push(ExpectedRustMethod {
            name: export_name,
            safe: attr_safe || overlay_safe,
            has_last_error,
        });
    }

    if include_overlay_safe_methods && safe_overlay_methods.is_empty() {
        if let Some(first_safe_candidate) = expected_methods.first() {
            safe_overlay_methods.push(first_safe_candidate.name.clone());
        }
    }

    let mut source = String::new();
    source.push_str("use neo_devpack::prelude::*;\n");
    if include_storage {
        source.push_str("use neo_devpack::neo_storage;\n");
        source.push_str("use serde::{Deserialize, Serialize};\n");
    }
    source.push('\n');
    source.push_str(&format!(
        "const BASE_VALUE: i64 = {bounded_base};\nconst ALT_VALUE: i64 = {bounded_alt};\nconst OFFSET_VALUE: i64 = {offset_value};\n\n"
    ));
    source.push_str(&format!(
        "neo_manifest_overlay!(r#\"{{\n    \"name\": \"{contract_name}\",\n    \"extra\": {{ \"tag\": \"{overlay_tag}\" }},\n    \"features\": {{ \"storage\": {} }}\n}}\"#);\n",
        if include_storage { "true" } else { "false" }
    ));
    if include_permissions {
        source.push_str("neo_permission!(\"*\", [\"balanceOf\"]);\n");
    }
    if let Some(standard) = expected_standard.as_deref() {
        source.push_str(&format!("neo_supported_standards!([\"{standard}\"]);\n"));
    }
    if include_trusts {
        source.push_str("neo_trusts!([\"*\"]);\n");
    }
    if !safe_overlay_methods.is_empty() {
        source.push_str("neo_safe_methods!([");
        for (index, method_name) in safe_overlay_methods.iter().enumerate() {
            if index > 0 {
                source.push_str(", ");
            }
            source.push('"');
            source.push_str(method_name);
            source.push('"');
        }
        source.push_str("]);\n");
    }
    source.push('\n');

    if include_event {
        source.push_str(&format!(
            "#[neo_event]\npub struct {event_name} {{\n    pub amount: NeoInteger,\n    pub accepted: NeoBoolean,\n}}\n\n"
        ));
    }

    if include_storage {
        source.push_str(&format!(
            "#[derive(Default, Serialize, Deserialize)]\n#[neo_storage]\npub struct {storage_name} {{\n    value: NeoInteger,\n    flag: NeoBoolean,\n}}\n\n"
        ));
    }

    source.push_str(&format!(
        "#[neo_contract]\npub struct {contract_name};\n\n#[neo_contract]\nimpl {contract_name} {{\n    pub fn new() -> Self {{\n        Self\n    }}\n\n{method_tokens}}}\n\nimpl Default for {contract_name} {{\n    fn default() -> Self {{\n        Self::new()\n    }}\n}}\n"
    ));

    if include_entry {
        source.push_str("\n#[neo_entry]\npub fn deploy() -> NeoResult<()> {\n    Ok(())\n}\n");
    }

    GeneratedRustContract {
        contract_name,
        source,
        source_url,
        behavior: BehaviorConfig {
            max_memory_pages: 32 + u32::from((input.flags % 16) as u8),
            max_table_size: 64 + u32::from((input.method_count % 32) as u8),
            stack_size_limit: 256 + u32::from((input.flags % 256) as u8),
            aggressive_optimization: input.aggressive_optimization,
            strict_validation: input.strict_validation,
            allow_float: false,
            enable_bulk_memory: input.enable_bulk_memory,
            ..BehaviorConfig::default()
        },
        expected_methods,
        expected_event: include_event.then_some(event_name),
        expected_standard,
        expected_overlay_tag: overlay_tag,
        expects_permissions: include_permissions,
        expects_trusts: include_trusts,
        expects_entry: include_entry,
        expects_storage_feature: include_storage,
    }
}

/// Compile a generated Rust contract to Wasm using the same devpack surface the end-user pipeline uses.
pub fn compile_generated_rust_contract(
    workspace_key: &str,
    contract: &GeneratedRustContract,
) -> RustContractCompileOutcome {
    let workspace_root = rust_contract_workspace_root(workspace_key);
    let source_dir = workspace_root.join("src");
    let manifest_path = workspace_root.join("Cargo.toml");
    let source_path = source_dir.join("lib.rs");
    let target_dir = workspace_root.join("target-artifacts");
    let wasm_path = target_dir
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("fuzz_contract.wasm");

    fs::create_dir_all(&source_dir).expect("create Rust contract fuzz workspace");
    fs::write(&manifest_path, render_rust_contract_cargo_toml()).expect("write Cargo.toml");
    fs::write(&source_path, &contract.source).expect("write lib.rs");
    let _ = fs::remove_file(&wasm_path);

    let output = Command::new("cargo")
        .arg("+stable")
        .arg("build")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .arg("--release")
        .current_dir(&workspace_root)
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("CARGO_BUILD_RUSTFLAGS")
        .env_remove("CARGO_TARGET_DIR")
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("spawn cargo for rust contract fuzzing");

    let stderr = String::from_utf8_lossy(&output.stderr);
    if output.status.code().is_none() || rust_compiler_failed_internally(&stderr) {
        panic!(
            "Rust compiler/devpack crashed while compiling fuzz contract `{}`:\n{}",
            contract.contract_name, stderr
        );
    }

    if !output.status.success() {
        return RustContractCompileOutcome::Rejected;
    }

    let wasm = fs::read(&wasm_path).expect("compiled rust contract must produce a Wasm artifact");
    RustContractCompileOutcome::Compiled(CompiledRustContract { wasm })
}

/// Translate a compiled Rust contract with the Neo pipeline and assert postconditions that should hold.
pub fn translate_generated_rust_contract(
    compiled: &CompiledRustContract,
    contract: &GeneratedRustContract,
) -> Translation {
    let builder = TranslationBuilder::new(contract.contract_name.as_str())
        .with_wasm(compiled.wasm.clone())
        .from_chain(SourceChain::Neo)
        .with_behavior(contract.behavior.clone())
        .with_source_url(contract.source_url.clone());

    let (translation, stats) = builder.translate_with_stats().unwrap_or_else(|error| {
        panic!(
            "Rust devpack contract translated unsuccessfully for `{}`: {error}\n{}",
            contract.contract_name, contract.source
        )
    });

    assert!(
        stats.translation_time_ms.is_some(),
        "translate_with_stats must populate timing information for rust contracts"
    );
    assert!(
        stats.export_count >= contract.expected_methods.len(),
        "generated rust contracts must export their expected methods"
    );
    assert_translation_invariants(&translation);
    assert_generated_rust_contract_manifest(&translation, contract);
    translation
}

/// Assert deterministic parity between two successful Rust contract translations.
pub fn assert_rust_contract_translation_parity(lhs: &Translation, rhs: &Translation) {
    assert_eq!(
        lhs.contract_name, rhs.contract_name,
        "Rust contract translations must agree on the contract name"
    );
    assert_eq!(
        lhs.script, rhs.script,
        "Rust contract translations must emit deterministic NeoVM scripts"
    );
    assert_eq!(
        lhs.manifest.value, rhs.manifest.value,
        "Rust contract translations must emit deterministic manifests"
    );
    assert_method_tokens_match(&lhs.method_tokens, &rhs.method_tokens);

    let lhs_nef =
        encode_nef_with_metadata(&lhs.script, lhs.source_url.as_deref(), &lhs.method_tokens)
            .expect("left NEF encoding must succeed");
    let rhs_nef =
        encode_nef_with_metadata(&rhs.script, rhs.source_url.as_deref(), &rhs.method_tokens)
            .expect("right NEF encoding must succeed");
    assert_eq!(lhs_nef, rhs_nef, "NEF output must stay deterministic");
}

fn render_method_attribute(
    attr_safe: bool,
    export_name: &str,
    custom_export_name: bool,
    legacy_export_name: bool,
) -> String {
    let mut options = Vec::with_capacity(2);
    if custom_export_name {
        if legacy_export_name {
            options.push(format!("export_name = \"{export_name}\""));
        } else {
            options.push(format!("name = \"{export_name}\""));
        }
    }
    if attr_safe {
        options.push("safe".to_string());
    }

    if options.is_empty() {
        "#[neo_method]".to_string()
    } else {
        format!("#[neo_method({})]", options.join(", "))
    }
}

fn render_method_body(
    shape: u8,
    method_ident: &str,
    include_storage: bool,
    include_event: bool,
    event_name: &str,
    storage_name: &str,
    bounded_base: i64,
    bounded_alt: i64,
    offset_value: i64,
) -> (String, bool) {
    let tokens = match shape {
        0 => format!(
            "    pub fn {method_ident}(&self, input: NeoInteger) -> NeoResult<NeoInteger> {{\n        Ok(input + NeoInteger::new(BASE_VALUE.wrapping_add({offset_value})))\n    }}\n"
        ),
        1 => format!(
            "    pub fn {method_ident}(&self, flag: NeoBoolean) -> NeoResult<NeoBoolean> {{\n        let expected = ((ALT_VALUE.wrapping_add({offset_value}) & 1) == 0);\n        Ok(NeoBoolean::new(flag.as_bool() ^ expected))\n    }}\n"
        ),
        2 => render_mutating_void_method(
            method_ident,
            include_storage,
            include_event,
            event_name,
            storage_name,
        ),
        3 => format!(
            "    pub fn {method_ident}(input: i64, flag: bool) -> i64 {{\n        if flag {{\n            input.wrapping_add(BASE_VALUE.wrapping_add({offset_value}))\n        }} else {{\n            input.wrapping_sub(ALT_VALUE.wrapping_sub({offset_value}))\n        }}\n    }}\n"
        ),
        4 => format!(
            "    pub fn {method_ident}(&self) -> bool {{\n        (BASE_VALUE.wrapping_add(OFFSET_VALUE).wrapping_add({offset_value}) & 1) == 0\n    }}\n"
        ),
        _ => render_mutating_result_method(
            method_ident,
            include_storage,
            include_event,
            event_name,
            storage_name,
            bounded_base,
            bounded_alt,
            offset_value,
        ),
    };

    let has_last_error = !matches!(shape, 3 | 4);
    (tokens, has_last_error)
}

fn render_mutating_void_method(
    method_ident: &str,
    include_storage: bool,
    include_event: bool,
    event_name: &str,
    storage_name: &str,
) -> String {
    let mut body = String::new();
    body.push_str(&format!(
        "    pub fn {method_ident}(&mut self, amount: NeoInteger) -> NeoResult<()> {{\n"
    ));
    if include_storage {
        body.push_str("        let context = NeoRuntime::get_storage_context()?;\n");
        body.push_str(&format!(
            "        let mut state = {storage_name}::load(&context);\n"
        ));
        body.push_str("        state.value = state.value + amount.clone();\n");
        body.push_str(
            "        state.flag = NeoBoolean::new((state.value.as_i32_saturating() & 1) == 0);\n",
        );
        body.push_str("        state.save(&context)?;\n");
        if include_event {
            body.push_str(&format!(
                "        {event_name} {{ amount: state.value.clone(), accepted: state.flag.clone() }}.emit()?;\n"
            ));
        }
    } else {
        body.push_str(
            "        let accepted = NeoBoolean::new((amount.as_i32_saturating() & 1) == 0);\n",
        );
        if include_event {
            body.push_str(&format!(
                "        {event_name} {{ amount: amount.clone(), accepted }}.emit()?;\n"
            ));
        } else {
            body.push_str("        let _ = accepted;\n");
        }
    }
    body.push_str("        Ok(())\n");
    body.push_str("    }\n");
    body
}

fn render_mutating_result_method(
    method_ident: &str,
    include_storage: bool,
    include_event: bool,
    event_name: &str,
    storage_name: &str,
    bounded_base: i64,
    bounded_alt: i64,
    offset_value: i64,
) -> String {
    let mut body = String::new();
    body.push_str(&format!(
        "    pub fn {method_ident}(&mut self, flag: NeoBoolean) -> NeoResult<NeoInteger> {{\n"
    ));
    if include_storage {
        body.push_str("        let context = NeoRuntime::get_storage_context()?;\n");
        body.push_str(&format!(
            "        let mut state = {storage_name}::load(&context);\n"
        ));
        body.push_str("        let delta = if flag.as_bool() { NeoInteger::one() } else { NeoInteger::new(OFFSET_VALUE) };\n");
        body.push_str("        state.value = state.value + delta;\n");
        body.push_str("        state.flag = flag.clone();\n");
        body.push_str("        state.save(&context)?;\n");
        if include_event {
            body.push_str(&format!(
                "        {event_name} {{ amount: state.value.clone(), accepted: flag.clone() }}.emit()?;\n"
            ));
        }
        body.push_str("        Ok(state.value.clone())\n");
    } else {
        body.push_str(&format!(
            "        let value = if flag.as_bool() {{ BASE_VALUE.wrapping_add({bounded_base}) }} else {{ ALT_VALUE.wrapping_add({bounded_alt}) }};\n"
        ));
        if include_event {
            body.push_str(&format!(
                "        {event_name} {{ amount: NeoInteger::new(value), accepted: flag.clone() }}.emit()?;\n"
            ));
        }
        body.push_str(&format!(
            "        Ok(NeoInteger::new(value.wrapping_add({offset_value})))\n"
        ));
    }
    body.push_str("    }\n");
    body
}

fn render_rust_contract_cargo_toml() -> String {
    let rust_devpack_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../rust-devpack")
        .canonicalize()
        .expect("resolve rust-devpack path");

    format!(
        "[package]\nname = \"fuzz-contract\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\nneo-devpack = {{ path = \"{}\", default-features = false }}\nserde = {{ version = \"1\", default-features = false, features = [\"derive\"] }}\n\n[workspace]\n\n[profile.release]\nopt-level = \"z\"\nlto = true\ncodegen-units = 1\npanic = \"abort\"\nstrip = \"symbols\"\n",
        rust_devpack_path.display()
    )
}

fn rust_contract_workspace_root(workspace_key: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("rust-contract-harness")
        .join(format!("pid-{}", std::process::id()))
        .join(workspace_key)
}

fn rust_compiler_failed_internally(stderr: &str) -> bool {
    stderr.contains("internal compiler error")
        || stderr.contains("query stack during panic")
        || stderr.contains("compiler unexpectedly panicked")
}

fn assert_generated_rust_contract_manifest(
    translation: &Translation,
    contract: &GeneratedRustContract,
) {
    let manifest = &translation.manifest.value;
    let methods = manifest["abi"]["methods"]
        .as_array()
        .expect("translated rust contract must expose ABI methods");

    for expected in &contract.expected_methods {
        let method = methods
            .iter()
            .find(|method| method["name"].as_str() == Some(expected.name.as_str()))
            .unwrap_or_else(|| {
                panic!(
                    "manifest missing expected rust contract method `{}`",
                    expected.name
                )
            });
        assert_eq!(
            method["safe"].as_bool().unwrap_or(false),
            expected.safe,
            "manifest safe flag drift for rust contract method `{}`",
            expected.name
        );

        if expected.has_last_error {
            let last_error_name = format!("{}LastError", expected.name);
            let last_error_method = methods
                .iter()
                .find(|method| method["name"].as_str() == Some(last_error_name.as_str()))
                .unwrap_or_else(|| {
                    panic!(
                        "manifest missing generated status export `{last_error_name}` for rust contract"
                    )
                });
            assert!(
                !last_error_method["safe"].as_bool().unwrap_or(false),
                "generated status exports must never be marked safe"
            );
        }
    }

    if let Some(event_name) = contract.expected_event.as_deref() {
        let events = manifest["abi"]["events"]
            .as_array()
            .expect("translated rust contract must expose ABI events");
        assert!(
            events
                .iter()
                .any(|event| event["name"].as_str() == Some(event_name)),
            "manifest missing expected rust contract event `{event_name}`"
        );
    }

    if let Some(standard) = contract.expected_standard.as_deref() {
        let standards = manifest["supportedstandards"]
            .as_array()
            .expect("supported standards must be rendered as an array");
        assert!(
            standards
                .iter()
                .any(|value| value.as_str() == Some(standard)),
            "manifest missing expected supported standard `{standard}`"
        );
    }

    if contract.expects_permissions {
        let permissions = manifest["permissions"]
            .as_array()
            .expect("permissions must be rendered as an array");
        assert!(
            permissions.iter().any(|entry| {
                entry["contract"].as_str() == Some("*")
                    && entry["methods"]
                        .as_array()
                        .map(|methods| {
                            methods
                                .iter()
                                .any(|method| method.as_str() == Some("balanceOf"))
                        })
                        .unwrap_or(false)
            }),
            "manifest missing expected permission macro output"
        );
    }

    if contract.expects_trusts {
        let trusts = manifest["trusts"]
            .as_array()
            .expect("trusts must be rendered as an array");
        assert!(
            trusts.iter().any(|entry| entry.as_str() == Some("*")),
            "manifest missing expected trusts macro output"
        );
    }

    if contract.expects_entry {
        assert_eq!(
            manifest["entry"]["name"].as_str(),
            Some("deploy"),
            "manifest entry metadata drifted from the generated #[neo_entry] function"
        );
    }

    assert_eq!(
        manifest["features"]["storage"].as_bool(),
        Some(contract.expects_storage_feature),
        "manifest storage feature drifted from the generated overlay"
    );
    assert_eq!(
        manifest["extra"]["tag"].as_str(),
        Some(contract.expected_overlay_tag.as_str()),
        "manifest overlay tag drifted during translation"
    );
}

fn is_rust_keyword(candidate: &str) -> bool {
    matches!(
        candidate,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use arbitrary::Arbitrary;

    fn load_saved_rust_contract_input(name: &str) -> RustContractFuzzInput<'static> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("artifacts")
            .join("fuzz_rust_contract_differential")
            .join(name);
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read saved differential artifact `{}`: {error}",
                path.display()
            )
        });
        let leaked = Box::leak(bytes.into_boxed_slice());
        let mut unstructured = arbitrary::Unstructured::new(leaked);
        RustContractFuzzInput::arbitrary(&mut unstructured)
            .expect("saved differential artifact must decode as RustContractFuzzInput")
    }

    #[test]
    fn sanitize_contract_name_filters_and_truncates() {
        let sanitized =
            sanitize_contract_name("bad name!*with spaces and punctuation___1234567890");
        assert!(sanitized
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'));
        assert!(sanitized.len() <= 32);

        assert_eq!(sanitize_contract_name("!!!"), "FuzzContract");
    }

    #[test]
    fn sanitize_symbol_preserves_allowed_characters_and_falls_back() {
        assert_eq!(
            sanitize_symbol("user:op-name.v1", "fallback"),
            "user:op-name.v1"
        );
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

    #[test]
    fn sanitize_rust_ident_prefixes_digits_and_keywords() {
        assert_eq!(
            sanitize_rust_ident("42 weird value", "fallback"),
            "f42_weird_value"
        );
        assert_eq!(sanitize_rust_ident("match", "fallback"), "neo_match");
    }

    #[test]
    fn sanitize_rust_type_name_builds_pascal_case_names() {
        assert_eq!(
            sanitize_rust_type_name("hello-world", "Fallback"),
            "HelloWorld"
        );
        assert_eq!(sanitize_rust_type_name("99", "Fallback"), "F99");
    }

    #[test]
    fn snake_to_camel_case_matches_macro_export_rules() {
        assert_eq!(snake_to_camel_case("get_value"), "getValue");
        assert_eq!(snake_to_camel_case("raw_value_status"), "rawValueStatus");
    }

    #[test]
    fn render_structured_rust_contract_tracks_expected_manifest_shape() {
        let rendered = render_structured_rust_contract(&RustContractFuzzInput {
            contract_name: "hello_contract",
            method_seed: "counter",
            overlay_tag: "ci",
            flags: 0b11_1111_1111,
            method_count: 4,
            base_value: 7,
            alternate_value: -9,
            shape_bytes: vec![0, 1, 2, 5],
            strict_validation: true,
            aggressive_optimization: false,
            enable_bulk_memory: true,
        });

        assert!(rendered.source.contains("#[neo_contract]"));
        assert!(rendered.source.contains("neo_manifest_overlay!"));
        assert!(rendered.expected_methods.iter().any(|method| method.safe));
        assert!(rendered.expected_event.is_some());
        assert!(rendered.expects_storage_feature);
    }

    #[test]
    fn structured_rust_contract_smoke_compiles_and_translates() {
        let rendered = render_structured_rust_contract(&RustContractFuzzInput {
            contract_name: "smoke_contract",
            method_seed: "balance",
            overlay_tag: "smoke",
            flags: 0b11_1111_1111,
            method_count: 4,
            base_value: 11,
            alternate_value: -17,
            shape_bytes: vec![0, 1, 2, 5],
            strict_validation: true,
            aggressive_optimization: false,
            enable_bulk_memory: true,
        });

        let compiled = match compile_generated_rust_contract("lib-test-smoke", &rendered) {
            RustContractCompileOutcome::Compiled(compiled) => compiled,
            RustContractCompileOutcome::Rejected => {
                panic!(
                    "generated smoke contract unexpectedly failed to compile:\n{}",
                    rendered.source
                )
            }
        };

        let translation = translate_generated_rust_contract(&compiled, &rendered);
        assert_eq!(
            translation.contract_name.as_ref(),
            rendered.contract_name.as_str()
        );
    }

    #[test]
    fn saved_differential_artifact_translates_deterministically() {
        let input =
            load_saved_rust_contract_input("crash-1fb38c7780f76f2eaae669eb80b9b0fc5240935c");
        let rendered = render_structured_rust_contract(&input);

        let first = match compile_generated_rust_contract("lib-test-differential-a", &rendered) {
            RustContractCompileOutcome::Compiled(compiled) => compiled,
            RustContractCompileOutcome::Rejected => {
                panic!("saved differential artifact unexpectedly failed first compilation")
            }
        };
        let second = match compile_generated_rust_contract("lib-test-differential-b", &rendered) {
            RustContractCompileOutcome::Compiled(compiled) => compiled,
            RustContractCompileOutcome::Rejected => {
                panic!("saved differential artifact unexpectedly failed second compilation")
            }
        };

        assert_eq!(
            first.wasm, second.wasm,
            "saved differential artifact must compile to deterministic Wasm"
        );

        let first_translation = translate_generated_rust_contract(&first, &rendered);
        let second_translation = translate_generated_rust_contract(&second, &rendered);
        assert_rust_contract_translation_parity(&first_translation, &second_translation);
    }
}

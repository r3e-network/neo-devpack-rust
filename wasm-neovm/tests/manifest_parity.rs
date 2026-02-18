use wasm_neovm::manifest::{build_manifest, ManifestMethod, ManifestParameter};

fn method(name: &str, params: &[&str], return_type: &str) -> ManifestMethod {
    ManifestMethod {
        name: name.to_string(),
        parameters: params
            .iter()
            .enumerate()
            .map(|(idx, kind)| ManifestParameter {
                name: format!("arg{idx}"),
                kind: (*kind).to_string(),
            })
            .collect(),
        return_type: return_type.to_string(),
        offset: 0,
        safe: false,
    }
}

fn standards(manifest: &serde_json::Value) -> Vec<String> {
    manifest["supportedstandards"]
        .as_array()
        .expect("supported standards array")
        .iter()
        .filter_map(|value| value.as_str())
        .map(str::to_string)
        .collect()
}

#[test]
fn nep17_detection_requires_expected_method_shape() {
    let valid_methods = vec![
        method("symbol", &[], "String"),
        method("decimals", &[], "Integer"),
        method("total_supply", &[], "Integer"),
        method("balance_of", &["Hash160"], "Integer"),
        method("transfer", &["Hash160", "Hash160", "Integer"], "Boolean"),
    ];
    let valid_manifest = build_manifest("ValidNEP17", &valid_methods).value;
    let valid_standards = standards(&valid_manifest);
    assert!(valid_standards.iter().any(|entry| entry == "NEP-17"));

    let invalid_methods = vec![
        method("symbol", &["Any"], "String"),
        method("decimals", &[], "Integer"),
        method("total_supply", &[], "Integer"),
        method("balance_of", &["Hash160"], "Integer"),
        method("transfer", &["Hash160", "Integer"], "Boolean"),
    ];
    let invalid_manifest = build_manifest("InvalidNEP17", &invalid_methods).value;
    let invalid_standards = standards(&invalid_manifest);
    assert!(!invalid_standards.iter().any(|entry| entry == "NEP-17"));
}

#[test]
fn lifecycle_alias_types_are_detected_without_overaccepting_unknown_types() {
    let alias_methods = vec![
        method("update", &["byte_array", "str", "Map"], "void"),
        method("_deploy", &["Map", "bool"], "VOID"),
        method(
            "on_nep11_payment",
            &["UInt160", "int", "buffer", "Map"],
            "Void",
        ),
        method(
            "on_nep17_payment",
            &["script_hash", "Integer", "Map"],
            "Void",
        ),
    ];
    let alias_manifest = build_manifest("AliasLifecycle", &alias_methods).value;
    let alias_standards = standards(&alias_manifest);
    for expected in ["NEP-22", "NEP-26", "NEP-27", "NEP-29"] {
        assert!(alias_standards.iter().any(|entry| entry == expected));
    }

    let unknown_methods = vec![
        method("update", &["Bytes", "str", "Any"], "Void"),
        method("on_nep17_payment", &["Hash256", "int", "Any"], "Void"),
    ];
    let unknown_manifest = build_manifest("UnknownLifecycle", &unknown_methods).value;
    let unknown_standards = standards(&unknown_manifest);
    assert!(!unknown_standards.iter().any(|entry| entry == "NEP-22"));
    assert!(!unknown_standards.iter().any(|entry| entry == "NEP-27"));
}

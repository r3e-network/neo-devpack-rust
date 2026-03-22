// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::{
    build_manifest, collect_method_names, ensure_manifest_methods_match, merge_manifest,
    ManifestMethod, ManifestParameter,
};
use serde_json::json;
use std::collections::HashSet;

fn manifest_method(name: &str, param_count: usize) -> ManifestMethod {
    ManifestMethod {
        name: name.to_string(),
        parameters: (0..param_count)
            .map(|idx| ManifestParameter {
                name: format!("arg{idx}"),
                kind: "Any".to_string(),
            })
            .collect(),
        return_type: "Any".to_string(),
        offset: 0,
        safe: false,
    }
}

fn void_manifest_method(name: &str, param_count: usize) -> ManifestMethod {
    let mut method = manifest_method(name, param_count);
    method.return_type = "Void".to_string();
    method
}

fn typed_manifest_method(name: &str, kinds: &[&str], return_type: &str) -> ManifestMethod {
    ManifestMethod {
        name: name.to_string(),
        parameters: kinds
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

#[test]
fn merge_manifest_deeply_combines_objects() {
    let mut base = json!({
        "abi": {
            "methods": ["foo"],
            "events": []
        },
        "extra": {
            "author": "neo-devpack-rust"
        }
    });
    let overlay = json!({
        "abi": {
            "events": [
                {"name": "Transfer", "parameters": []}
            ]
        },
        "supportedstandards": ["NEP-17"]
    });

    merge_manifest(&mut base, &overlay);

    assert_eq!(
        base["supportedstandards"].as_array().unwrap()[0]
            .as_str()
            .unwrap(),
        "NEP-17"
    );
    assert_eq!(base["abi"]["methods"].as_array().unwrap().len(), 1);
    assert_eq!(base["abi"]["events"].as_array().unwrap().len(), 1);
}

#[test]
fn build_manifest_detects_nep17_nep24_nep26_nep27() {
    let methods = vec![
        manifest_method("symbol", 0),
        manifest_method("decimals", 0),
        manifest_method("total_supply", 0),
        manifest_method("balanceOf", 1),
        manifest_method("transfer", 3),
        manifest_method("royalty_info", 3),
        void_manifest_method("on_nep11_payment", 4),
        void_manifest_method("on_nep17_payment", 3),
    ];

    let manifest = build_manifest("TokenLike", &methods).value;
    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supported standards array");
    let standards: Vec<&str> = standards
        .iter()
        .filter_map(|value| value.as_str())
        .collect();

    assert!(standards.contains(&"NEP-17"));
    assert!(standards.contains(&"NEP-24"));
    assert!(standards.contains(&"NEP-26"));
    assert!(standards.contains(&"NEP-27"));
}

#[test]
fn build_manifest_detects_lifecycle_neps() {
    let mut verify = manifest_method("verify", 0);
    verify.return_type = "Boolean".to_string();

    let mut destroy = manifest_method("destroy", 0);
    destroy.return_type = "Void".to_string();

    let methods = vec![
        void_manifest_method("update", 3),
        void_manifest_method("_deploy", 2),
        verify,
        destroy,
    ];

    let manifest = build_manifest("LifecycleLike", &methods).value;
    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supported standards array");
    let standards: Vec<&str> = standards
        .iter()
        .filter_map(|value| value.as_str())
        .collect();

    assert!(standards.contains(&"NEP-22"));
    assert!(standards.contains(&"NEP-29"));
    assert!(standards.contains(&"NEP-30"));
    assert!(standards.contains(&"NEP-31"));
}

#[test]
fn build_manifest_does_not_detect_lifecycle_standards_with_non_void_handlers() {
    let methods = vec![
        manifest_method("update", 3),
        manifest_method("_deploy", 2),
        manifest_method("on_nep11_payment", 4),
        manifest_method("on_nep17_payment", 3),
    ];

    let manifest = build_manifest("LooseLifecycle", &methods).value;
    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supported standards array");
    let standards: Vec<&str> = standards
        .iter()
        .filter_map(|value| value.as_str())
        .collect();

    assert!(!standards.contains(&"NEP-22"));
    assert!(!standards.contains(&"NEP-26"));
    assert!(!standards.contains(&"NEP-27"));
    assert!(!standards.contains(&"NEP-29"));
}

#[test]
fn build_manifest_does_not_detect_lifecycle_standards_with_invalid_parameter_types() {
    let methods = vec![
        typed_manifest_method("update", &["Integer", "Integer", "Integer"], "Void"),
        typed_manifest_method("_deploy", &["Any", "Integer"], "Void"),
        typed_manifest_method(
            "on_nep11_payment",
            &["ByteString", "Boolean", "ByteString", "Any"],
            "Void",
        ),
        typed_manifest_method(
            "on_nep17_payment",
            &["ByteString", "Boolean", "Any"],
            "Void",
        ),
    ];

    let manifest = build_manifest("InvalidLifecycleTypes", &methods).value;
    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supported standards array");
    let standards: Vec<&str> = standards
        .iter()
        .filter_map(|value| value.as_str())
        .collect();

    assert!(!standards.contains(&"NEP-22"));
    assert!(!standards.contains(&"NEP-26"));
    assert!(!standards.contains(&"NEP-27"));
    assert!(!standards.contains(&"NEP-29"));
}

#[test]
fn build_manifest_detects_lifecycle_standards_with_alias_type_spellings() {
    let methods = vec![
        typed_manifest_method("update", &["byte_array", "str", "Map"], "void"),
        typed_manifest_method("_deploy", &["Map", "bool"], "VOID"),
        typed_manifest_method(
            "on_nep11_payment",
            &["UInt160", "int", "buffer", "Map"],
            "Void",
        ),
        typed_manifest_method(
            "on_nep17_payment",
            &["script_hash", "Integer", "Map"],
            "Void",
        ),
    ];

    let manifest = build_manifest("AliasLifecycleTypes", &methods).value;
    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supported standards array");
    let standards: Vec<&str> = standards
        .iter()
        .filter_map(|value| value.as_str())
        .collect();

    assert!(standards.contains(&"NEP-22"));
    assert!(standards.contains(&"NEP-26"));
    assert!(standards.contains(&"NEP-27"));
    assert!(standards.contains(&"NEP-29"));
}

#[test]
fn build_manifest_does_not_over_accept_unknown_alias_type_spellings() {
    let methods = vec![
        typed_manifest_method("update", &["Bytes", "str", "Any"], "Void"),
        typed_manifest_method("on_nep17_payment", &["Hash256", "int", "Any"], "Void"),
    ];

    let manifest = build_manifest("UnknownAliasLifecycleTypes", &methods).value;
    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supported standards array");
    let standards: Vec<&str> = standards
        .iter()
        .filter_map(|value| value.as_str())
        .collect();

    assert!(!standards.contains(&"NEP-22"));
    assert!(!standards.contains(&"NEP-27"));
}

#[test]
fn build_manifest_does_not_detect_nep30_without_boolean_verify() {
    let mut verify = manifest_method("verify", 0);
    verify.return_type = "Void".to_string();

    let manifest = build_manifest("NoVerifyBool", &[verify]).value;
    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supported standards array");
    let standards: Vec<&str> = standards
        .iter()
        .filter_map(|value| value.as_str())
        .collect();

    assert!(
        !standards.contains(&"NEP-30"),
        "verify must return Boolean to satisfy NEP-30"
    );
}

#[test]
fn build_manifest_does_not_detect_nep17_with_invalid_required_arities() {
    let methods = vec![
        typed_manifest_method("symbol", &["Any"], "String"),
        typed_manifest_method("decimals", &[], "Integer"),
        typed_manifest_method("total_supply", &[], "Integer"),
        typed_manifest_method("balance_of", &["Any"], "Integer"),
        typed_manifest_method("transfer", &["Any", "Any"], "Boolean"),
    ];

    let manifest = build_manifest("InvalidNep17Arity", &methods).value;
    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supported standards array");
    let standards: Vec<&str> = standards
        .iter()
        .filter_map(|value| value.as_str())
        .collect();

    assert!(!standards.contains(&"NEP-17"));
}

#[test]
fn build_manifest_does_not_detect_nep11_with_invalid_required_arities() {
    let methods = vec![
        typed_manifest_method("balance_of", &["Any", "Any"], "Integer"),
        typed_manifest_method("owner_of", &[], "Any"),
        typed_manifest_method("transfer", &["Any", "Any"], "Boolean"),
    ];

    let manifest = build_manifest("InvalidNep11Arity", &methods).value;
    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supported standards array");
    let standards: Vec<&str> = standards
        .iter()
        .filter_map(|value| value.as_str())
        .collect();

    assert!(!standards.contains(&"NEP-11"));
}

#[test]
fn build_manifest_detects_nep11() {
    let methods = vec![
        manifest_method("balance_of", 1),
        manifest_method("owner_of", 1),
        manifest_method("tokens_of", 1),
    ];

    let manifest = build_manifest("NftLike", &methods).value;
    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supported standards array");
    let standards: Vec<&str> = standards
        .iter()
        .filter_map(|value| value.as_str())
        .collect();

    assert!(standards.contains(&"NEP-11"));
    assert!(
        !standards.contains(&"NEP-17"),
        "ownerOf signal should prevent NEP-17 detection"
    );
}

#[test]
fn merge_manifest_appends_arrays() {
    let mut base = json!({
        "permissions": [
            {"contract": "0x01", "methods": ["foo"]}
        ],
        "abi": {
            "events": [
                {"name": "First", "parameters": []}
            ]
        }
    });

    let overlay = json!({
        "permissions": [
            {"contract": "0x02", "methods": ["bar"]}
        ],
        "abi": {
            "events": [
                {"name": "Second", "parameters": []}
            ]
        }
    });

    merge_manifest(&mut base, &overlay);

    let permissions = base["permissions"].as_array().unwrap();
    assert_eq!(permissions.len(), 2);
    assert_eq!(permissions[1]["contract"].as_str().unwrap(), "0x02");

    let events = base["abi"]["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1]["name"].as_str().unwrap(), "Second");
}

#[test]
fn merge_manifest_deduplicates_permissions_and_standards() {
    let mut base = json!({
        "permissions": [
            {"contract": "0x01", "methods": ["a", "b"]}
        ],
        "supportedstandards": ["NEP-17"]
    });

    let overlay = json!({
        "permissions": [
            {"contract": "0x01", "methods": ["b", "c"]},
            {"contract": "0x02", "methods": ["d"]}
        ],
        "supportedstandards": ["NEP-17", "NEP-11"]
    });

    merge_manifest(&mut base, &overlay);

    let permissions = base["permissions"].as_array().unwrap();
    assert_eq!(permissions.len(), 2);
    let methods = permissions[0]["methods"].as_array().unwrap();
    let method_set: std::collections::HashSet<_> =
        methods.iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(method_set.len(), 3);
    assert!(method_set.contains("a"));
    assert!(method_set.contains("b"));
    assert!(method_set.contains("c"));

    let standards = base["supportedstandards"].as_array().unwrap();
    assert_eq!(standards.len(), 2);
}

#[test]
fn merge_manifest_permissions_preserve_wildcard_and_extra_fields() {
    let mut base = json!({
        "permissions": [
            {"contract": "0x01", "methods": ["balanceOf"], "note": "base"}
        ]
    });

    let overlay = json!({
        "permissions": [
            {"contract": "0x01", "methods": "*", "author": "neo"},
            {"contract": "0x01", "methods": ["transfer"], "extraField": true}
        ]
    });

    merge_manifest(&mut base, &overlay);

    let permissions = base["permissions"]
        .as_array()
        .expect("permissions should be an array");
    assert_eq!(permissions.len(), 1);

    let permission = &permissions[0];
    assert_eq!(permission["contract"].as_str(), Some("0x01"));
    assert_eq!(permission["methods"].as_str(), Some("*"));
    assert_eq!(permission["note"].as_str(), Some("base"));
    assert_eq!(permission["author"].as_str(), Some("neo"));
    assert_eq!(permission["extraField"].as_bool(), Some(true));
}

#[test]
fn merge_manifest_deduplicates_events() {
    let mut base = json!({
        "abi": {
            "events": [
                {"name": "Transfer", "parameters": [{"name": "from", "type": "ByteArray"}]}
            ]
        }
    });

    let overlay = json!({
        "abi": {
            "events": [
                {"name": "Transfer", "parameters": [{"name": "to", "type": "ByteArray"}], "extra": true},
                {"name": "Approval", "parameters": []}
            ]
        }
    });

    merge_manifest(&mut base, &overlay);

    let events = base["abi"]["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    let transfer = events
        .iter()
        .find(|entry| entry["name"].as_str() == Some("Transfer"))
        .unwrap();
    assert_eq!(transfer["extra"].as_bool(), Some(true));
    assert_eq!(transfer["parameters"].as_array().unwrap().len(), 1);
}

#[test]
fn collect_method_names_extracts_unique_set() {
    let manifest = json!({
        "abi": {
            "methods": [
                {"name": "foo"},
                {"name": "foo"},
                {"name": "bar"}
            ]
        }
    });
    let names = collect_method_names(&manifest);
    assert_eq!(names.len(), 2);
    assert!(names.contains("foo"));
    assert!(names.contains("bar"));
}

#[test]
fn ensure_manifest_methods_catches_added_entries() {
    let manifest = json!({
        "abi": { "methods": [
            {"name": "hello"},
            {"name": "ghost"}
        ]}
    });
    let baseline = HashSet::from(["hello".to_string()]);
    assert!(ensure_manifest_methods_match(&manifest, &baseline, Some("overlay.json")).is_err());
}

#[test]
fn ensure_manifest_methods_catches_removed_entries() {
    let manifest = json!({
        "abi": { "methods": [
            {"name": "hello"}
        ]}
    });
    let baseline = HashSet::from(["hello".to_string(), "extra".to_string()]);
    assert!(ensure_manifest_methods_match(&manifest, &baseline, None).is_err());
}

#[test]
fn ensure_manifest_methods_catches_signature_mutation() {
    let methods = vec![crate::manifest::ManifestMethod {
        name: "foo".to_string(),
        parameters: vec![crate::manifest::ManifestParameter {
            name: "arg0".to_string(),
            kind: "Integer".to_string(),
        }],
        return_type: "Void".to_string(),
        offset: 4,
        safe: false,
    }];

    let mut builder = crate::manifest::ManifestBuilder::new("Contract", &methods);
    let overlay = json!({
        "abi": {
            "methods": [{
                "name": "foo",
                "parameters": [],
                "returntype": "Any",
                "offset": 8
            }]
        }
    });
    builder.merge_overlay(&overlay, Some("overlay.json".to_string()));
    builder.propagate_safe_flags();
    let err = builder.ensure_method_parity().unwrap_err();
    assert!(err
        .to_string()
        .contains("mutated ABI arity or offsets for existing methods"));
}

#[test]
fn ensure_manifest_methods_allows_type_overrides() {
    let methods = vec![crate::manifest::ManifestMethod {
        name: "foo".to_string(),
        parameters: vec![crate::manifest::ManifestParameter {
            name: "arg0".to_string(),
            kind: "Integer".to_string(),
        }],
        return_type: "Integer".to_string(),
        offset: 4,
        safe: false,
    }];

    let mut builder = crate::manifest::ManifestBuilder::new("Contract", &methods);
    let overlay = json!({
        "abi": {
            "methods": [{
                "name": "foo",
                "parameters": [{"name": "arg0", "type": "ByteString"}],
                "returntype": "Boolean"
            }]
        }
    });
    builder.merge_overlay(&overlay, Some("overlay.json".to_string()));
    builder.propagate_safe_flags();
    builder
        .ensure_method_parity()
        .expect("parameter/return types may be overridden when arity and offsets match");
}

#[test]
fn ensure_manifest_methods_rejects_offset_out_of_u32_range() {
    let methods = vec![crate::manifest::ManifestMethod {
        name: "foo".to_string(),
        parameters: vec![],
        return_type: "Void".to_string(),
        offset: 4,
        safe: false,
    }];

    let mut builder = crate::manifest::ManifestBuilder::new("Contract", &methods);
    let overlay = json!({
        "abi": {
            "methods": [{
                "name": "foo",
                "parameters": [],
                "returntype": "Void",
                "offset": 4294967296u64
            }]
        }
    });
    builder.merge_overlay(&overlay, Some("overlay.json".to_string()));

    let err = builder.ensure_method_parity().unwrap_err();
    assert!(err.to_string().contains("offset exceeds u32 range"));
}

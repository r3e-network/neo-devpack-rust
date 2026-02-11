use super::{collect_method_names, ensure_manifest_methods_match, merge_manifest};
use serde_json::json;
use std::collections::HashSet;

#[test]
fn merge_manifest_deeply_combines_objects() {
    let mut base = json!({
        "abi": {
            "methods": ["foo"],
            "events": []
        },
        "extra": {
            "author": "neo-llvm"
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
    assert!(err
        .to_string()
        .contains("offset exceeds u32 range"));
}

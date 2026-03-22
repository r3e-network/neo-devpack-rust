// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Extended manifest merge edge case tests

use serde_json::json;
use wasm_neovm::manifest::{merge_manifest, propagate_safe_flags};

fn merge(base: &mut serde_json::Value, overlay: &serde_json::Value) {
    merge_manifest(base, overlay);
}

// ============================================================================
// Scalar override tests
// ============================================================================

#[test]
fn merge_scalar_override() {
    let mut base = json!({"name": "old"});
    let overlay = json!({"name": "new"});
    merge(&mut base, &overlay);
    assert_eq!(base["name"].as_str().unwrap(), "new");
}

#[test]
fn merge_number_override() {
    let mut base = json!(42);
    let overlay = json!(99);
    merge(&mut base, &overlay);
    assert_eq!(base.as_i64().unwrap(), 99);
}

#[test]
fn merge_bool_override() {
    let mut base = json!(false);
    let overlay = json!(true);
    merge(&mut base, &overlay);
    assert!(base.as_bool().unwrap());
}

#[test]
fn merge_null_override() {
    let mut base = json!("something");
    let overlay = json!(null);
    merge(&mut base, &overlay);
    assert!(base.is_null());
}

// ============================================================================
// Array merging
// ============================================================================

#[test]
fn merge_arrays_concatenate() {
    let mut base = json!([1, 2]);
    let overlay = json!([3, 4]);
    merge(&mut base, &overlay);
    assert_eq!(base.as_array().unwrap().len(), 4);
}

#[test]
fn merge_empty_array_with_non_empty() {
    let mut base = json!([]);
    let overlay = json!([1, 2, 3]);
    merge(&mut base, &overlay);
    assert_eq!(base.as_array().unwrap().len(), 3);
}

#[test]
fn merge_non_empty_with_empty_array() {
    let mut base = json!([1, 2]);
    let overlay = json!([]);
    merge(&mut base, &overlay);
    assert_eq!(base.as_array().unwrap().len(), 2);
}

// ============================================================================
// Object merging
// ============================================================================

#[test]
fn merge_nested_objects_deeply() {
    let mut base = json!({
        "a": {
            "b": {
                "c": 1,
                "d": 2
            }
        }
    });
    let overlay = json!({
        "a": {
            "b": {
                "c": 99,
                "e": 3
            }
        }
    });
    merge(&mut base, &overlay);
    assert_eq!(base["a"]["b"]["c"].as_i64().unwrap(), 99);
    assert_eq!(base["a"]["b"]["d"].as_i64().unwrap(), 2);
    assert_eq!(base["a"]["b"]["e"].as_i64().unwrap(), 3);
}

#[test]
fn merge_adds_new_keys() {
    let mut base = json!({"existing": true});
    let overlay = json!({"new_key": "value"});
    merge(&mut base, &overlay);
    assert!(base["existing"].as_bool().unwrap());
    assert_eq!(base["new_key"].as_str().unwrap(), "value");
}

#[test]
fn merge_empty_object_with_non_empty() {
    let mut base = json!({});
    let overlay = json!({"key": "value"});
    merge(&mut base, &overlay);
    assert_eq!(base["key"].as_str().unwrap(), "value");
}

// ============================================================================
// Type mismatch behavior
// ============================================================================

#[test]
fn merge_object_with_scalar_overlay_replaces() {
    let mut base = json!({"key": {"nested": true}});
    let overlay = json!({"key": "flat"});
    merge(&mut base, &overlay);
    assert_eq!(base["key"].as_str().unwrap(), "flat");
}

#[test]
fn merge_scalar_with_object_overlay_replaces() {
    let mut base = json!({"key": "flat"});
    let overlay = json!({"key": {"nested": true}});
    merge(&mut base, &overlay);
    assert!(base["key"]["nested"].as_bool().unwrap());
}

#[test]
fn merge_array_with_object_replaces() {
    let mut base = json!([1, 2, 3]);
    let overlay = json!({"key": "value"});
    merge(&mut base, &overlay);
    assert!(base.is_object());
}

// ============================================================================
// Dedup supportedstandards
// ============================================================================

#[test]
fn merge_deduplicates_supported_standards() {
    let mut base = json!({
        "supportedstandards": ["NEP-17", "NEP-11"]
    });
    let overlay = json!({
        "supportedstandards": ["NEP-11", "NEP-24"]
    });
    merge(&mut base, &overlay);
    let standards = base["supportedstandards"].as_array().unwrap();
    assert_eq!(standards.len(), 3);
}

// ============================================================================
// Dedup trusts
// ============================================================================

#[test]
fn merge_deduplicates_trusts() {
    let mut base = json!({
        "trusts": ["0xabc", "0xdef"]
    });
    let overlay = json!({
        "trusts": ["0xabc", "0x123"]
    });
    merge(&mut base, &overlay);
    let trusts = base["trusts"].as_array().unwrap();
    assert_eq!(trusts.len(), 3); // 0xabc, 0xdef, 0x123
}

// ============================================================================
// Propagate safe flags
// ============================================================================

#[test]
fn propagate_safe_marks_all_occurrences() {
    let mut manifest = json!({
        "abi": {
            "methods": [
                {"name": "balanceOf", "safe": true},
                {"name": "balanceOf"},
                {"name": "transfer"}
            ]
        }
    });
    propagate_safe_flags(&mut manifest);

    let methods = manifest["abi"]["methods"].as_array().unwrap();
    assert!(methods[0]["safe"].as_bool().unwrap());
    assert!(methods[1]["safe"].as_bool().unwrap());
    assert!(methods[2].get("safe").is_none() || methods[2]["safe"].as_bool() == Some(false));
}

#[test]
fn propagate_safe_noop_when_no_safe() {
    let mut manifest = json!({
        "abi": {
            "methods": [
                {"name": "transfer"},
                {"name": "balanceOf"}
            ]
        }
    });
    propagate_safe_flags(&mut manifest);
    let methods = manifest["abi"]["methods"].as_array().unwrap();
    assert!(methods[0].get("safe").is_none());
    assert!(methods[1].get("safe").is_none());
}

#[test]
fn propagate_safe_noop_without_abi() {
    let mut manifest = json!({"name": "test"});
    propagate_safe_flags(&mut manifest);
    assert!(manifest.get("abi").is_none());
}

#[test]
fn propagate_safe_noop_without_methods() {
    let mut manifest = json!({"abi": {"events": []}});
    propagate_safe_flags(&mut manifest);
    assert!(manifest["abi"].get("methods").is_none());
}

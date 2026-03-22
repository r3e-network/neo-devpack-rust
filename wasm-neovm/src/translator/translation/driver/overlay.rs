// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

pub(super) fn collect_safe_methods(value: &Value, accumulator: &mut HashSet<String>) {
    match value {
        Value::Object(map) => {
            if let Some(methods) = map
                .get("abi")
                .and_then(Value::as_object)
                .and_then(|abi| abi.get("methods"))
                .and_then(Value::as_array)
            {
                for method in methods {
                    if method.get("safe").and_then(Value::as_bool).unwrap_or(false) {
                        if let Some(name) = method.get("name").and_then(Value::as_str) {
                            accumulator.insert(name.to_string());
                        }
                    }
                }
            }
            for child in map.values() {
                collect_safe_methods(child, accumulator);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_safe_methods(item, accumulator);
            }
        }
        _ => {}
    }
}

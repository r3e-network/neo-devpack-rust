use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub fn merge_manifest(base: &mut Value, overlay: &Value) {
    if let (Some(base_map), Some(overlay_map)) = (base.as_object_mut(), overlay.as_object()) {
        for (key, value) in overlay_map {
            match base_map.get_mut(key) {
                Some(existing) => merge_manifest(existing, value),
                None => {
                    base_map.insert(key.clone(), value.clone());
                }
            }
        }
        dedup_manifest_collections(base_map);
        return;
    }

    if let (Some(base_arr), Some(overlay_arr)) = (base.as_array_mut(), overlay.as_array()) {
        base_arr.extend(overlay_arr.iter().cloned());
        return;
    }

    *base = overlay.clone();
}

pub fn propagate_safe_flags(manifest: &mut Value) {
    let Some(abi) = manifest
        .get_mut("abi")
        .and_then(|value| value.as_object_mut())
    else {
        return;
    };
    let Some(methods) = abi
        .get_mut("methods")
        .and_then(|methods| methods.as_array_mut())
    else {
        return;
    };

    let mut safe_names: HashSet<String> = HashSet::new();
    for method in methods.iter() {
        if method
            .get("safe")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        {
            if let Some(name) = method.get("name").and_then(serde_json::Value::as_str) {
                safe_names.insert(name.to_string());
            }
        }
    }

    if safe_names.is_empty() {
        return;
    }

    for method in methods.iter_mut() {
        if let Some(name) = method.get("name").and_then(serde_json::Value::as_str) {
            if safe_names.contains(name) {
                if let Some(obj) = method.as_object_mut() {
                    obj.insert("safe".to_string(), serde_json::Value::Bool(true));
                }
            }
        }
    }
}

fn dedup_manifest_collections(map: &mut serde_json::Map<String, Value>) {
    if let Some(value) = map.get_mut("supportedstandards") {
        dedup_string_array(value);
    }
    if let Some(value) = map.get_mut("trusts") {
        dedup_string_array(value);
    }
    if let Some(value) = map.get_mut("permissions") {
        dedup_permissions(value);
    }
    if let Some(extra) = map.get_mut("abi").and_then(|abi| abi.as_object_mut()) {
        if let Some(methods) = extra.get_mut("methods") {
            dedup_method_offsets(methods);
        }
        if let Some(events) = extra.get_mut("events") {
            dedup_events(events);
        }
    }
}

fn dedup_string_array(value: &mut Value) {
    if let Some(array) = value.as_array_mut() {
        let mut seen = HashSet::new();
        array.retain(|item| {
            if let Some(s) = item.as_str() {
                seen.insert(s.to_string())
            } else {
                true
            }
        });
    }
}

fn dedup_permissions(value: &mut Value) {
    let Some(array) = value.as_array_mut() else {
        return;
    };

    let mut merged = Vec::new();
    let mut index_by_contract: HashMap<String, usize> = HashMap::new();
    let mut fallback_seen: HashSet<String> = HashSet::new();

    for mut entry in array.drain(..) {
        if let Some(contract) = entry
            .get("contract")
            .and_then(serde_json::Value::as_str)
            .map(|s| s.to_string())
        {
            let position = index_by_contract
                .entry(contract.clone())
                .or_insert_with(|| {
                    merged.push(serde_json::json!({
                        "contract": contract,
                        "methods": []
                    }));
                    merged.len() - 1
                });

            if let Some(methods) = entry.get_mut("methods") {
                merge_methods(&mut merged[*position], methods);
            }
        } else {
            let key = entry.to_string();
            if fallback_seen.insert(key) {
                merged.push(entry);
            }
        }
    }

    *array = merged;
}

fn merge_methods(target: &mut Value, incoming: &Value) {
    let Some(target_obj) = target.as_object_mut() else {
        return;
    };
    let Some(target_methods) = target_obj.get_mut("methods") else {
        return;
    };
    if !target_methods.is_array() {
        *target_methods = serde_json::Value::Array(Vec::new());
    }
    if let Some(array) = target_methods.as_array_mut() {
        if let Some(incoming_array) = incoming.as_array() {
            let mut seen: HashSet<String> = array
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            for method in incoming_array {
                if let Some(name) = method.as_str() {
                    if seen.insert(name.to_string()) {
                        array.push(serde_json::Value::String(name.to_string()));
                    }
                }
            }
        }
    }
}

fn dedup_method_offsets(value: &mut Value) {
    if let Some(items) = value.as_array_mut() {
        let mut merged: Vec<serde_json::Value> = Vec::new();
        let mut index_by_name: HashMap<String, usize> = HashMap::new();

        for entry in items.drain(..) {
            let method_name = entry
                .get("name")
                .and_then(serde_json::Value::as_str)
                .map(|s| s.to_string());

            if let Some(name) = method_name {
                if let Some(existing_index) = index_by_name.get(&name).copied() {
                    if let (Some(existing), Some(overlay)) =
                        (merged[existing_index].as_object_mut(), entry.as_object())
                    {
                        for (key, value) in overlay {
                            existing.insert(key.clone(), value.clone());
                        }
                    }
                    continue;
                }

                index_by_name.insert(name, merged.len());
                merged.push(entry);
            } else {
                merged.push(entry);
            }
        }

        *items = merged;
    }
}

fn dedup_events(value: &mut Value) {
    if let Some(items) = value.as_array_mut() {
        let mut merged: Vec<serde_json::Value> = Vec::new();
        let mut index_by_name: HashMap<String, usize> = HashMap::new();

        for entry in items.drain(..) {
            let event_name = entry
                .get("name")
                .and_then(serde_json::Value::as_str)
                .map(|s| s.to_string());

            if let Some(name) = event_name {
                if let Some(existing_index) = index_by_name.get(&name).copied() {
                    if let (Some(existing), Some(overlay)) =
                        (merged[existing_index].as_object_mut(), entry.as_object())
                    {
                        for (key, value) in overlay {
                            existing.insert(key.clone(), value.clone());
                        }
                    }
                    continue;
                }

                index_by_name.insert(name, merged.len());
                merged.push(entry);
            } else {
                merged.push(entry);
            }
        }

        *items = merged;
    }
}

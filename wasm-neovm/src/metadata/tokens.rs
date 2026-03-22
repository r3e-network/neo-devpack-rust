// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use std::collections::HashSet;

use serde_json::{Map, Value};

use super::{
    TOKEN_CALLFLAGS_KEY, TOKEN_HASH_KEY, TOKEN_HAS_RETURN_KEY, TOKEN_METHOD_KEY,
    TOKEN_PARAMCOUNT_KEY,
};
use crate::nef::MethodToken;

/// Serialize method tokens to a JSON array value.
pub fn method_tokens_to_json(tokens: &[MethodToken]) -> Value {
    let entries: Vec<Value> = tokens
        .iter()
        .map(|token| {
            let mut obj = Map::new();
            obj.insert(
                TOKEN_HASH_KEY.to_string(),
                Value::String(format!("0x{}", hex::encode(token.contract_hash))),
            );
            obj.insert(
                TOKEN_METHOD_KEY.to_string(),
                Value::String(token.method.clone()),
            );
            obj.insert(
                TOKEN_PARAMCOUNT_KEY.to_string(),
                Value::Number(token.parameters_count.into()),
            );
            obj.insert(
                TOKEN_HAS_RETURN_KEY.to_string(),
                Value::Bool(token.has_return_value),
            );
            obj.insert(
                TOKEN_CALLFLAGS_KEY.to_string(),
                Value::Number(token.call_flags.into()),
            );
            Value::Object(obj)
        })
        .collect();
    Value::Array(entries)
}

/// Deduplicates method tokens while preserving order.
///
/// This function removes duplicate method tokens based on their contract hash,
/// method name, parameters count, return value flag, and call flags.
pub fn dedup_method_tokens(tokens: &mut Vec<MethodToken>) {
    let mut seen = HashSet::with_capacity(tokens.len());
    tokens.retain(|token| {
        // Use the method string's hash instead of the string itself to avoid clones
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        token.method.hash(&mut hasher);
        let method_hash = hasher.finish();

        let key = (
            token.contract_hash,
            method_hash,
            token.parameters_count,
            token.has_return_value,
            token.call_flags,
        );
        seen.insert(key)
    });
}

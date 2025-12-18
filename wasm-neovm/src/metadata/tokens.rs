use std::collections::HashSet;

use serde_json::{Map, Value};

use super::{
    TOKEN_CALLFLAGS_KEY, TOKEN_HASH_KEY, TOKEN_HAS_RETURN_KEY, TOKEN_METHOD_KEY,
    TOKEN_PARAMCOUNT_KEY,
};
use crate::nef::MethodToken;

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

pub fn dedup_method_tokens(tokens: &mut Vec<MethodToken>) {
    let mut seen = HashSet::new();
    tokens.retain(|token| {
        seen.insert((
            token.contract_hash,
            token.method.clone(),
            token.parameters_count,
            token.has_return_value,
            token.call_flags,
        ))
    });
}

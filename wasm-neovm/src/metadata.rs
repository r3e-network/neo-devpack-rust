use std::collections::HashSet;

use anyhow::{anyhow, bail, ensure, Context, Result};
use serde_json::{json, Map, Value};

use crate::nef::{MethodToken, HASH160_LENGTH};

pub const TOKEN_COLLECTION_KEY: &str = "nefMethodTokens";
pub const TOKEN_HASH_KEY: &str = "hash";
pub const TOKEN_METHOD_KEY: &str = "method";
pub const TOKEN_PARAMCOUNT_KEY: &str = "paramcount";
pub const TOKEN_HAS_RETURN_KEY: &str = "hasreturnvalue";
pub const TOKEN_CALLFLAGS_KEY: &str = "callflags";
pub const SOURCE_TOP_LEVEL_KEY: &str = "source";
pub const SOURCE_EXTRA_KEY: &str = "nefSource";

#[derive(Debug, Default, Clone)]
pub struct NefMetadata {
    pub source: Option<String>,
    pub method_tokens: Vec<MethodToken>,
}

pub fn extract_nef_metadata(manifest: &Value) -> Result<NefMetadata> {
    let source = extract_source(manifest);
    let method_tokens = extract_method_tokens(manifest)?;
    Ok(NefMetadata {
        source,
        method_tokens,
    })
}

pub fn parse_method_token_section(bytes: &[u8]) -> Result<NefMetadata> {
    let value: Value = serde_json::from_slice(bytes)
        .context("failed to parse neo.methodtokens custom section as JSON")?;

    match value {
        Value::Array(array) => {
            let tokens = parse_method_token_array(&array, "neo.methodtokens section")?;
            Ok(NefMetadata {
                source: None,
                method_tokens: tokens,
            })
        }
        Value::Object(ref map) => {
            let source = map
                .get(SOURCE_EXTRA_KEY)
                .or_else(|| map.get(SOURCE_TOP_LEVEL_KEY))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);

            let tokens = if let Some(value) = map.get(TOKEN_COLLECTION_KEY) {
                parse_method_tokens_value(value, "neo.methodtokens section")?
            } else if let Some(value) = map.get("tokens") {
                parse_method_tokens_value(value, "neo.methodtokens section")?
            } else if has_token_fields(map) {
                vec![parse_method_token_object(
                    map,
                    "neo.methodtokens section",
                    0,
                )?]
            } else {
                Vec::new()
            };

            Ok(NefMetadata {
                source,
                method_tokens: tokens,
            })
        }
        other => bail!(
            "neo.methodtokens custom section must be a JSON array or object (found {:?})",
            other
        ),
    }
}

pub fn update_manifest_metadata(
    manifest: &mut Value,
    source: Option<&str>,
    tokens: &[MethodToken],
) -> Result<()> {
    let manifest_obj = manifest
        .as_object_mut()
        .ok_or_else(|| anyhow!("manifest JSON must be an object"))?;

    if let Some(src) = source {
        let extra = ensure_extra_object(manifest_obj)?;
        extra.insert(SOURCE_EXTRA_KEY.to_string(), Value::String(src.to_string()));
    }

    if !tokens.is_empty() {
        let extra = ensure_extra_object(manifest_obj)?;
        extra.insert(
            TOKEN_COLLECTION_KEY.to_string(),
            method_tokens_to_json(tokens),
        );
    }

    Ok(())
}

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

fn extract_source(manifest: &Value) -> Option<String> {
    manifest
        .get(SOURCE_TOP_LEVEL_KEY)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            manifest
                .get("extra")
                .and_then(Value::as_object)
                .and_then(|extra| extra.get(SOURCE_EXTRA_KEY))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
}

fn extract_method_tokens(manifest: &Value) -> Result<Vec<MethodToken>> {
    let extra = match manifest.get("extra") {
        Some(Value::Object(obj)) => obj,
        Some(_) => bail!("manifest 'extra' field must be an object when present"),
        None => return Ok(Vec::new()),
    };

    let raw_tokens = match extra.get(TOKEN_COLLECTION_KEY) {
        Some(value) => value,
        None => return Ok(Vec::new()),
    };

    parse_method_tokens_value(raw_tokens, "manifest extra")
}

fn parse_method_tokens_value(value: &Value, context: &str) -> Result<Vec<MethodToken>> {
    match value {
        Value::Array(array) => parse_method_token_array(array, context),
        Value::Object(obj) => Ok(vec![parse_method_token_object(obj, context, 0)?]),
        other => bail!(
            "{} must be an array or object describing method tokens (found {:?})",
            context,
            other
        ),
    }
}

fn parse_method_token_array(array: &[Value], context: &str) -> Result<Vec<MethodToken>> {
    let mut tokens = Vec::with_capacity(array.len());
    for (index, entry) in array.iter().enumerate() {
        let obj = entry
            .as_object()
            .ok_or_else(|| anyhow!("{} entry #{} must be a JSON object", context, index))?;
        tokens.push(parse_method_token_object(obj, context, index)?);
    }
    Ok(tokens)
}

fn parse_method_token_object(
    obj: &Map<String, Value>,
    context: &str,
    index: usize,
) -> Result<MethodToken> {
    let hash_str = obj
        .get(TOKEN_HASH_KEY)
        .and_then(Value::as_str)
        .ok_or_else(|| {
            anyhow!(
                "{} method token #{} missing '{}' field",
                context,
                index,
                TOKEN_HASH_KEY
            )
        })?;
    let contract_hash = parse_hash160(hash_str).with_context(|| {
        format!(
            "{} method token #{} has invalid '{}'",
            context, index, TOKEN_HASH_KEY
        )
    })?;

    let method = obj
        .get(TOKEN_METHOD_KEY)
        .and_then(Value::as_str)
        .ok_or_else(|| {
            anyhow!(
                "{} method token #{} missing '{}' field",
                context,
                index,
                TOKEN_METHOD_KEY
            )
        })?
        .to_owned();

    let paramcount = obj
        .get(TOKEN_PARAMCOUNT_KEY)
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            anyhow!(
                "{} method token #{} missing '{}' field",
                context,
                index,
                TOKEN_PARAMCOUNT_KEY
            )
        })?;
    ensure!(
        paramcount <= u16::MAX as u64,
        "{} method token #{} has parameter count exceeding u16 range",
        context,
        index
    );

    let has_return_value = obj
        .get(TOKEN_HAS_RETURN_KEY)
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            anyhow!(
                "{} method token #{} missing '{}' field",
                context,
                index,
                TOKEN_HAS_RETURN_KEY
            )
        })?;

    let call_flags = obj
        .get(TOKEN_CALLFLAGS_KEY)
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            anyhow!(
                "{} method token #{} missing '{}' field",
                context,
                index,
                TOKEN_CALLFLAGS_KEY
            )
        })?;
    ensure!(
        call_flags <= u8::MAX as u64,
        "{} method token #{} call flags overflow u8 range",
        context,
        index
    );

    Ok(MethodToken {
        contract_hash,
        method,
        parameters_count: paramcount as u16,
        has_return_value,
        call_flags: call_flags as u8,
    })
}

fn has_token_fields(obj: &Map<String, Value>) -> bool {
    obj.contains_key(TOKEN_HASH_KEY)
        && obj.contains_key(TOKEN_METHOD_KEY)
        && obj.contains_key(TOKEN_PARAMCOUNT_KEY)
        && obj.contains_key(TOKEN_HAS_RETURN_KEY)
        && obj.contains_key(TOKEN_CALLFLAGS_KEY)
}

fn ensure_extra_object<'a>(
    manifest_obj: &'a mut Map<String, Value>,
) -> Result<&'a mut Map<String, Value>> {
    if !manifest_obj.contains_key("extra") {
        manifest_obj.insert("extra".to_string(), json!({}));
    }
    manifest_obj
        .get_mut("extra")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| anyhow!("manifest 'extra' field must be a JSON object"))
}

pub(crate) fn parse_hash160(input: &str) -> Result<[u8; HASH160_LENGTH]> {
    let trimmed = input.strip_prefix("0x").unwrap_or(input);
    ensure!(
        trimmed.len() == HASH160_LENGTH * 2,
        "Hash160 strings must contain exactly {} hexadecimal characters",
        HASH160_LENGTH * 2
    );
    let bytes = hex::decode(trimmed)?;
    let mut array = [0u8; HASH160_LENGTH];
    array.copy_from_slice(&bytes);
    Ok(array)
}

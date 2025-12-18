use anyhow::{anyhow, bail, ensure, Context, Result};
use serde_json::{Map, Value};

use super::{
    NefMetadata, SOURCE_EXTRA_KEY, SOURCE_TOP_LEVEL_KEY, TOKEN_CALLFLAGS_KEY, TOKEN_COLLECTION_KEY,
    TOKEN_HASH_KEY, TOKEN_HAS_RETURN_KEY, TOKEN_METHOD_KEY, TOKEN_PARAMCOUNT_KEY,
};
use crate::nef::{MethodToken, HASH160_LENGTH};

const MAX_TOKEN_METHOD_LENGTH: usize = 32;

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

pub(super) fn parse_method_tokens_value(value: &Value, context: &str) -> Result<Vec<MethodToken>> {
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
    ensure!(
        method.len() <= MAX_TOKEN_METHOD_LENGTH,
        "{} method token #{} has method name exceeding {} bytes",
        context,
        index,
        MAX_TOKEN_METHOD_LENGTH
    );

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

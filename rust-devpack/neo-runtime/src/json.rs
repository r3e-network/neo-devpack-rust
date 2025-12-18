use neo_types::*;
use num_bigint::BigInt;
use serde_json::{json, Value as JsonValue};
use std::vec::Vec;

/// Minimal JSON helpers to support tests.
pub struct NeoJSON;

impl NeoJSON {
    pub fn serialize(value: &NeoValue) -> NeoResult<NeoString> {
        let json_value = Self::value_to_json(value)?;
        let text = serde_json::to_string(&json_value)
            .map_err(|err| NeoError::new(&format!("failed to serialize JSON: {err}")))?;
        Ok(NeoString::from_str(&text))
    }

    pub fn deserialize(json: &NeoString) -> NeoResult<NeoValue> {
        let parsed: JsonValue = serde_json::from_str(json.as_str())
            .map_err(|err| NeoError::new(&format!("failed to parse JSON: {err}")))?;
        Self::json_to_value(&parsed)
    }

    fn value_to_json(value: &NeoValue) -> NeoResult<JsonValue> {
        match value {
            NeoValue::Integer(i) => Ok(json!({
                "type": "Integer",
                "value": i.as_bigint().to_string()
            })),
            NeoValue::Boolean(b) => Ok(json!({
                "type": "Boolean",
                "value": b.as_bool()
            })),
            NeoValue::String(s) => Ok(json!({
                "type": "String",
                "value": s.as_str()
            })),
            NeoValue::ByteString(bs) => {
                let bytes: Vec<JsonValue> =
                    bs.as_slice().iter().map(|b| JsonValue::from(*b)).collect();
                Ok(json!({ "type": "ByteString", "value": bytes }))
            }
            NeoValue::Array(arr) => {
                let mut values = Vec::new();
                for item in arr.iter() {
                    values.push(Self::value_to_json(item)?);
                }
                Ok(json!({ "type": "Array", "value": values }))
            }
            NeoValue::Map(map) => {
                let mut entries = Vec::new();
                for (key, value) in map.iter() {
                    entries.push(json!({
                        "key": Self::value_to_json(key)?,
                        "value": Self::value_to_json(value)?,
                    }));
                }
                Ok(json!({ "type": "Map", "value": entries }))
            }
            NeoValue::Struct(st) => {
                let mut fields_json = Vec::new();
                for (name, value) in st.iter() {
                    fields_json.push(json!({
                        "name": name,
                        "value": Self::value_to_json(value)?,
                    }));
                }
                Ok(json!({ "type": "Struct", "value": fields_json }))
            }
            NeoValue::Null => Ok(json!({ "type": "Null" })),
        }
    }

    fn json_to_value(json: &JsonValue) -> NeoResult<NeoValue> {
        let obj = json.as_object().ok_or_else(|| NeoError::InvalidType)?;
        let kind = obj
            .get("type")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| NeoError::InvalidType)?;

        match kind {
            "Integer" => {
                let value_str = obj
                    .get("value")
                    .and_then(JsonValue::as_str)
                    .ok_or_else(|| NeoError::InvalidType)?;
                let bigint = BigInt::parse_bytes(value_str.as_bytes(), 10)
                    .ok_or_else(|| NeoError::InvalidType)?;
                Ok(NeoValue::from(NeoInteger::new(bigint)))
            }
            "Boolean" => {
                let value = obj
                    .get("value")
                    .and_then(JsonValue::as_bool)
                    .ok_or_else(|| NeoError::InvalidType)?;
                Ok(NeoValue::from(NeoBoolean::new(value)))
            }
            "String" => {
                let value = obj
                    .get("value")
                    .and_then(JsonValue::as_str)
                    .ok_or_else(|| NeoError::InvalidType)?;
                Ok(NeoValue::from(NeoString::from_str(value)))
            }
            "ByteString" => {
                let array = obj
                    .get("value")
                    .and_then(JsonValue::as_array)
                    .ok_or_else(|| NeoError::InvalidType)?;
                let mut bytes = Vec::with_capacity(array.len());
                for item in array {
                    let byte = item.as_u64().ok_or_else(|| NeoError::InvalidType)?;
                    bytes.push(byte as u8);
                }
                Ok(NeoValue::from(NeoByteString::new(bytes)))
            }
            "Array" => {
                let array = obj
                    .get("value")
                    .and_then(JsonValue::as_array)
                    .ok_or_else(|| NeoError::InvalidType)?;
                let mut values = NeoArray::new();
                for item in array {
                    values.push(Self::json_to_value(item)?);
                }
                Ok(NeoValue::from(values))
            }
            "Map" => {
                let entries = obj
                    .get("value")
                    .and_then(JsonValue::as_array)
                    .ok_or_else(|| NeoError::InvalidType)?;
                let mut map = NeoMap::new();
                for entry in entries {
                    let entry_obj = entry.as_object().ok_or_else(|| NeoError::InvalidType)?;
                    let key_json = entry_obj.get("key").ok_or_else(|| NeoError::InvalidType)?;
                    let value_json = entry_obj
                        .get("value")
                        .ok_or_else(|| NeoError::InvalidType)?;
                    let key = Self::json_to_value(key_json)?;
                    let value = Self::json_to_value(value_json)?;
                    map.insert(key, value);
                }
                Ok(NeoValue::from(map))
            }
            "Struct" => {
                let fields = obj
                    .get("value")
                    .and_then(JsonValue::as_array)
                    .ok_or_else(|| NeoError::InvalidType)?;
                let mut result = NeoStruct::new();
                for field in fields {
                    let field_obj = field.as_object().ok_or_else(|| NeoError::InvalidType)?;
                    let name = field_obj
                        .get("name")
                        .and_then(JsonValue::as_str)
                        .ok_or_else(|| NeoError::InvalidType)?;
                    let value_json = field_obj
                        .get("value")
                        .ok_or_else(|| NeoError::InvalidType)?;
                    let value = Self::json_to_value(value_json)?;
                    result = result.with_field(name, value);
                }
                Ok(NeoValue::from(result))
            }
            "Null" => Ok(NeoValue::Null),
            _ => Err(NeoError::InvalidType),
        }
    }
}


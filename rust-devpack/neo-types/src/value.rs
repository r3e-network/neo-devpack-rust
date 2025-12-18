use std::string::String;
use std::vec::Vec;

use crate::array::NeoArray;
use crate::boolean::NeoBoolean;
use crate::bytestring::NeoByteString;
use crate::integer::NeoInteger;
use crate::map::NeoMap;
use crate::string::NeoString;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Neo N3 Struct type
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NeoStruct {
    fields: Vec<(String, NeoValue)>,
}

impl NeoStruct {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn with_field(mut self, name: &str, value: NeoValue) -> Self {
        self.fields.push((name.to_string(), value));
        self
    }

    pub fn get_field(&self, name: &str) -> Option<&NeoValue> {
        for (field_name, value) in &self.fields {
            if field_name == name {
                return Some(value);
            }
        }
        None
    }

    pub fn set_field(&mut self, name: &str, value: NeoValue) {
        for (field_name, field_value) in &mut self.fields {
            if field_name == name {
                *field_value = value;
                return;
            }
        }
        self.fields.push((name.to_string(), value));
    }

    pub fn insert(&mut self, name: NeoString, value: NeoValue) {
        self.set_field(name.as_str(), value);
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &NeoValue)> {
        self.fields
            .iter()
            .map(|(name, value)| (name.as_str(), value))
    }
}

impl Default for NeoStruct {
    fn default() -> Self {
        Self::new()
    }
}

/// Neo N3 Value type (union of all Neo types)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum NeoValue {
    Integer(NeoInteger),
    Boolean(NeoBoolean),
    ByteString(NeoByteString),
    String(NeoString),
    Array(NeoArray<NeoValue>),
    Map(NeoMap<NeoValue, NeoValue>),
    Struct(NeoStruct),
    Null,
}

impl NeoValue {
    pub fn is_null(&self) -> bool {
        matches!(self, NeoValue::Null)
    }

    pub fn as_integer(&self) -> Option<NeoInteger> {
        match self {
            NeoValue::Integer(i) => Some(i.clone()),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<NeoBoolean> {
        match self {
            NeoValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_byte_string(&self) -> Option<&NeoByteString> {
        match self {
            NeoValue::ByteString(bs) => Some(bs),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&NeoString> {
        match self {
            NeoValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&NeoArray<NeoValue>> {
        match self {
            NeoValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&NeoMap<NeoValue, NeoValue>> {
        match self {
            NeoValue::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_struct(&self) -> Option<&NeoStruct> {
        match self {
            NeoValue::Struct(s) => Some(s),
            _ => None,
        }
    }
}

impl From<NeoInteger> for NeoValue {
    fn from(value: NeoInteger) -> Self {
        NeoValue::Integer(value)
    }
}

impl From<NeoBoolean> for NeoValue {
    fn from(value: NeoBoolean) -> Self {
        NeoValue::Boolean(value)
    }
}

impl From<NeoByteString> for NeoValue {
    fn from(value: NeoByteString) -> Self {
        NeoValue::ByteString(value)
    }
}

impl From<NeoString> for NeoValue {
    fn from(value: NeoString) -> Self {
        NeoValue::String(value)
    }
}

impl From<NeoArray<NeoValue>> for NeoValue {
    fn from(value: NeoArray<NeoValue>) -> Self {
        NeoValue::Array(value)
    }
}

impl From<NeoMap<NeoValue, NeoValue>> for NeoValue {
    fn from(value: NeoMap<NeoValue, NeoValue>) -> Self {
        NeoValue::Map(value)
    }
}

impl From<NeoStruct> for NeoValue {
    fn from(value: NeoStruct) -> Self {
        NeoValue::Struct(value)
    }
}


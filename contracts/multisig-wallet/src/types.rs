use neo_devpack::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub struct WalletConfig {
    pub owners: Vec<NeoByteString>,
    pub threshold: i64,
}

#[derive(Clone)]
pub struct Proposal {
    pub proposer: NeoByteString,
    pub target: NeoByteString,
    pub method: String,
    pub arguments: Vec<CallArgument>,
    pub approvals: Vec<NeoByteString>,
    pub executed: bool,
}

#[derive(Clone)]
pub enum CallArgument {
    Integer(i64),
    Boolean(bool),
    ByteString(Vec<u8>),
    String(String),
}

pub const ARG_INTEGER: u8 = 0;
pub const ARG_BOOL: u8 = 1;
pub const ARG_BYTES: u8 = 2;
pub const ARG_STRING: u8 = 3;

pub fn encode_arguments(arguments: &[CallArgument]) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.push(arguments.len() as u8);
    for argument in arguments {
        match argument {
            CallArgument::Integer(value) => {
                buffer.push(ARG_INTEGER);
                buffer.extend_from_slice(&(8u16).to_le_bytes());
                buffer.extend_from_slice(&value.to_le_bytes());
            }
            CallArgument::Boolean(value) => {
                buffer.push(ARG_BOOL);
                buffer.extend_from_slice(&(1u16).to_le_bytes());
                buffer.push(*value as u8);
            }
            CallArgument::ByteString(bytes) => {
                buffer.push(ARG_BYTES);
                buffer.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
                buffer.extend_from_slice(bytes);
            }
            CallArgument::String(value) => {
                buffer.push(ARG_STRING);
                buffer.extend_from_slice(&(value.len() as u16).to_le_bytes());
                buffer.extend_from_slice(value.as_bytes());
            }
        }
    }
    buffer
}

pub fn decode_arguments_from_bytes(bytes: &[u8]) -> Option<Vec<CallArgument>> {
    if bytes.is_empty() {
        return Some(Vec::new());
    }
    let mut cursor = 0usize;
    let count = bytes[cursor] as usize;
    cursor += 1;
    let mut parsed = Vec::with_capacity(count);
    for _ in 0..count {
        if cursor >= bytes.len() {
            return None;
        }
        let kind = bytes[cursor];
        cursor += 1;
        if cursor + 2 > bytes.len() {
            return None;
        }
        let length = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        cursor += 2;
        if cursor + length > bytes.len() {
            return None;
        }
        let segment = &bytes[cursor..cursor + length];
        cursor += length;
        let arg = match kind {
            ARG_INTEGER => {
                if length != 8 {
                    return None;
                }
                let mut buf = [0u8; 8];
                buf.copy_from_slice(segment);
                CallArgument::Integer(i64::from_le_bytes(buf))
            }
            ARG_BOOL => {
                if length != 1 {
                    return None;
                }
                CallArgument::Boolean(segment[0] != 0)
            }
            ARG_BYTES => CallArgument::ByteString(segment.to_vec()),
            ARG_STRING => match core::str::from_utf8(segment) {
                Ok(value) => CallArgument::String(value.to_string()),
                Err(_) => return None,
            },
            _ => return None,
        };
        parsed.push(arg);
    }
    Some(parsed)
}

pub fn build_argument_array(arguments: &[CallArgument]) -> Option<NeoArray<NeoValue>> {
    let mut values = Vec::with_capacity(arguments.len());
    for argument in arguments {
        let value = match argument {
            CallArgument::Integer(v) => NeoValue::from(NeoInteger::new(*v)),
            CallArgument::Boolean(v) => NeoValue::from(NeoBoolean::new(*v)),
            CallArgument::String(v) => NeoValue::from(NeoString::from_str(v)),
            CallArgument::ByteString(bytes) => NeoValue::from(NeoByteString::from_slice(bytes)),
        };
        values.push(value);
    }
    Some(NeoArray::from_vec(values))
}

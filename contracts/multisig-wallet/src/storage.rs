use neo_devpack::prelude::*;

pub const CONFIG_THRESHOLD_KEY: &[u8] = b"cfg:threshold";
pub const CONFIG_OWNER_COUNT_KEY: &[u8] = b"cfg:owners";
pub const CONFIG_OWNER_PREFIX: &[u8] = b"cfg:owner:";
pub const PROPOSAL_COUNTER_KEY: &[u8] = b"proposal:counter";
pub const PROPOSAL_PREFIX: &[u8] = b"proposal:";
pub const PROPOSER_SUFFIX: &[u8] = b":proposer";
pub const TARGET_SUFFIX: &[u8] = b":target";
pub const METHOD_SUFFIX: &[u8] = b":method";
pub const ARG_SUFFIX: &[u8] = b":args";
pub const APPROVAL_COUNT_SUFFIX: &[u8] = b":approvals";
pub const APPROVAL_PREFIX: &[u8] = b":approval:";
pub const EXECUTED_SUFFIX: &[u8] = b":executed";

pub fn storage_context() -> Option<NeoStorageContext> {
    NeoStorage::get_context().ok()
}

pub fn write_bytes(ctx: &NeoStorageContext, key: &[u8], bytes: &[u8]) -> NeoResult<()> {
    let key_bytes = NeoByteString::from_slice(key);
    let value = NeoByteString::from_slice(bytes);
    NeoStorage::put(ctx, &key_bytes, &value)
}

pub fn read_storage_bytes(ctx: &NeoStorageContext, key: &[u8]) -> Option<Vec<u8>> {
    let key_bytes = NeoByteString::from_slice(key);
    let bytes = NeoStorage::get(ctx, &key_bytes).ok()?;
    if bytes.is_empty() {
        return None;
    }
    Some(bytes.as_slice().to_vec())
}

pub fn write_i64(ctx: &NeoStorageContext, key: &[u8], value: i64) -> NeoResult<()> {
    write_bytes(ctx, key, &value.to_le_bytes())
}

pub fn read_i64(ctx: &NeoStorageContext, key: &[u8]) -> Option<i64> {
    let bytes = read_storage_bytes(ctx, key)?;
    if bytes.len() != 8 {
        return None;
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes);
    Some(i64::from_le_bytes(buf))
}

pub fn write_u16(ctx: &NeoStorageContext, key: &[u8], value: u16) -> NeoResult<()> {
    write_bytes(ctx, key, &value.to_le_bytes())
}

pub fn read_u16(ctx: &NeoStorageContext, key: &[u8]) -> Option<u16> {
    let bytes = read_storage_bytes(ctx, key)?;
    if bytes.len() != 2 {
        return None;
    }
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

pub fn write_bool(ctx: &NeoStorageContext, key: &[u8], value: bool) -> NeoResult<()> {
    write_bytes(ctx, key, &[value as u8])
}

pub fn read_bool(ctx: &NeoStorageContext, key: &[u8]) -> Option<bool> {
    let bytes = read_storage_bytes(ctx, key)?;
    if bytes.len() != 1 {
        return None;
    }
    Some(bytes[0] != 0)
}

pub fn write_string(ctx: &NeoStorageContext, key: &[u8], value: &str) -> NeoResult<()> {
    let mut buffer = Vec::with_capacity(2 + value.len());
    buffer.extend_from_slice(&(value.len() as u16).to_le_bytes());
    buffer.extend_from_slice(value.as_bytes());
    write_bytes(ctx, key, &buffer)
}

pub fn read_storage_string(ctx: &NeoStorageContext, key: &[u8]) -> Option<String> {
    let bytes = read_storage_bytes(ctx, key)?;
    if bytes.len() < 2 {
        return None;
    }
    let len = u16::from_le_bytes([bytes[0], bytes[1]]) as usize;
    if bytes.len() - 2 != len {
        return None;
    }
    String::from_utf8(bytes[2..].to_vec()).ok()
}

pub fn config_owner_key(index: u16) -> Vec<u8> {
    let mut key = CONFIG_OWNER_PREFIX.to_vec();
    key.extend_from_slice(&index.to_le_bytes());
    key
}

pub fn proposal_field_key(id: i64, suffix: &[u8]) -> Vec<u8> {
    let mut key = PROPOSAL_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key.extend_from_slice(suffix);
    key
}

pub fn proposal_approval_key(id: i64, index: u16) -> Vec<u8> {
    let mut key = PROPOSAL_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key.extend_from_slice(APPROVAL_PREFIX);
    key.extend_from_slice(&index.to_le_bytes());
    key
}

use neo_devpack::{codec, prelude::*};
use neo_devpack::serde::{Deserialize, Serialize};

pub const CONFIG_KEY: &[u8] = b"dao:config";
pub const PROPOSAL_COUNTER_KEY: &[u8] = b"dao:counter";
pub const PROPOSAL_PREFIX: &[u8] = b"dao:proposal:";
pub const STAKE_PREFIX: &[u8] = b"dao:stake:";
pub const VOTE_PREFIX: &[u8] = b"dao:vote:";

pub fn storage_context() -> Option<NeoStorageContext> {
    NeoStorage::get_context().ok()
}

pub fn load_from_storage<T>(ctx: &NeoStorageContext, key: &[u8]) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    let key_bytes = NeoByteString::from_slice(key);
    let data = NeoStorage::get(ctx, &key_bytes).ok()?;
    if data.is_empty() {
        return None;
    }
    codec::deserialize(data.as_slice()).ok()
}

pub fn store_to_storage<T>(ctx: &NeoStorageContext, key: &[u8], value: &T) -> NeoResult<()>
where
    T: Serialize,
{
    let encoded = codec::serialize(value)?;
    let key_bytes = NeoByteString::from_slice(key);
    let value_bytes = NeoByteString::from_slice(&encoded);
    NeoStorage::put(ctx, &key_bytes, &value_bytes)
}

pub fn serialize_value<T: Serialize>(value: &T) -> NeoByteString {
    match codec::serialize(value) {
        Ok(bytes) => NeoByteString::from_slice(&bytes),
        Err(_) => NeoByteString::new(Vec::new()),
    }
}

pub fn proposal_key(id: i64) -> Vec<u8> {
    let mut key = PROPOSAL_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key
}

pub fn stake_key(address: &NeoByteString) -> Vec<u8> {
    let mut key = STAKE_PREFIX.to_vec();
    key.extend_from_slice(address.as_slice());
    key
}

pub fn vote_key(id: i64, address: &NeoByteString) -> Vec<u8> {
    let mut key = VOTE_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key.push(b':');
    key.extend_from_slice(address.as_slice());
    key
}

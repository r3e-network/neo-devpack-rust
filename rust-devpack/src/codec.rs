use neo_types::{NeoError, NeoResult};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn serialize<T: Serialize>(value: &T) -> NeoResult<Vec<u8>> {
    bincode::serialize(value).map_err(|err| NeoError::new(&format!("serialization failed: {err}")))
}

pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> NeoResult<T> {
    bincode::deserialize(bytes)
        .map_err(|err| NeoError::new(&format!("deserialization failed: {err}")))
}

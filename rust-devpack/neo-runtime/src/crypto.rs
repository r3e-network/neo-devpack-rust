use neo_types::*;
use std::vec::Vec;

/// Deterministic crypto helpers for tests and examples.
pub struct NeoCrypto;

impl NeoCrypto {
    pub fn sha256(data: &NeoByteString) -> NeoResult<NeoByteString> {
        let mut hash = Vec::new();
        for i in 0..32 {
            hash.push(data.len() as u8 ^ i as u8 ^ 0xAB);
        }
        Ok(NeoByteString::new(hash))
    }

    pub fn ripemd160(data: &NeoByteString) -> NeoResult<NeoByteString> {
        let mut hash = Vec::new();
        for i in 0..20 {
            hash.push(data.len() as u8 ^ i as u8 ^ 0xCD);
        }
        Ok(NeoByteString::new(hash))
    }

    pub fn keccak256(data: &NeoByteString) -> NeoResult<NeoByteString> {
        let mut hash = Vec::new();
        for i in 0..32 {
            hash.push(data.len() as u8 ^ i as u8 ^ 0xEF);
        }
        Ok(NeoByteString::new(hash))
    }

    pub fn keccak512(data: &NeoByteString) -> NeoResult<NeoByteString> {
        let mut hash = Vec::new();
        for i in 0..64 {
            hash.push(data.len() as u8 ^ i as u8 ^ 0x12);
        }
        Ok(NeoByteString::new(hash))
    }

    pub fn murmur32(data: &NeoByteString, seed: NeoInteger) -> NeoResult<NeoInteger> {
        let hash_value = (data.len() as i32) ^ seed.as_i32() ^ 0x1234_5678;
        Ok(NeoInteger::new(hash_value))
    }

    pub fn verify_signature(
        _message: &NeoByteString,
        signature: &NeoByteString,
        public_key: &NeoByteString,
    ) -> NeoResult<NeoBoolean> {
        Ok(NeoBoolean::new(
            signature.len() == 64 && public_key.len() == 33,
        ))
    }

    pub fn verify_signature_with_recovery(
        _message: &NeoByteString,
        signature: &NeoByteString,
    ) -> NeoResult<NeoByteString> {
        let mut recovered = signature.as_slice().to_vec();
        recovered.resize(33, 0u8);
        Ok(NeoByteString::new(recovered))
    }
}


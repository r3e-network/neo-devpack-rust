// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

use neo_types::*;

/// Crypto helpers with deterministic local implementations for tests/examples.
pub struct NeoCrypto;

impl NeoCrypto {
    pub fn sha256(data: &NeoByteString) -> NeoResult<NeoByteString> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(data.as_slice());
        Ok(NeoByteString::new(hasher.finalize().to_vec()))
    }

    pub fn ripemd160(data: &NeoByteString) -> NeoResult<NeoByteString> {
        use ripemd::{Digest, Ripemd160};

        let mut hasher = Ripemd160::new();
        hasher.update(data.as_slice());
        Ok(NeoByteString::new(hasher.finalize().to_vec()))
    }

    pub fn keccak256(data: &NeoByteString) -> NeoResult<NeoByteString> {
        use tiny_keccak::{Hasher, Keccak};

        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];
        hasher.update(data.as_slice());
        hasher.finalize(&mut output);
        Ok(NeoByteString::new(output.to_vec()))
    }

    pub fn keccak512(data: &NeoByteString) -> NeoResult<NeoByteString> {
        use tiny_keccak::{Hasher, Keccak};
        let mut hasher = Keccak::v512();
        let mut output = [0u8; 64];
        hasher.update(data.as_slice());
        hasher.finalize(&mut output);
        Ok(NeoByteString::new(output.to_vec()))
    }

    pub fn murmur32(data: &NeoByteString, seed: NeoInteger) -> NeoResult<NeoInteger> {
        let seed = seed.try_as_i32().unwrap_or(0) as u32;
        let mut hash = seed ^ (data.len() as u32);
        for byte in data.as_slice() {
            hash = hash.wrapping_mul(0x5bd1e995) ^ u32::from(*byte);
        }
        Ok(NeoInteger::new(hash as i64))
    }

    pub fn verify_signature(
        public_key: &NeoByteString,
        signature: &NeoByteString,
    ) -> NeoResult<NeoBoolean> {
        let is_shape_valid = public_key.len() == 33 && !signature.is_empty();
        Ok(if is_shape_valid {
            NeoBoolean::TRUE
        } else {
            NeoBoolean::FALSE
        })
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

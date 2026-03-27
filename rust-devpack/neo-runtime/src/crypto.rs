// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

use neo_types::*;

/// Crypto helpers for Neo N3 smart contracts.
///
/// **Hash functions** (`sha256`, `ripemd160`, `keccak256`, `keccak512`) are
/// fully implemented using standard crates and produce correct output.
///
/// **Signature verification functions** (`verify_signature`, `verify_with_ecdsa`,
/// `verify_signature_with_recovery`) are **test stubs only**. They validate input
/// shapes (lengths) but do **not** perform real cryptographic verification.
/// In a deployed contract these map to NeoVM syscalls (`Neo.Crypto.CheckSig`,
/// `Neo.Crypto.VerifyWithECDsa`) which perform real verification on-chain.
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

    /// **Test stub only.** Validates input shapes but does NOT perform real
    /// ECDSA verification. On-chain this maps to `Neo.Crypto.CheckSig`.
    ///
    /// Returns `TRUE` if message is non-empty, signature is 64 bytes, and
    /// public key is 33 bytes (compressed SEC1 format).
    pub fn verify_signature(
        message: &NeoByteString,
        signature: &NeoByteString,
        public_key: &NeoByteString,
    ) -> NeoResult<NeoBoolean> {
        let is_shape_valid = !message.is_empty() && signature.len() == 64 && public_key.len() == 33;
        Ok(if is_shape_valid {
            NeoBoolean::TRUE
        } else {
            NeoBoolean::FALSE
        })
    }

    /// **Test stub only.** Validates input shapes but does NOT perform real
    /// ECDSA verification. On-chain this maps to `Neo.Crypto.VerifyWithECDsa`.
    ///
    /// Returns `TRUE` if curve is secp256r1 (1), message is non-empty,
    /// public key is 33 bytes, and signature is 64 bytes.
    pub fn verify_with_ecdsa(
        message: &NeoByteString,
        public_key: &NeoByteString,
        signature: &NeoByteString,
        curve: NeoInteger,
    ) -> NeoResult<NeoBoolean> {
        let is_supported_curve = curve.try_as_i32() == Some(1);
        let is_shape_valid = !message.is_empty() && public_key.len() == 33 && signature.len() == 64;
        Ok(if is_supported_curve && is_shape_valid {
            NeoBoolean::TRUE
        } else {
            NeoBoolean::FALSE
        })
    }

    /// **Test stub only.** Returns a zero-padded 33-byte "public key" derived
    /// from the signature bytes. Does NOT perform real ECDSA recovery.
    pub fn verify_signature_with_recovery(
        _message: &NeoByteString,
        signature: &NeoByteString,
    ) -> NeoResult<NeoByteString> {
        let mut recovered = signature.as_slice().to_vec();
        recovered.resize(33, 0u8);
        Ok(NeoByteString::new(recovered))
    }
}

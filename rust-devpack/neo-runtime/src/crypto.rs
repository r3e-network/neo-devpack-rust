// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

use neo_syscalls::NeoVMSyscall;
use neo_types::*;

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
        // Use try_as_i32() for safe conversion, defaulting to 0 if out of range
        let seed_i32 = seed.try_as_i32().unwrap_or(0);
        let hash_value = (data.len() as i32) ^ seed_i32 ^ 0x1234_5678;
        Ok(NeoInteger::new(hash_value))
    }

    /// Verify a signature using the Neo `System.Crypto.CheckSig` semantics.
    ///
    /// Parameter order is `(message, signature, public_key)`.
    pub fn verify_signature(
        _message: &NeoByteString,
        signature: &NeoByteString,
        public_key: &NeoByteString,
    ) -> NeoResult<NeoBoolean> {
        // Keep basic shape validation deterministic in host tests.
        if signature.len() != 64 || public_key.len() != 33 {
            return Ok(NeoBoolean::FALSE);
        }

        // Delegate to syscall shim so host-mode auth hardening applies.
        NeoVMSyscall::check_sig(public_key, signature)
    }

    /// Verify a signature with explicit ECDSA curve selection.
    ///
    /// Parameter order is `(message, public_key, signature, curve)`.
    pub fn verify_with_ecdsa(
        message: &NeoByteString,
        public_key: &NeoByteString,
        signature: &NeoByteString,
        curve: NeoInteger,
    ) -> NeoResult<NeoBoolean> {
        // Keep deterministic shape checks in host tests.
        if signature.len() != 64 || public_key.len() != 33 {
            return Ok(NeoBoolean::FALSE);
        }

        // Neo supports secp256k1 (0) and secp256r1 (1) here.
        let curve_id = curve.try_as_i32().unwrap_or(-1);
        if curve_id != 0 && curve_id != 1 {
            return Ok(NeoBoolean::FALSE);
        }

        NeoVMSyscall::verify_with_ecdsa(message, public_key, signature, &curve)
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

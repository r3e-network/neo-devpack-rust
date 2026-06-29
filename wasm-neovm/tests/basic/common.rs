// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use sha2::{Digest, Sha256};

pub fn double_sha256_checksum(data: &[u8]) -> u32 {
    let hash = Sha256::digest(data);
    let hash = Sha256::digest(hash);
    u32::from_le_bytes(hash[..4].try_into().unwrap())
}

/// Decode a Neo `VarInt` length prefix. Delegates to the crate's canonical
/// decoder so tests share one definition of the wire format; panics on
/// malformed input (test-only).
pub fn read_var_uint(bytes: &[u8]) -> (u64, usize) {
    wasm_neovm::core::decode_varint(bytes).expect("valid Neo VarInt")
}

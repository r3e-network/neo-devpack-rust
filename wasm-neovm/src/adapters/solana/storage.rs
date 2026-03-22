// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

/// Solana account info layout constants.
#[allow(dead_code)]
pub mod account_layout {
    /// Size of a public key in bytes.
    pub const PUBKEY_SIZE: usize = 32;

    /// Offset of lamports in serialized account info.
    pub const LAMPORTS_OFFSET: usize = 0;

    /// Offset of data length in serialized account info.
    pub const DATA_LEN_OFFSET: usize = 8;

    /// Offset of data in serialized account info.
    pub const DATA_OFFSET: usize = 16;
}

/// Helper to generate storage key from Solana account pubkey.
#[allow(dead_code)]
pub fn solana_pubkey_to_storage_key(pubkey: &[u8; 32]) -> Vec<u8> {
    // Use first 20 bytes as Neo-compatible key.
    // Prefix with "sol:" to namespace Solana-origin data.
    let mut key = Vec::with_capacity(24);
    key.extend_from_slice(b"sol:");
    key.extend_from_slice(&pubkey[..20]);
    key
}

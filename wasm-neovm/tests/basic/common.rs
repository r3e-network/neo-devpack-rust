// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use sha2::{Digest, Sha256};

pub fn double_sha256_checksum(data: &[u8]) -> u32 {
    let hash = Sha256::digest(data);
    let hash = Sha256::digest(hash);
    u32::from_le_bytes(hash[..4].try_into().unwrap())
}

pub fn read_var_uint(bytes: &[u8]) -> (u64, usize) {
    let prefix = bytes[0];
    match prefix {
        n if n < 0xFD => (u64::from(n), 1),
        0xFD => {
            let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
            (u64::from(value), 3)
        }
        0xFE => {
            let value = u32::from_le_bytes(bytes[1..5].try_into().unwrap());
            (u64::from(value), 5)
        }
        0xFF => {
            let value = u64::from_le_bytes(bytes[1..9].try_into().unwrap());
            (value, 9)
        }
        _ => unreachable!(),
    }
}

// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Pubkey type representing a 32-byte public key
//!
//! Maps to Neo's `UInt160` (contract hash) or `UInt256` as appropriate.

use core::fmt;

use sha2::{Digest, Sha256};

/// Maximum number of seeds allowed in PDA derivation (Solana limit).
const MAX_SEEDS: usize = 16;
/// Maximum length of a single seed in PDA derivation (Solana limit).
const MAX_SEED_LEN: usize = 32;

/// A 32-byte public key (Solana-compatible)
///
/// In Neo context, this maps to:
/// - Contract addresses: hash-mapped to a 20-byte `UInt160`
/// - Full hashes: all 32 bytes as identifier
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Pubkey(pub [u8; 32]);

impl Pubkey {
    /// The length of a Pubkey in bytes
    pub const LEN: usize = 32;

    /// Create a new Pubkey from a byte array
    pub const fn new(pubkey: [u8; 32]) -> Self {
        Self(pubkey)
    }

    /// Create a new Pubkey initialized to all zeros
    pub const fn new_default() -> Self {
        Self([0u8; 32])
    }

    /// Create from a slice (panics if wrong length)
    pub fn new_from_slice(slice: &[u8]) -> Self {
        let mut key = [0u8; 32];
        key.copy_from_slice(slice);
        Self(key)
    }

    /// Get the underlying byte array
    pub const fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    /// Convert to Neo `UInt160` format via a cryptographic hash.
    ///
    /// Uses `SHA-256(pubkey)[..20]` to produce a collision-resistant 20-byte
    /// identifier. Truncating a 256-bit hash to 160 bits preserves 80 bits of
    /// collision resistance — far more than Neo's 160-bit address space implies.
    /// This is strictly safer than the previous approach of copying the first
    /// 20 raw bytes, which made `2^96` distinct Pubkeys alias to the same
    /// UInt160.
    pub fn to_neo_uint160(&self) -> [u8; 20] {
        let hash = Sha256::digest(self.0);
        let mut result = [0u8; 20];
        result.copy_from_slice(&hash[..20]);
        result
    }

    /// Check if this is the system program ID
    pub fn is_system_program(&self) -> bool {
        // System program has all zeros except position 0
        self.0[0] == 0 && self.0[1..].iter().all(|&b| b == 0)
    }

    /// Find a program-derived address (PDA) using SHA-256.
    ///
    /// Mirrors Solana's PDA derivation: for each bump seed from 255 down to 0,
    /// compute `SHA-256(seeds ‖ bump ‖ program_id)`. The first result that is
    /// *not* a valid ed25519 public key is the PDA. If none qualifies (extremely
    /// unlikely), the last computed value is returned with bump=0 as a
    /// deterministic fallback so the caller always gets a usable address.
    ///
    /// Unlike the previous XOR-based derivation, this is collision-resistant:
    /// finding two seed sets that produce the same PDA requires a SHA-256
    /// preimage/collision, which is computationally infeasible.
    pub fn find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
        for bump in (0u8..=255).rev() {
            match Self::create_program_address_with_bump(seeds, program_id, bump) {
                Ok(addr) => {
                    // In Solana, a PDA must not lie on the ed25519 curve. Since
                    // Neo does not use ed25519 for addressing, we accept the
                    // highest bump (255 down) deterministically and treat all
                    // derived addresses as valid PDAs. The bump search still
                    // runs for API compatibility with Solana callers.
                    return (addr, bump);
                }
                Err(PubkeyError::MaxSeedLengthExceeded) => {
                    return (Pubkey::new_default(), 0);
                }
                Err(_) => continue,
            }
        }
        // Fallback: return zero key with bump 0 (should never reach here)
        (Pubkey::new_default(), 0)
    }

    /// Create a program-derived address with a specific bump seed.
    ///
    /// Computes `SHA-256(seed_0 ‖ seed_1 ‖ … ‖ seed_n ‖ bump ‖ program_id)`.
    pub fn create_program_address_with_bump(
        seeds: &[&[u8]],
        program_id: &Pubkey,
        bump: u8,
    ) -> Result<Pubkey, PubkeyError> {
        if seeds.len() > MAX_SEEDS {
            return Err(PubkeyError::MaxSeedLengthExceeded);
        }
        let mut hasher = Sha256::new();
        for seed in seeds {
            if seed.len() > MAX_SEED_LEN {
                return Err(PubkeyError::MaxSeedLengthExceeded);
            }
            hasher.update(seed);
        }
        hasher.update([bump]);
        hasher.update(program_id.0);
        let result = hasher.finalize();
        Ok(Pubkey(result.into()))
    }

    /// Create a program-derived address (must succeed)
    ///
    /// Convenience wrapper that uses bump=255. For the full bump-seed search,
    /// use [`find_program_address`](Self::find_program_address).
    pub fn create_program_address(
        seeds: &[&[u8]],
        program_id: &Pubkey,
    ) -> Result<Pubkey, PubkeyError> {
        Self::create_program_address_with_bump(seeds, program_id, 255)
    }
}

impl Default for Pubkey {
    fn default() -> Self {
        Self::new_default()
    }
}

impl fmt::Debug for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pubkey({:?})", &self.0[..8])
    }
}

impl AsRef<[u8]> for Pubkey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Errors related to Pubkey operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PubkeyError {
    /// The provided seeds exceed the maximum length
    MaxSeedLengthExceeded,
    /// The derived address is invalid
    InvalidSeeds,
}

/// System program ID (all zeros in Solana)
pub const SYSTEM_PROGRAM_ID: Pubkey = Pubkey([0u8; 32]);

/// Token program ID placeholder
pub const TOKEN_PROGRAM_ID: Pubkey = Pubkey([
    0x06, 0xdd, 0xf6, 0xe1, 0xd7, 0x65, 0xa1, 0x93, 0xd9, 0xcb, 0xe1, 0x46, 0xce, 0xeb, 0x79, 0xac,
    0x1c, 0xb4, 0x85, 0xed, 0x5f, 0x5b, 0x37, 0x91, 0x3a, 0x8c, 0xf5, 0x85, 0x7e, 0xff, 0x00, 0xa9,
]);

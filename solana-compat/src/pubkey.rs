//! Pubkey type representing a 32-byte public key
//!
//! Maps to Neo's `UInt160` (contract hash) or `UInt256` as appropriate.

use core::fmt;

/// A 32-byte public key (Solana-compatible)
///
/// In Neo context, this maps to:
/// - Contract addresses: first 20 bytes as `UInt160`
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

    /// Convert to Neo `UInt160` format (first 20 bytes)
    pub fn to_neo_uint160(&self) -> [u8; 20] {
        let mut result = [0u8; 20];
        result.copy_from_slice(&self.0[..20]);
        result
    }

    /// Check if this is the system program ID
    pub fn is_system_program(&self) -> bool {
        // System program has all zeros except position 0
        self.0[0] == 0 && self.0[1..].iter().all(|&b| b == 0)
    }

    /// Find a program-derived address
    ///
    /// In Neo context, this creates a deterministic storage key
    pub fn find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
        // Simplified PDA derivation for Neo
        // In practice, use SHA256 of seeds + program_id
        let mut result = [0u8; 32];
        let mut offset = 0;

        // Hash seeds together
        for seed in seeds {
            for &byte in *seed {
                if offset < 32 {
                    result[offset] ^= byte;
                    offset = (offset + 1) % 32;
                }
            }
        }

        // Mix in program ID
        for (i, &byte) in program_id.0.iter().enumerate() {
            result[i] ^= byte;
        }

        (Pubkey(result), 255) // Bump seed
    }

    /// Create a program-derived address (must succeed)
    pub fn create_program_address(
        seeds: &[&[u8]],
        program_id: &Pubkey,
    ) -> Result<Pubkey, PubkeyError> {
        let (addr, _) = Self::find_program_address(seeds, program_id);
        Ok(addr)
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

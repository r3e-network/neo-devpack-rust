//! Account info structure for Solana compatibility
//!
//! Maps Solana account model to Neo contract storage.

use crate::pubkey::Pubkey;
use core::cell::{Ref, RefCell, RefMut};

/// Account information for a Solana-compatible program
///
/// In Neo context:
/// - `key`: Storage key identifier
/// - `data`: Value stored at that key
/// - `owner`: Contract that owns this storage slot
/// - `lamports`: Mapped to GAS balance (informational only)
pub struct AccountInfo<'a> {
    /// Public key of the account (storage key in Neo)
    pub key: &'a Pubkey,
    /// The lamports in the account (GAS equivalent, read-only in Neo)
    pub lamports: RefCell<&'a mut u64>,
    /// The data held in this account (storage value in Neo)
    pub data: RefCell<&'a mut [u8]>,
    /// Program that owns this account (contract hash in Neo)
    pub owner: &'a Pubkey,
    /// Is this account a signer?
    pub is_signer: bool,
    /// Is this account writable?
    pub is_writable: bool,
    /// Was the data modified?
    pub executable: bool,
    /// Rent epoch (not applicable in Neo)
    pub rent_epoch: u64,
}

impl<'a> AccountInfo<'a> {
    /// Create a new AccountInfo
    pub fn new(
        key: &'a Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
        executable: bool,
        rent_epoch: u64,
    ) -> Self {
        Self {
            key,
            lamports: RefCell::new(lamports),
            data: RefCell::new(data),
            owner,
            is_signer,
            is_writable,
            executable,
            rent_epoch,
        }
    }

    /// Get the account's public key
    pub fn key(&self) -> &Pubkey {
        self.key
    }

    /// Get the lamports balance
    pub fn lamports(&self) -> u64 {
        **self.lamports.borrow()
    }

    /// Try to borrow the account data
    pub fn try_borrow_data(&self) -> Result<Ref<&'a mut [u8]>, BorrowError> {
        self.data.try_borrow().map_err(|_| BorrowError)
    }

    /// Try to borrow the account data mutably
    pub fn try_borrow_mut_data(&self) -> Result<RefMut<&'a mut [u8]>, BorrowError> {
        self.data.try_borrow_mut().map_err(|_| BorrowError)
    }

    /// Get the length of the data
    pub fn data_len(&self) -> usize {
        self.data.borrow().len()
    }

    /// Check if the data is empty
    pub fn data_is_empty(&self) -> bool {
        self.data.borrow().is_empty()
    }

    /// Get the owner pubkey
    pub fn owner(&self) -> &Pubkey {
        self.owner
    }

    /// Check if this account is a signer
    pub fn is_signer(&self) -> bool {
        self.is_signer
    }

    /// Check if this account is writable
    pub fn is_writable(&self) -> bool {
        self.is_writable
    }

    /// Assign a new owner to the account
    ///
    /// Note: In Neo, this operation is not directly supported.
    /// Ownership is determined by the contract that created the storage.
    pub fn assign(&self, _new_owner: &Pubkey) {
        // No-op in Neo context
    }

    /// Realloc the account data
    ///
    /// Note: In Neo, storage values can grow dynamically.
    pub fn realloc(&self, _new_len: usize, _zero_init: bool) -> Result<(), ProgramError> {
        // In Neo, storage is dynamically sized
        // This is a no-op since we handle it at the storage layer
        Ok(())
    }
}

impl<'a> core::fmt::Debug for AccountInfo<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AccountInfo")
            .field("key", self.key)
            .field("owner", self.owner)
            .field("is_signer", &self.is_signer)
            .field("is_writable", &self.is_writable)
            .field("data_len", &self.data_len())
            .finish()
    }
}

/// Error type for borrow failures
#[derive(Debug, Clone, Copy)]
pub struct BorrowError;

use crate::program_error::ProgramError;

/// Helper to find an account by public key in a slice
pub fn next_account_info<'a, 'b, I: Iterator<Item = &'a AccountInfo<'b>>>(
    iter: &mut I,
) -> Result<&'a AccountInfo<'b>, ProgramError> {
    iter.next().ok_or(ProgramError::NotEnoughAccountKeys)
}

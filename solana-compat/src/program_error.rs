//! Program error types for Solana compatibility

use core::fmt;

/// Reasons the program may fail
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgramError {
    /// Custom program error
    Custom(u32),
    /// Invalid argument provided
    InvalidArgument,
    /// Invalid instruction data
    InvalidInstructionData,
    /// Invalid account data
    InvalidAccountData,
    /// Account data too small
    AccountDataTooSmall,
    /// Insufficient funds
    InsufficientFunds,
    /// Incorrect program id
    IncorrectProgramId,
    /// Missing required signature
    MissingRequiredSignature,
    /// Account already initialized
    AccountAlreadyInitialized,
    /// Uninitialized account
    UninitializedAccount,
    /// Not enough account keys
    NotEnoughAccountKeys,
    /// Account borrow failed
    AccountBorrowFailed,
    /// Max seed length exceeded
    MaxSeedLengthExceeded,
    /// Invalid seeds
    InvalidSeeds,
    /// Borsh IO error
    BorshIoError,
    /// Account not rent exempt
    AccountNotRentExempt,
    /// Unsupported sysvar
    UnsupportedSysvar,
    /// Illegal owner
    IllegalOwner,
    /// Max accounts data size exceeded
    MaxAccountsDataSizeExceeded,
    /// Invalid reentrancy
    InvalidReentrancy,
}

impl ProgramError {
    /// Convert to u64 error code
    pub const fn to_u64(&self) -> u64 {
        match self {
            ProgramError::Custom(error) => *error as u64,
            ProgramError::InvalidArgument => 1,
            ProgramError::InvalidInstructionData => 2,
            ProgramError::InvalidAccountData => 3,
            ProgramError::AccountDataTooSmall => 4,
            ProgramError::InsufficientFunds => 5,
            ProgramError::IncorrectProgramId => 6,
            ProgramError::MissingRequiredSignature => 7,
            ProgramError::AccountAlreadyInitialized => 8,
            ProgramError::UninitializedAccount => 9,
            ProgramError::NotEnoughAccountKeys => 10,
            ProgramError::AccountBorrowFailed => 11,
            ProgramError::MaxSeedLengthExceeded => 12,
            ProgramError::InvalidSeeds => 13,
            ProgramError::BorshIoError => 14,
            ProgramError::AccountNotRentExempt => 15,
            ProgramError::UnsupportedSysvar => 16,
            ProgramError::IllegalOwner => 17,
            ProgramError::MaxAccountsDataSizeExceeded => 18,
            ProgramError::InvalidReentrancy => 19,
        }
    }
}

impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgramError::Custom(code) => write!(f, "Custom error: {}", code),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl From<u64> for ProgramError {
    fn from(error: u64) -> Self {
        match error {
            1 => ProgramError::InvalidArgument,
            2 => ProgramError::InvalidInstructionData,
            3 => ProgramError::InvalidAccountData,
            4 => ProgramError::AccountDataTooSmall,
            5 => ProgramError::InsufficientFunds,
            6 => ProgramError::IncorrectProgramId,
            7 => ProgramError::MissingRequiredSignature,
            8 => ProgramError::AccountAlreadyInitialized,
            9 => ProgramError::UninitializedAccount,
            10 => ProgramError::NotEnoughAccountKeys,
            11 => ProgramError::AccountBorrowFailed,
            12 => ProgramError::MaxSeedLengthExceeded,
            13 => ProgramError::InvalidSeeds,
            14 => ProgramError::BorshIoError,
            15 => ProgramError::AccountNotRentExempt,
            16 => ProgramError::UnsupportedSysvar,
            17 => ProgramError::IllegalOwner,
            18 => ProgramError::MaxAccountsDataSizeExceeded,
            19 => ProgramError::InvalidReentrancy,
            _ => ProgramError::Custom(error as u32),
        }
    }
}

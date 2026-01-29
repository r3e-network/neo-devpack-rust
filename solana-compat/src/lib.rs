// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Neo-Solana Compatibility Layer
//!
//! This crate provides a Solana-compatible API surface that compiles to WASM
//! and can be translated to `NeoVM` bytecode via `wasm-neovm`.
//!
//! # Usage
//!
//! Replace `solana_program` imports with `neo_solana_compat`:
//!
//! ```rust,ignore
//! // Before (Solana)
//! use solana_program::{account_info::AccountInfo, entrypoint, pubkey::Pubkey};
//!
//! // After (Neo-compatible)
//! use neo_solana_compat::{account_info::AccountInfo, entrypoint, pubkey::Pubkey};
//! ```
//!
//! # Architecture
//!
//! The compatibility layer maps Solana concepts to Neo equivalents:
//!
//! - **Accounts** → Neo contract storage slots
//! - **Program IDs** → Neo contract hashes (`UInt160`)
//! - **Syscalls** → Neo interop services
//! - **Signatures** → `CheckWitness` verification

#![no_std]
#![allow(dead_code)]

pub mod account_info;
pub mod entrypoint;
pub mod program;
pub mod program_error;
pub mod pubkey;
pub mod syscalls;

pub use account_info::AccountInfo;
pub use entrypoint::ProgramResult;
pub use program_error::ProgramError;
pub use pubkey::Pubkey;

/// Re-export commonly used items
pub mod prelude {
    pub use crate::account_info::AccountInfo;
    pub use crate::entrypoint;
    pub use crate::entrypoint::ProgramResult;
    pub use crate::program::invoke;
    pub use crate::program_error::ProgramError;
    pub use crate::pubkey::Pubkey;
}

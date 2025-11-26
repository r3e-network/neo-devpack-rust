//! Program invocation utilities
//!
//! Provides cross-program invocation (CPI) support mapped to Neo contract calls.

use crate::account_info::AccountInfo;
use crate::program_error::ProgramError;
use crate::pubkey::Pubkey;
use crate::syscalls;

/// Instruction to be invoked
pub struct Instruction {
    /// Program ID to invoke
    pub program_id: Pubkey,
    /// Accounts required by the instruction
    pub accounts: Vec<AccountMeta>,
    /// Instruction data
    pub data: Vec<u8>,
}

impl Instruction {
    /// Create a new instruction
    pub fn new(program_id: Pubkey, data: Vec<u8>, accounts: Vec<AccountMeta>) -> Self {
        Self {
            program_id,
            accounts,
            data,
        }
    }

    /// Create a new instruction with method name (Neo-style)
    pub fn new_with_method(
        program_id: Pubkey,
        method: &str,
        data: Vec<u8>,
        accounts: Vec<AccountMeta>,
    ) -> Self {
        // Encode method name at start of data
        let mut full_data = Vec::with_capacity(method.len() + 1 + data.len());
        full_data.push(method.len() as u8);
        full_data.extend_from_slice(method.as_bytes());
        full_data.extend_from_slice(&data);

        Self {
            program_id,
            accounts,
            data: full_data,
        }
    }
}

/// Account metadata for an instruction
#[derive(Clone)]
pub struct AccountMeta {
    /// Public key of the account
    pub pubkey: Pubkey,
    /// Is this account a signer?
    pub is_signer: bool,
    /// Is this account writable?
    pub is_writable: bool,
}

impl AccountMeta {
    /// Create a new writable account meta
    pub fn new(pubkey: Pubkey, is_signer: bool) -> Self {
        Self {
            pubkey,
            is_signer,
            is_writable: true,
        }
    }

    /// Create a new read-only account meta
    pub fn new_readonly(pubkey: Pubkey, is_signer: bool) -> Self {
        Self {
            pubkey,
            is_signer,
            is_writable: false,
        }
    }
}

/// Invoke a cross-program invocation
///
/// Maps to: System.Contract.Call in Neo
pub fn invoke(
    instruction: &Instruction,
    _account_infos: &[AccountInfo],
) -> Result<(), ProgramError> {
    // Extract method name from data if present, otherwise use default
    let method = if !instruction.data.is_empty() {
        let method_len = instruction.data[0] as usize;
        if method_len > 0 && instruction.data.len() > method_len {
            core::str::from_utf8(&instruction.data[1..=method_len]).unwrap_or("invoke")
        } else {
            "invoke"
        }
    } else {
        "invoke"
    };

    // Get the actual instruction data (after method name)
    let data_start = if !instruction.data.is_empty() {
        let method_len = instruction.data[0] as usize;
        1 + method_len
    } else {
        0
    };
    let args = if data_start < instruction.data.len() {
        &instruction.data[data_start..]
    } else {
        &[]
    };

    syscalls::sol_invoke(&instruction.program_id, method, args)
        .map_err(|e| ProgramError::Custom(e as u32))
}

/// Invoke a cross-program invocation with signer seeds
///
/// In Neo, signing is handled via CheckWitness rather than PDA seeds
pub fn invoke_signed(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
    _signers_seeds: &[&[&[u8]]],
) -> Result<(), ProgramError> {
    // In Neo context, the contract itself is the signer
    // PDA-based signing doesn't directly apply
    invoke(instruction, account_infos)
}

/// Set return data for the current instruction
pub fn set_return_data(data: &[u8]) {
    // In NeoVM, return values are pushed to the stack
    // This is handled automatically by the function return
    let _ = data;
}

/// Get return data from the last CPI
pub fn get_return_data() -> Option<(Pubkey, Vec<u8>)> {
    // Return data would be captured from the contract call result
    // Implementation depends on how wasm-neovm handles contract call returns
    None
}

// ============================================================================
// External allocation (for Vec support in no_std)
// ============================================================================

extern crate alloc;
pub use alloc::vec::Vec;

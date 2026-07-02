// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Example Solana-style contract that compiles to NeoVM via WASM
//!
//! This demonstrates how contracts written using Solana-compatible APIs
//! can be cross-compiled to run on Neo blockchain.

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

#[cfg(not(test))]
use core::panic::PanicInfo;

// Install the shared wasm32 bump allocator from the compat layer so heap
// allocations (Vec, String, ...) work instead of failing at runtime.
neo_solana_compat::neo_bump_allocator!();

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Use the Neo-Solana compatibility layer
use neo_solana_compat::prelude::*;
use neo_solana_compat::syscalls;

// Define the entrypoint
#[cfg(not(test))]
neo_solana_compat::entrypoint!(process_instruction);

/// Main entry point for the program
pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Log a message
    syscalls::sol_log("Hello from Solana-style contract on Neo!");

    // Check instruction data
    if instruction_data.is_empty() {
        syscalls::sol_log("No instruction data provided");
        return Err(ProgramError::InvalidInstructionData);
    }

    // Process based on instruction
    match instruction_data[0] {
        0 => {
            // Initialize
            syscalls::sol_log("Initialize instruction");
        }
        1 => {
            // Store data
            syscalls::sol_log("Store instruction");
            if instruction_data.len() > 1 {
                syscalls::storage_write(b"data", &instruction_data[1..]);
            }
        }
        2 => {
            // Read data
            syscalls::sol_log("Read instruction");
            let mut buffer = [0u8; 256];
            let _len = syscalls::storage_read(b"data", &mut buffer);
        }
        _ => {
            syscalls::sol_log("Unknown instruction");
            return Err(ProgramError::InvalidInstructionData);
        }
    }

    Ok(())
}

/// Alternative Neo-native entry point
/// This demonstrates the simpler Neo-style contract interface
#[no_mangle]
pub extern "C" fn hello() {
    syscalls::sol_log("Hello Neo from Solana code!");
}

/// Get the current time
#[no_mangle]
pub extern "C" fn get_time() -> i64 {
    syscalls::sol_get_clock_sysvar()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn program_id() -> Pubkey {
        Pubkey::new_default()
    }

    #[test]
    fn process_instruction_rejects_empty_payload() {
        let result = process_instruction(&program_id(), &[], &[]);
        assert_eq!(result, Err(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn process_instruction_accepts_supported_tags() {
        assert_eq!(process_instruction(&program_id(), &[], &[0]), Ok(()));
        assert_eq!(process_instruction(&program_id(), &[], &[1]), Ok(()));
        assert_eq!(process_instruction(&program_id(), &[], &[1, 42]), Ok(()));
        assert_eq!(process_instruction(&program_id(), &[], &[2]), Ok(()));
    }

    #[test]
    fn process_instruction_rejects_unknown_tag() {
        let result = process_instruction(&program_id(), &[], &[99]);
        assert_eq!(result, Err(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn native_entry_helpers_are_callable() {
        hello();
        assert_eq!(get_time(), 0);
    }
}

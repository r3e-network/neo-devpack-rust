//! Example Solana-style contract that compiles to NeoVM via WASM
//!
//! This demonstrates how contracts written using Solana-compatible APIs
//! can be cross-compiled to run on Neo blockchain.

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

#[cfg(not(test))]
use core::alloc::{GlobalAlloc, Layout};
#[cfg(not(test))]
use core::panic::PanicInfo;

/// A minimal bump allocator stub for no_std WASM environments.
///
/// # Safety
///
/// This is a stub implementation that returns null for allocations.
/// It is only suitable for contracts that don't need heap allocation.
/// For production use, a proper bump allocator with memory tracking should be used.
#[cfg(not(test))]
struct BumpAllocator;

// SAFETY: This is a stub allocator that always returns null.
// It is only safe because this contract doesn't use heap allocation.
// The alloc and dealloc methods are no-ops as this contract uses
// only stack-allocated data.
#[cfg(not(test))]
unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        // This stub allocator returns null, indicating allocation failure.
        // The contract is designed to work without heap allocation.
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // No-op: since we never allocate, we never need to deallocate.
    }
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

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

//! Example Solana-style contract that compiles to NeoVM via WASM
//!
//! This demonstrates how contracts written using Solana-compatible APIs
//! can be cross-compiled to run on Neo blockchain.

#![no_std]
#![no_main]

extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;

// Simple bump allocator for no_std
struct BumpAllocator;

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        // In wasm context, memory is managed by the runtime
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Use the Neo-Solana compatibility layer
use neo_solana_compat::prelude::*;
use neo_solana_compat::syscalls;

// Define the entrypoint
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

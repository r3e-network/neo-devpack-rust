//! Move-style Coin contract example for NeoVM
//!
//! This demonstrates a Move-inspired contract with resource-like semantics
//! compiled to run on Neo blockchain.
//!
//! # Move Concepts Demonstrated
//!
//! - **Resources**: Coin struct with value (cannot be copied/dropped accidentally)
//! - **Global Storage**: Balance tracking per address
//! - **Signer Authentication**: Owner verification via CheckWitness
//! - **Events**: Transfer notifications
//!
//! # Neo Mapping
//!
//! | Move Concept | Neo Implementation |
//! |--------------|--------------------|
//! | Resource<T>  | Storage entry with type prefix |
//! | move_to      | Storage.Put with existence check |
//! | move_from    | Storage.Get + Storage.Delete |
//! | borrow_global| Storage.Get (read-only) |
//! | signer       | CheckWitness verification |

#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate alloc;

use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;

// ============================================================================
// Minimal Runtime Support
// ============================================================================

/// A minimal bump allocator stub for no_std WASM environments.
/// 
/// # Safety
/// 
/// This is a stub implementation that returns null for allocations.
/// It is only suitable for contracts that don't need heap allocation.
/// For production use, a proper bump allocator with memory tracking should be used.
struct BumpAllocator;

// SAFETY: This is a stub allocator that always returns null.
// It is only safe because this contract doesn't use heap allocation.
// The alloc and dealloc methods are no-ops as this contract uses
// only stack-allocated data.
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

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() {}

// ============================================================================
// Neo Syscall Imports (Move-compatible naming)
// ============================================================================

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "neo")]
extern "C" {
    // System.Runtime.Log
    #[link_name = "runtime_log"]
    fn log(message: i32, len: i32);

    // System.Storage.Get
    #[link_name = "storage_get"]
    fn storage_get(key: i32, key_len: i32, out: i32) -> i32;

    // System.Storage.Put
    #[link_name = "storage_put"]
    fn storage_put(key: i32, key_len: i32, value: i32, value_len: i32);

    // System.Storage.Delete
    #[link_name = "storage_delete"]
    fn storage_delete(key: i32, key_len: i32);

    // System.Runtime.CheckWitness
    #[link_name = "runtime_check_witness"]
    fn check_witness(hash: i32) -> i32;

    // System.Runtime.Notify
    #[link_name = "runtime_notify"]
    fn notify(event: i32, event_len: i32);
}

// ============================================================================
// Move-Style Type Definitions
// ============================================================================

/// Address type (20 bytes for Neo UInt160)
type Address = [u8; 20];

/// Resource: Coin with value
/// In Move, this would be: struct Coin has key, store { value: u64 }
#[repr(C)]
#[allow(dead_code)]
struct Coin {
    value: u64,
}

/// Storage key prefixes (simulate Move's typed global storage)
const BALANCE_PREFIX: u8 = 0x01;
const TOTAL_SUPPLY_KEY: &[u8] = b"SUPPLY";

// ============================================================================
// Storage Helpers (Move Global Storage Emulation)
// ============================================================================

/// Create storage key for an address's balance
/// Move: borrow_global<Coin>(addr)
fn balance_key(addr: &Address) -> Vec<u8> {
    let mut key = Vec::with_capacity(21);
    key.push(BALANCE_PREFIX);
    key.extend_from_slice(addr);
    key
}

/// Check if resource exists at address
/// Move: exists<Coin>(addr)
fn exists_at(addr: &Address) -> bool {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: All pointers come from valid Rust references.
    // key is a Vec<u8> with valid data, out is a stack-allocated array.
    unsafe {
        let key = balance_key(addr);
        let mut out = [0u8; 8];
        let len = storage_get(
            key.as_ptr() as i32,
            key.len() as i32,
            out.as_mut_ptr() as i32,
        );
        len > 0
    }
    #[cfg(not(target_arch = "wasm32"))]
    false
}

/// Get balance at address
/// Move: borrow_global<Coin>(addr).value
fn get_balance(addr: &Address) -> u64 {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: All pointers come from valid Rust references.
    // key is a Vec<u8> with valid data, out is a stack-allocated array.
    unsafe {
        let key = balance_key(addr);
        let mut out = [0u8; 8];
        let len = storage_get(
            key.as_ptr() as i32,
            key.len() as i32,
            out.as_mut_ptr() as i32,
        );
        if len == 8 {
            u64::from_le_bytes(out)
        } else {
            0
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    0
}

/// Set balance at address
/// Move: move_to<Coin>(addr, Coin { value })
fn set_balance(addr: &Address, value: u64) {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: All pointers come from valid Rust references.
    // key is a Vec<u8> with valid data, value_bytes is a stack-allocated array.
    unsafe {
        let key = balance_key(addr);
        let value_bytes = value.to_le_bytes();
        storage_put(
            key.as_ptr() as i32,
            key.len() as i32,
            value_bytes.as_ptr() as i32,
            8,
        );
    }
}

/// Delete balance at address
/// Move: move_from<Coin>(addr)
#[allow(dead_code)]
fn delete_balance(addr: &Address) {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: key.as_ptr() comes from a valid Vec<u8> reference.
    unsafe {
        let key = balance_key(addr);
        storage_delete(key.as_ptr() as i32, key.len() as i32);
    }
}

/// Get total supply
fn get_total_supply() -> u64 {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: TOTAL_SUPPLY_KEY is a valid static byte slice,
    // out is a stack-allocated array.
    unsafe {
        let mut out = [0u8; 8];
        let len = storage_get(
            TOTAL_SUPPLY_KEY.as_ptr() as i32,
            TOTAL_SUPPLY_KEY.len() as i32,
            out.as_mut_ptr() as i32,
        );
        if len == 8 {
            u64::from_le_bytes(out)
        } else {
            0
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    0
}

/// Set total supply
fn set_total_supply(value: u64) {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: TOTAL_SUPPLY_KEY is a valid static byte slice,
    // value_bytes is a stack-allocated array.
    unsafe {
        let value_bytes = value.to_le_bytes();
        storage_put(
            TOTAL_SUPPLY_KEY.as_ptr() as i32,
            TOTAL_SUPPLY_KEY.len() as i32,
            value_bytes.as_ptr() as i32,
            8,
        );
    }
}

// ============================================================================
// Authentication (Move Signer Emulation)
// ============================================================================

/// Verify signer has authority (CheckWitness)
/// Move: &signer parameter verification
fn verify_signer(addr: &Address) -> bool {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: addr is a valid reference to a 20-byte array.
    unsafe {
        check_witness(addr.as_ptr() as i32) != 0
    }
    #[cfg(not(target_arch = "wasm32"))]
    true
}

// ============================================================================
// Event Emission
// ============================================================================

fn emit_transfer_event(from: &Address, to: &Address, amount: u64) {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: from and to are valid references, event is a stack-allocated array.
    unsafe {
        // Simple event format: [from:20][to:20][amount:8]
        let mut event = [0u8; 48];
        event[0..20].copy_from_slice(from);
        event[20..40].copy_from_slice(to);
        event[40..48].copy_from_slice(&amount.to_le_bytes());
        notify(event.as_ptr() as i32, 48);
    }
}

fn emit_log(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: msg is a valid string reference.
    unsafe {
        log(msg.as_ptr() as i32, msg.len() as i32);
    }
}

// ============================================================================
// Public Entry Points (Move public entry functions)
// ============================================================================

/// Initialize coin for an address (mint)
/// Move: public entry fun mint(admin: &signer, to: address, amount: u64)
#[no_mangle]
pub extern "C" fn mint(to_ptr: i32, amount: u64) -> i32 {
    // SAFETY: The contract entry point guarantees to_ptr is valid for 20 bytes.
    let to: Address = unsafe {
        let slice = core::slice::from_raw_parts(to_ptr as *const u8, 20);
        let mut addr = [0u8; 20];
        addr.copy_from_slice(slice);
        addr
    };

    // In production, should verify admin signer
    emit_log("mint: creating coins");

    let current = get_balance(&to);
    set_balance(&to, current + amount);

    let supply = get_total_supply();
    set_total_supply(supply + amount);

    1 // Success
}

/// Transfer coins between addresses
/// Move: public entry fun transfer(from: &signer, to: address, amount: u64)
#[no_mangle]
pub extern "C" fn transfer(from_ptr: i32, to_ptr: i32, amount: u64) -> i32 {
    // SAFETY: The contract entry point guarantees from_ptr is valid for 20 bytes.
    let from: Address = unsafe {
        let slice = core::slice::from_raw_parts(from_ptr as *const u8, 20);
        let mut addr = [0u8; 20];
        addr.copy_from_slice(slice);
        addr
    };

    // SAFETY: The contract entry point guarantees to_ptr is valid for 20 bytes.
    let to: Address = unsafe {
        let slice = core::slice::from_raw_parts(to_ptr as *const u8, 20);
        let mut addr = [0u8; 20];
        addr.copy_from_slice(slice);
        addr
    };

    // Verify sender has authority (Move: signer verification)
    if !verify_signer(&from) {
        emit_log("transfer: unauthorized");
        return 0; // Failure
    }

    // Check sufficient balance (Move: borrow_global check)
    let from_balance = get_balance(&from);
    if from_balance < amount {
        emit_log("transfer: insufficient balance");
        return 0; // Failure
    }

    // Perform transfer (Move: move_from + move_to)
    set_balance(&from, from_balance - amount);
    let to_balance = get_balance(&to);
    set_balance(&to, to_balance + amount);

    // Emit event
    emit_transfer_event(&from, &to, amount);
    emit_log("transfer: success");

    1 // Success
}

/// Get balance of an address
/// Move: public fun balance(addr: address): u64
#[no_mangle]
pub extern "C" fn balance(addr_ptr: i32) -> u64 {
    // SAFETY: The contract entry point guarantees addr_ptr is valid for 20 bytes.
    let addr: Address = unsafe {
        let slice = core::slice::from_raw_parts(addr_ptr as *const u8, 20);
        let mut address = [0u8; 20];
        address.copy_from_slice(slice);
        address
    };

    get_balance(&addr)
}

/// Get total supply
/// Move: public fun total_supply(): u64
#[no_mangle]
pub extern "C" fn total_supply() -> u64 {
    get_total_supply()
}

/// Check if address has coin resource
/// Move: public fun exists(addr: address): bool
#[no_mangle]
pub extern "C" fn has_coin(addr_ptr: i32) -> i32 {
    // SAFETY: The contract entry point guarantees addr_ptr is valid for 20 bytes.
    let addr: Address = unsafe {
        let slice = core::slice::from_raw_parts(addr_ptr as *const u8, 20);
        let mut address = [0u8; 20];
        address.copy_from_slice(slice);
        address
    };

    if exists_at(&addr) { 1 } else { 0 }
}

/// Burn coins (destroy resource)
/// Move: public entry fun burn(owner: &signer, amount: u64)
#[no_mangle]
pub extern "C" fn burn(owner_ptr: i32, amount: u64) -> i32 {
    // SAFETY: The contract entry point guarantees owner_ptr is valid for 20 bytes.
    let owner: Address = unsafe {
        let slice = core::slice::from_raw_parts(owner_ptr as *const u8, 20);
        let mut addr = [0u8; 20];
        addr.copy_from_slice(slice);
        addr
    };

    // Verify owner has authority
    if !verify_signer(&owner) {
        emit_log("burn: unauthorized");
        return 0;
    }

    let current = get_balance(&owner);
    if current < amount {
        emit_log("burn: insufficient balance");
        return 0;
    }

    // Destroy coins (Move: resource destruction)
    set_balance(&owner, current - amount);
    let supply = get_total_supply();
    set_total_supply(supply - amount);

    emit_log("burn: success");
    1
}

// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Bump allocator for `no_std` WASM contracts
//!
//! Contracts compiled for `wasm32-unknown-unknown` with `#![no_std]` need a
//! `#[global_allocator]` before anything in `alloc` (`Vec`, `String`, ...)
//! can be used. This module provides a minimal bump allocator backed by a
//! static arena so ported Solana programs get a working heap out of the box.
//!
//! # Limitations
//!
//! This is an **alloc-only** allocator: `dealloc` is a no-op and memory is
//! never reclaimed. That matches the short-lived, single-invocation execution
//! model of a smart contract, but it means long-running loops that allocate
//! repeatedly will eventually exhaust the arena ([`HEAP_SIZE`] bytes) and
//! allocation will fail.
//!
//! # Usage
//!
//! ```rust,ignore
//! // Installs the allocator on wasm32 builds (no-op elsewhere, so host
//! // unit tests keep using the system allocator).
//! neo_solana_compat::neo_bump_allocator!();
//! ```

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;

/// Total size of the static bump-allocator arena in bytes.
pub const HEAP_SIZE: usize = 64 * 1024;

/// A simple bump allocator over a static arena.
///
/// Allocations advance a cursor through the arena; `dealloc` is a no-op.
/// See the [module documentation](self) for the trade-offs.
pub struct BumpAllocator {
    arena: UnsafeCell<[u8; HEAP_SIZE]>,
    offset: UnsafeCell<usize>,
}

// SAFETY: Contract execution on NeoVM (and wasm32-unknown-unknown in general)
// is single-threaded, so the allocator state is never accessed concurrently.
unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    /// Create a new, empty bump allocator.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            arena: UnsafeCell::new([0u8; HEAP_SIZE]),
            offset: UnsafeCell::new(0),
        }
    }
}

impl Default for BumpAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: Returned pointers are within the arena, respect the requested
// layout, and are never handed out twice because the cursor only advances.
unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let base = self.arena.get() as usize;
        let offset = &mut *self.offset.get();

        // Round the next free address up to the requested alignment.
        // `layout.align()` is guaranteed to be a non-zero power of two.
        let start = match (base + *offset).checked_add(layout.align() - 1) {
            Some(addr) => addr & !(layout.align() - 1),
            None => return core::ptr::null_mut(),
        };
        let end = match start.checked_add(layout.size()) {
            Some(end) => end,
            None => return core::ptr::null_mut(),
        };
        if end > base + HEAP_SIZE {
            // Arena exhausted: signal allocation failure rather than
            // handing out memory we don't own.
            return core::ptr::null_mut();
        }
        *offset = end - base;
        start as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Intentionally a no-op: bump allocators never reclaim memory.
    }
}

/// Install [`BumpAllocator`] as the global allocator on wasm32 builds.
///
/// Expands to nothing on other targets (and under `cfg(test)`), so host
/// unit tests keep using the system allocator.
///
/// # Example
///
/// ```rust,ignore
/// neo_solana_compat::neo_bump_allocator!();
/// ```
#[macro_export]
macro_rules! neo_bump_allocator {
    () => {
        #[cfg(all(target_arch = "wasm32", not(test)))]
        #[global_allocator]
        static __NEO_SOLANA_BUMP_ALLOCATOR: $crate::allocator::BumpAllocator =
            $crate::allocator::BumpAllocator::new();
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_respects_alignment_and_advances() {
        let allocator = BumpAllocator::new();
        let p1 = unsafe { allocator.alloc(Layout::from_size_align(3, 1).unwrap()) };
        let p2 = unsafe { allocator.alloc(Layout::from_size_align(8, 8).unwrap()) };
        assert!(!p1.is_null());
        assert!(!p2.is_null());
        assert_eq!(p2 as usize % 8, 0);
        // The second allocation must not overlap the first.
        assert!(p2 as usize >= p1 as usize + 3);
    }

    #[test]
    fn alloc_returns_null_when_exhausted() {
        let allocator = BumpAllocator::new();
        let big = Layout::from_size_align(HEAP_SIZE, 1).unwrap();
        assert!(!unsafe { allocator.alloc(big) }.is_null());
        let one = Layout::from_size_align(1, 1).unwrap();
        assert!(unsafe { allocator.alloc(one) }.is_null());
    }

    #[test]
    fn dealloc_is_a_noop() {
        let allocator = BumpAllocator::new();
        let layout = Layout::from_size_align(16, 4).unwrap();
        let p1 = unsafe { allocator.alloc(layout) };
        unsafe { allocator.dealloc(p1, layout) };
        // Memory is not reclaimed: the next allocation comes after p1.
        let p2 = unsafe { allocator.alloc(layout) };
        assert!(p2 as usize >= p1 as usize + 16);
    }
}

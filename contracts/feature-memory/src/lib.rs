// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Feature-coverage sample: heap allocation (the std `dlmalloc` allocator on
//! wasm32), globals (`static` / `static mut`), and bulk memory ops
//! (Vec growth, fill, copy, slices). Allocates by design.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use neo_devpack::prelude::*;

neo_manifest_overlay!(r#"{ "name": "FeatureMemory" }"#);

#[neo_contract]
pub struct MemoryContract;

static SEED: i64 = 100;
static mut COUNTER: i64 = 0;

#[neo_contract]
impl MemoryContract {
    pub fn new() -> Self {
        Self
    }

    /// Heap `Vec` allocation + growth (memory.grow) + iteration/sum.
    #[neo_method(safe)]
    pub fn vec_sum(n: i64) -> i64 {
        let cap = n.clamp(0, 4096) as usize;
        let mut v: Vec<i64> = Vec::new();
        for i in 0..cap as i64 {
            v.push(i.wrapping_mul(2).wrapping_add(1));
        }
        v.iter().copied().fold(0i64, i64::wrapping_add)
    }

    /// `vec![v; n]` fill (memory.fill) + slice length.
    #[neo_method(safe)]
    pub fn byte_fill(n: i64, v: i64) -> i64 {
        let len = n.clamp(0, 8192) as usize;
        let buf = vec![v as u8; len];
        buf.iter().map(|&b| b as i64).sum()
    }

    /// Copy between two heap buffers (memory.copy via copy_from_slice).
    #[neo_method(safe)]
    pub fn copy_blob(n: i64) -> i64 {
        let len = n.clamp(0, 8192) as usize;
        let src: Vec<u8> = (0..len).map(|i| (i & 0xFF) as u8).collect();
        let mut dst = vec![0u8; len];
        dst.copy_from_slice(&src);
        dst.iter().map(|&b| b as i64).sum()
    }

    /// Read an immutable global (`static`) via a volatile read (global.get).
    #[neo_method(safe)]
    pub fn global_read() -> i64 {
        unsafe { core::ptr::read_volatile(&SEED) }
    }

    /// Mutate a `static mut` global (global.set) — the NeoVM is
    /// single-threaded so this is well-defined within an invocation.
    #[neo_method]
    pub fn global_mut(x: i64) -> i64 {
        unsafe {
            let c = core::ptr::addr_of_mut!(COUNTER);
            *c = (*c).wrapping_add(x);
            *c
        }
    }

    /// Stack array + slice indexing/iteration (no heap).
    #[neo_method(safe)]
    pub fn slice_work(n: i64) -> i64 {
        let arr: [i64; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let k = (n.rem_euclid(8)) as usize;
        let slice = &arr[..=k];
        slice.iter().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_methods() {
        assert_eq!(MemoryContract::vec_sum(5), 1 + 3 + 5 + 7 + 9);
        assert_eq!(MemoryContract::byte_fill(4, 2), 8);
        assert_eq!(MemoryContract::copy_blob(3), 0 + 1 + 2);
        assert_eq!(MemoryContract::global_read(), 100);
        assert_eq!(MemoryContract::slice_work(2), 1 + 2 + 3);
    }
}

// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Storage convenience helpers built on top of the syscall layer.
//!
//! This module exposes two facades:
//!
//! - [`NeoStorage`] is the byte-string-typed API used by the host-side test
//!   harness and by contracts that already manage `NeoByteString`/`Vec<u8>`
//!   storage values themselves. It allocates through the standard Rust
//!   allocator and is best suited to host (`cfg(not(target_arch = "wasm32"))`)
//!   builds.
//!
//! - [`RawStorage`] is a heap-free facade that takes plain `&[u8]` slices and
//!   writes results into caller-supplied buffers. On `wasm32` it lowers
//!   directly to the translator-emitted Neo storage syscall helpers without
//!   ever touching the wasm allocator. Production smart contracts that run on
//!   Neo Express should prefer this path: it sidesteps the dlmalloc bookkeeping
//!   that the wasm-to-NeoVM translator does not currently materialise on the
//!   contract's NeoVM stack, so storage-heavy state transitions (multisig,
//!   escrow, crowdfund, etc.) stay deploy-and-invoke-able rather than
//!   "deploy-only".

use neo_syscalls::NeoVMSyscall;
use neo_types::*;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "neo")]
extern "C" {
    #[link_name = "neo_storage_put_bytes"]
    fn neo_storage_put_bytes(key_ptr: i32, key_len: i32, value_ptr: i32, value_len: i32);

    #[link_name = "neo_storage_delete_bytes"]
    fn neo_storage_delete_bytes(key_ptr: i32, key_len: i32);

    #[link_name = "neo_storage_get_into"]
    fn neo_storage_get_into(key_ptr: i32, key_len: i32, out_ptr: i32, out_cap: i32) -> i32;
}

/// Storage convenience helpers built on top of the syscall layer.
pub struct NeoStorage;

impl NeoStorage {
    pub fn get_context() -> NeoResult<NeoStorageContext> {
        NeoVMSyscall::storage_get_context()
    }

    pub fn get_read_only_context() -> NeoResult<NeoStorageContext> {
        NeoVMSyscall::storage_get_read_only_context()
    }

    pub fn as_read_only(context: &NeoStorageContext) -> NeoResult<NeoStorageContext> {
        NeoVMSyscall::storage_as_read_only(context)
    }

    pub fn get(context: &NeoStorageContext, key: &NeoByteString) -> NeoResult<NeoByteString> {
        NeoVMSyscall::storage_get(context, key)
    }

    pub fn put(
        context: &NeoStorageContext,
        key: &NeoByteString,
        value: &NeoByteString,
    ) -> NeoResult<()> {
        NeoVMSyscall::storage_put(context, key, value)
    }

    pub fn delete(context: &NeoStorageContext, key: &NeoByteString) -> NeoResult<()> {
        NeoVMSyscall::storage_delete(context, key)
    }

    pub fn find(
        context: &NeoStorageContext,
        prefix: &NeoByteString,
    ) -> NeoResult<NeoIterator<NeoValue>> {
        NeoVMSyscall::storage_find(context, prefix)
    }
}

/// Heap-free storage facade that operates on `&[u8]` slices.
///
/// `wasm32` lowers each call to the translator's `System.Storage.*` SYSCALL
/// helpers directly, so contracts that use this path do not depend on the
/// Rust allocator being functional inside NeoVM. Host (non-wasm32) builds
/// route through the existing `NeoVMSyscall` simulation so unit tests behave
/// the same as on wasm32.
pub struct RawStorage;

/// Outcome of [`RawStorage::get_into`].
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RawStorageGet {
    /// Value was found and fully written into the caller buffer; the contained
    /// `usize` is the number of bytes written.
    Found(usize),
    /// Key was not present in the contract's storage namespace.
    Missing,
    /// Value exists but is larger than the caller buffer; the contained
    /// `usize` is the byte length the caller must allocate before retrying.
    BufferTooSmall(usize),
}

impl RawStorage {
    /// Write `value` to `key` in the executing contract's persistent storage.
    pub fn put(key: &[u8], value: &[u8]) {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            neo_storage_put_bytes(
                key.as_ptr() as i32,
                key.len() as i32,
                value.as_ptr() as i32,
                value.len() as i32,
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let ctx = match NeoVMSyscall::storage_get_context() {
                Ok(c) => c,
                Err(_) => return,
            };
            let _ = NeoVMSyscall::storage_put(
                &ctx,
                &NeoByteString::from_slice(key),
                &NeoByteString::from_slice(value),
            );
        }
    }

    /// Delete `key` from the executing contract's persistent storage.
    pub fn delete(key: &[u8]) {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            neo_storage_delete_bytes(key.as_ptr() as i32, key.len() as i32);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let ctx = match NeoVMSyscall::storage_get_context() {
                Ok(c) => c,
                Err(_) => return,
            };
            let _ = NeoVMSyscall::storage_delete(&ctx, &NeoByteString::from_slice(key));
        }
    }

    /// Read the value at `key` into `buf`.
    ///
    /// Returns one of:
    /// - [`RawStorageGet::Found`] with the byte count actually written into
    ///   `buf` when the key is present and the value fits.
    /// - [`RawStorageGet::Missing`] when the key is not in storage.
    /// - [`RawStorageGet::BufferTooSmall`] with the value's true length when
    ///   `buf` cannot hold it; the value bytes are NOT copied in this case.
    pub fn get_into(key: &[u8], buf: &mut [u8]) -> RawStorageGet {
        #[cfg(target_arch = "wasm32")]
        let actual = unsafe {
            neo_storage_get_into(
                key.as_ptr() as i32,
                key.len() as i32,
                buf.as_mut_ptr() as i32,
                buf.len() as i32,
            )
        };
        #[cfg(not(target_arch = "wasm32"))]
        let actual = host_get_into(key, buf);

        if actual == -1 {
            RawStorageGet::Missing
        } else if actual >= 0 {
            RawStorageGet::Found(actual as usize)
        } else {
            RawStorageGet::BufferTooSmall((-actual) as usize)
        }
    }

    /// Read an exact 8-byte little-endian `i64` at `key`. Returns `None` for
    /// missing keys or for stored values whose length is not exactly 8.
    pub fn get_i64(key: &[u8]) -> Option<i64> {
        let mut buf = [0u8; 8];
        match Self::get_into(key, &mut buf) {
            RawStorageGet::Found(8) => Some(i64::from_le_bytes(buf)),
            _ => None,
        }
    }

    /// Read an exact 2-byte little-endian `u16` at `key`. Returns `None` for
    /// missing keys or for stored values whose length is not exactly 2.
    pub fn get_u16(key: &[u8]) -> Option<u16> {
        let mut buf = [0u8; 2];
        match Self::get_into(key, &mut buf) {
            RawStorageGet::Found(2) => Some(u16::from_le_bytes(buf)),
            _ => None,
        }
    }

    /// Read an exact 1-byte boolean at `key`. Returns `None` for missing keys
    /// or for stored values whose length is not exactly 1.
    pub fn get_bool(key: &[u8]) -> Option<bool> {
        let mut buf = [0u8; 1];
        match Self::get_into(key, &mut buf) {
            RawStorageGet::Found(1) => Some(buf[0] != 0),
            _ => None,
        }
    }

    /// Convenience: store an `i64` little-endian at `key`.
    pub fn put_i64(key: &[u8], value: i64) {
        Self::put(key, &value.to_le_bytes());
    }

    /// Convenience: store a `u16` little-endian at `key`.
    pub fn put_u16(key: &[u8], value: u16) {
        Self::put(key, &value.to_le_bytes());
    }

    /// Convenience: store a `bool` (encoded as a single 0/1 byte) at `key`.
    pub fn put_bool(key: &[u8], value: bool) {
        Self::put(key, &[value as u8]);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn host_get_into(key: &[u8], buf: &mut [u8]) -> i32 {
    let ctx = match NeoVMSyscall::storage_get_context() {
        Ok(c) => c,
        Err(_) => return -1,
    };
    let stored = match NeoVMSyscall::storage_get(&ctx, &NeoByteString::from_slice(key)) {
        Ok(b) => b,
        Err(_) => return -1,
    };
    let bytes = stored.as_slice();
    if bytes.is_empty() {
        return -1;
    }
    if bytes.len() > buf.len() {
        return -(bytes.len() as i32);
    }
    let len = bytes.len();
    buf[..len].copy_from_slice(bytes);
    len as i32
}

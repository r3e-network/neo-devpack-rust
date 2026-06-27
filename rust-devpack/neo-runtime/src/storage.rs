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

    #[link_name = "raw_storage_put_i64"]
    fn neo_raw_storage_put_i64(key: i64, value: i64);

    #[link_name = "raw_storage_get_i64"]
    fn neo_raw_storage_get_i64(key: i64) -> i64;

    #[link_name = "raw_storage_has_i64"]
    fn neo_raw_storage_has_i64(key: i64) -> i32;

    #[link_name = "raw_storage_delete_i64"]
    fn neo_raw_storage_delete_i64(key: i64);
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

    /// Read a stored value.
    ///
    /// **Ambiguity warning (D8):** this returns an empty `NeoByteString` both
    /// when the key is absent AND when the stored value is genuinely empty, so
    /// the two cases are indistinguishable. For existence-sensitive reads,
    /// prefer `NeoVMSyscall::storage_try_get` (returns `Option`) or
    /// `RawStorage::get_into` (returns `RawStorageGet::Missing`).
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
    /// The runtime explicitly reported a null/missing value. Neo N3 storage
    /// commonly surfaces absent keys as an empty byte string instead, so
    /// callers must not rely on this variant for existence checks.
    Missing,
    /// Value exists but is larger than the caller buffer; the contained
    /// `usize` is the byte length the caller must allocate before retrying.
    BufferTooSmall(usize),
}

/// Fixed-capacity stack key builder for `RawStorage` keys.
///
/// This keeps contract samples on a heap-free path while centralizing the
/// small `copy_nonoverlapping` block that fixed key construction needs.
/// Push methods return `false` when the requested write would exceed capacity;
/// existing bytes are left unchanged in that case.
pub struct RawKeyBuilder<const N: usize> {
    buf: core::mem::MaybeUninit<[u8; N]>,
    len: usize,
}

impl<const N: usize> RawKeyBuilder<N> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            buf: core::mem::MaybeUninit::uninit(),
            len: 0,
        }
    }

    #[inline(always)]
    pub fn push_bytes(&mut self, bytes: &[u8]) -> bool {
        if bytes.len() > N - self.len {
            return false;
        }
        unsafe {
            core::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                self.buf.as_mut_ptr().cast::<u8>().add(self.len),
                bytes.len(),
            );
        }
        self.len += bytes.len();
        true
    }

    #[inline(always)]
    pub fn push_i64_le(&mut self, value: i64) -> bool {
        self.push_bytes(&value.to_le_bytes())
    }

    #[inline(always)]
    pub fn push_byte(&mut self, value: u8) -> bool {
        if self.len == N {
            return false;
        }
        unsafe {
            *self.buf.as_mut_ptr().cast::<u8>().add(self.len) = value;
        }
        self.len += 1;
        true
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        debug_assert!(self.len <= N);
        unsafe { core::slice::from_raw_parts(self.buf.as_ptr().cast::<u8>(), self.len) }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<const N: usize> Default for RawKeyBuilder<N> {
    fn default() -> Self {
        Self::new()
    }
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
    /// - [`RawStorageGet::Missing`] only when the runtime explicitly reports
    ///   null/missing; Neo N3 commonly returns zero bytes for absent keys.
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

    /// Store an `i64` value under an `i64` key without touching wasm linear
    /// memory. On wasm32 this lowers directly to `System.Storage.Put`.
    pub fn put_i64_key(key: i64, value: i64) {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            neo_raw_storage_put_i64(key, value);
        }
        #[cfg(not(target_arch = "wasm32"))]
        host_put_i64_key(key, value);
    }

    /// Read an `i64` value from an `i64` key. Missing keys return `0`.
    pub fn get_i64_key_or_zero(key: i64) -> i64 {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            neo_raw_storage_get_i64(key)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            host_get_i64_key(key).unwrap_or(0)
        }
    }

    /// Check whether an `i64` key has a stored integer value.
    ///
    /// Neo Express surfaces absent direct keys as empty bytes. The translator
    /// therefore treats any non-empty `Storage.Get` result as present.
    pub fn has_i64_key(key: i64) -> bool {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            neo_raw_storage_has_i64(key) != 0
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            host_has_i64_key(key)
        }
    }

    /// Delete an `i64` key without touching wasm linear memory.
    pub fn delete_i64_key(key: i64) {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            neo_raw_storage_delete_i64(key);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let ctx = match NeoVMSyscall::storage_get_context() {
                Ok(c) => c,
                Err(_) => return,
            };
            let key_bytes = neovm_i64_bytes(key);
            let _ = NeoVMSyscall::storage_delete(&ctx, &NeoByteString::from_slice(&key_bytes));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn host_get_into(key: &[u8], buf: &mut [u8]) -> i32 {
    let ctx = match NeoVMSyscall::storage_get_context() {
        Ok(c) => c,
        Err(_) => return -1,
    };
    let stored = match NeoVMSyscall::storage_try_get(&ctx, &NeoByteString::from_slice(key)) {
        Ok(Some(b)) => b,
        // D14: return -1 for a missing key so the host path matches the wasm
        // path's `RawStorageGet::Missing` (was returning 0, i.e. Found(0), so
        // host and wasm disagreed on absence).
        Ok(None) => return -1,
        Err(_) => return -1,
    };
    let bytes = stored.as_slice();
    if bytes.len() > buf.len() {
        return -(bytes.len() as i32);
    }
    let len = bytes.len();
    buf[..len].copy_from_slice(bytes);
    len as i32
}

#[cfg(not(target_arch = "wasm32"))]
fn host_put_i64_key(key: i64, value: i64) {
    let ctx = match NeoVMSyscall::storage_get_context() {
        Ok(c) => c,
        Err(_) => return,
    };
    let key_bytes = neovm_i64_bytes(key);
    let value_bytes = neovm_i64_bytes(value);
    let _ = NeoVMSyscall::storage_put(
        &ctx,
        &NeoByteString::from_slice(&key_bytes),
        &NeoByteString::from_slice(&value_bytes),
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn host_get_i64_key(key: i64) -> Option<i64> {
    let ctx = NeoVMSyscall::storage_get_context().ok()?;
    let key_bytes = neovm_i64_bytes(key);
    let stored = NeoVMSyscall::storage_try_get(&ctx, &NeoByteString::from_slice(&key_bytes))
        .ok()
        .flatten()?;
    storage_bytes_to_i64(stored.as_slice())
}

#[cfg(not(target_arch = "wasm32"))]
fn host_has_i64_key(key: i64) -> bool {
    let ctx = match NeoVMSyscall::storage_get_context() {
        Ok(c) => c,
        Err(_) => return false,
    };
    let key_bytes = neovm_i64_bytes(key);
    NeoVMSyscall::storage_try_get(&ctx, &NeoByteString::from_slice(&key_bytes))
        .ok()
        .flatten()
        .map(|stored| !stored.as_slice().is_empty())
        .unwrap_or(false)
}

#[cfg(not(target_arch = "wasm32"))]
fn neovm_i64_bytes(value: i64) -> Vec<u8> {
    if value == 0 {
        return vec![0];
    }

    let mut bytes = value.to_le_bytes().to_vec();
    while bytes.len() > 1 {
        let last = *bytes.last().unwrap_or(&0);
        let prev = bytes[bytes.len() - 2];
        let redundant_positive = last == 0x00 && (prev & 0x80) == 0;
        let redundant_negative = last == 0xff && (prev & 0x80) != 0;
        if redundant_positive || redundant_negative {
            bytes.pop();
        } else {
            break;
        }
    }
    bytes
}

#[cfg(not(target_arch = "wasm32"))]
fn storage_bytes_to_i64(bytes: &[u8]) -> Option<i64> {
    match bytes.len() {
        0 => None,
        1..=8 => {
            let sign_extend = bytes.last().copied().unwrap_or(0) & 0x80 != 0;
            let mut buf = if sign_extend { [0xff; 8] } else { [0u8; 8] };
            buf[..bytes.len()].copy_from_slice(bytes);
            Some(i64::from_le_bytes(buf))
        }
        _ => None,
    }
}

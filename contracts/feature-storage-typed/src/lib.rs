// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Feature-coverage sample: the byte-string `NeoStorage` facade,
//! `NeoStorageContext` (read-only handling), and the low-level
//! `NeoVMSyscall::storage_*` surface. All ByteString work is inside method
//! bodies; exports are scalar.
//!
//! `NeoStorage::find` works on wasm32 through the prefix-scan bridge
//! (`System.Storage.Find` + `System.Iterator.Next/Value`, single live
//! VM iterator drained eagerly — see `neo-syscalls/src/syscalls_abi.rs`);
//! `scan_prefix` exercises it end-to-end and `find_count` covers the
//! empty-prefix full scan.

extern crate alloc;
use neo_devpack::prelude::*;
use neo_devpack::NeoVMSyscall;

neo_manifest_overlay!(r#"{ "name": "FeatureStorageTyped", "features": { "storage": true } }"#);

#[neo_contract]
pub struct StorageTypedContract;

fn key() -> NeoByteString {
    NeoByteString::from_slice(b"k")
}

#[neo_contract]
impl StorageTypedContract {
    pub fn new() -> Self {
        Self
    }

    /// NeoStorage::get_context + put.
    #[neo_method]
    pub fn ctx_put(v: i64) -> NeoResult<()> {
        let ctx = NeoStorage::get_context()?;
        let val = NeoByteString::from_slice(&v.to_le_bytes());
        NeoStorage::put(&ctx, &key(), &val)
    }

    /// NeoStorage::get (empty bytes == absent per Neo N3).
    #[neo_method(safe)]
    pub fn ctx_get() -> i64 {
        let ctx = match NeoStorage::get_context() {
            Ok(c) => c,
            Err(_) => return -1,
        };
        match NeoStorage::get(&ctx, &key()) {
            Ok(bs) if bs.len() == 8 => {
                let mut b = [0u8; 8];
                b.copy_from_slice(bs.as_slice());
                i64::from_le_bytes(b)
            }
            _ => -1,
        }
    }

    /// NeoStorage::delete.
    #[neo_method]
    pub fn ctx_delete() -> NeoResult<()> {
        let ctx = NeoStorage::get_context()?;
        NeoStorage::delete(&ctx, &key())
    }

    /// get_read_only_context + as_read_only + NeoStorageContext::{is_read_only,id}.
    #[neo_method(safe)]
    pub fn ro_check() -> bool {
        let ro = match NeoStorage::get_read_only_context() {
            Ok(c) => c,
            Err(_) => return false,
        };
        let _id = ro.id();
        let derived = ro.as_read_only();
        ro.is_read_only() && derived.is_read_only()
    }

    /// NeoStorage::find with an empty prefix (full-store scan): entry count.
    #[neo_method(safe)]
    pub fn find_count() -> i64 {
        let ctx = match NeoStorage::get_context() {
            Ok(c) => c,
            Err(_) => return -1,
        };
        match NeoStorage::find(&ctx, &NeoByteString::from_slice(b"")) {
            Ok(it) => it.len() as i64,
            Err(_) => -1,
        }
    }

    /// End-to-end prefix scan: puts 3 keys under `fp:` plus one outside,
    /// `NeoStorage::find(b"fp:")`s, and folds the entries positionally.
    ///
    /// Returns `count * 10_000 + Σ (position+1) * value` — 3 entries in
    /// ascending key order (`fp:a`=10, `fp:b`=20, `fp:c`=30) give
    /// `3 * 10_000 + (1*10 + 2*20 + 3*30) = 30_140`; the weighting proves
    /// the element ORDER, not just membership. Any entry whose key does not
    /// start with the prefix (or whose struct shape is wrong) returns a
    /// negative sentinel instead. Host mode and the real VM must agree on
    /// the exact value.
    #[neo_method]
    pub fn scan_prefix() -> i64 {
        let ctx = match NeoStorage::get_context() {
            Ok(c) => c,
            Err(_) => return -1,
        };
        let put = |k: &[u8], v: i64| {
            NeoStorage::put(
                &ctx,
                &NeoByteString::from_slice(k),
                &NeoByteString::from_slice(&v.to_le_bytes()),
            )
        };
        if put(b"fp:a", 10).is_err()
            || put(b"fp:b", 20).is_err()
            || put(b"fp:c", 30).is_err()
            || put(b"zz", 99).is_err()
        {
            return -2;
        }

        let iter = match NeoStorage::find(&ctx, &NeoByteString::from_slice(b"fp:")) {
            Ok(it) => it,
            Err(_) => return -3,
        };
        let mut count: i64 = 0;
        let mut weighted: i64 = 0;
        for entry in iter {
            let Some(st) = entry.as_struct().cloned() else {
                return -4;
            };
            let Some(key) = st.get_field("key").and_then(NeoValue::as_byte_string).cloned()
            else {
                return -5;
            };
            if !key.as_slice().starts_with(b"fp:") {
                return -6;
            }
            let Some(value) = st
                .get_field("value")
                .and_then(NeoValue::as_byte_string)
                .cloned()
            else {
                return -7;
            };
            if value.len() != 8 {
                return -8;
            }
            let mut b = [0u8; 8];
            b.copy_from_slice(value.as_slice());
            count += 1;
            weighted += count * i64::from_le_bytes(b);
        }
        count * 10_000 + weighted
    }

    /// Buffer-growth path of the iterator-value bridge: a single 200-byte
    /// value makes the flattened element (`4 + key_len + 200` bytes)
    /// overflow the SDK drain loop's initial 128-byte buffer, so the first
    /// `runtime_iterator_value` call must return `-needed_len` (without
    /// advancing the iterator) and the retry must deliver the full payload.
    ///
    /// Returns `value_len * 100_000 + Σ value_bytes` — bytes are `i % 251`
    /// for i in 0..200, so `200 * 100_000 + 19_900 = 20_019_900`. Host mode
    /// and the real VM must agree on the exact value.
    #[neo_method]
    pub fn scan_big_value() -> i64 {
        let ctx = match NeoStorage::get_context() {
            Ok(c) => c,
            Err(_) => return -1,
        };
        let mut big = [0u8; 200];
        for (i, b) in big.iter_mut().enumerate() {
            *b = (i % 251) as u8;
        }
        if NeoStorage::put(
            &ctx,
            &NeoByteString::from_slice(b"bp:k"),
            &NeoByteString::from_slice(&big),
        )
        .is_err()
        {
            return -2;
        }

        let mut iter = match NeoStorage::find(&ctx, &NeoByteString::from_slice(b"bp:")) {
            Ok(it) => it,
            Err(_) => return -3,
        };
        let Some(entry) = iter.next() else {
            return -4;
        };
        if iter.next().is_some() {
            return -5; // exactly one entry under the prefix
        }
        let Some(value) = entry
            .as_struct()
            .and_then(|st| st.get_field("value"))
            .and_then(NeoValue::as_byte_string)
            .cloned()
        else {
            return -6;
        };
        let mut sum: i64 = 0;
        for b in value.as_slice() {
            sum += i64::from(*b);
        }
        (value.len() as i64) * 100_000 + sum
    }

    /// Low-level NeoVMSyscall::storage_get_context + storage_put.
    #[neo_method]
    pub fn sys_put(v: i64) -> NeoResult<()> {
        let ctx = NeoVMSyscall::storage_get_context()?;
        let val = NeoByteString::from_slice(&v.to_le_bytes());
        NeoVMSyscall::storage_put(&ctx, &NeoByteString::from_slice(b"s"), &val)
    }

    /// NeoVMSyscall::storage_get.
    #[neo_method(safe)]
    pub fn sys_get() -> i64 {
        let ctx = match NeoVMSyscall::storage_get_context() {
            Ok(c) => c,
            Err(_) => return -1,
        };
        match NeoVMSyscall::storage_get(&ctx, &NeoByteString::from_slice(b"s")) {
            Ok(bs) => bs.len() as i64,
            Err(_) => -1,
        }
    }

    /// NeoVMSyscall::storage_delete.
    #[neo_method]
    pub fn sys_delete() -> NeoResult<()> {
        let ctx = NeoVMSyscall::storage_get_context()?;
        NeoVMSyscall::storage_delete(&ctx, &NeoByteString::from_slice(b"s"))
    }

    /// NeoVMSyscall::storage_get_read_only_context + storage_as_read_only.
    #[neo_method(safe)]
    pub fn sys_ro_ctx() -> bool {
        let ro = match NeoVMSyscall::storage_get_read_only_context() {
            Ok(c) => c,
            Err(_) => return false,
        };
        let derived = match NeoVMSyscall::storage_as_read_only(&ro) {
            Ok(c) => c,
            Err(_) => return false,
        };
        ro.is_read_only() && derived.is_read_only()
    }

    /// NeoStorageContext::new (direct handle construction).
    #[neo_method(safe)]
    pub fn ctx_new() -> i64 {
        NeoStorageContext::new(1).id() as i64
    }
}

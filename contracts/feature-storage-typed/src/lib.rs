// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Feature-coverage sample: the byte-string `NeoStorage` facade,
//! `NeoStorageContext` (read-only handling), and the low-level
//! `NeoVMSyscall::storage_*` surface. All ByteString work is inside method
//! bodies; exports are scalar.
//!
//! NOTE: `NeoStorage::find` compiles and emits the SYSCALL but yields an
//! empty iterator on wasm32 (prefix iteration is not bridged) — `find_count`
//! documents that. Real enumeration uses indexed keys (see feature-storage-raw).

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

    /// NeoStorage::find (host-functional; empty on wasm32 — returns 0 on-chain).
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

// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Feature-coverage sample: the heap-free `RawStorage` + `RawKeyBuilder` +
//! `RawStorageGet` surface (lowered to `System.Storage.Get/Put/Delete`).
//! Exercises typed put/get round-trips, the `get_into` outcome variants, and
//! fixed-capacity key construction.

use neo_devpack::prelude::*;

neo_manifest_overlay!(r#"{ "name": "FeatureStorageRaw", "features": { "storage": true } }"#);

#[neo_contract]
pub struct StorageRawContract;

#[neo_contract]
impl StorageRawContract {
    pub fn new() -> Self {
        Self
    }

    /// `RawStorage::put` of a raw byte blob.
    #[neo_method]
    pub fn put_blob(seed: i64) {
        RawStorage::put(b"blob", &seed.to_le_bytes());
    }

    /// `RawStorage::delete`.
    #[neo_method]
    pub fn del_blob() {
        RawStorage::delete(b"blob");
    }

    /// `RawStorage::get_into` exercising all three `RawStorageGet` variants.
    #[neo_method(safe)]
    pub fn get_blob_status() -> i64 {
        let mut buf = [0u8; 8];
        match RawStorage::get_into(b"blob", &mut buf) {
            RawStorageGet::Found(n) => n as i64,
            RawStorageGet::Missing => -1,
            RawStorageGet::BufferTooSmall(k) => -(k as i64),
        }
    }

    /// `put_i64` / `get_i64` round trip.
    #[neo_method]
    pub fn round_i64(v: i64) -> i64 {
        RawStorage::put_i64(b"i64", v);
        RawStorage::get_i64(b"i64").unwrap_or(i64::MIN)
    }

    /// `put_u16` / `get_u16` round trip.
    #[neo_method]
    pub fn round_u16(v: u32) -> u32 {
        RawStorage::put_u16(b"u16", v as u16);
        RawStorage::get_u16(b"u16").map(|x| x as u32).unwrap_or(0)
    }

    /// `put_bool` / `get_bool` round trip.
    #[neo_method]
    pub fn round_bool(b: bool) -> bool {
        RawStorage::put_bool(b"bool", b);
        RawStorage::get_bool(b"bool").unwrap_or(false)
    }

    /// `put_i64_key` (integer-keyed put).
    #[neo_method]
    pub fn set_keyed(k: i64, v: i64) {
        RawStorage::put_i64_key(k, v);
    }

    /// `get_i64_key_or_zero`.
    #[neo_method(safe)]
    pub fn get_keyed(k: i64) -> i64 {
        RawStorage::get_i64_key_or_zero(k)
    }

    /// `has_i64_key`.
    #[neo_method(safe)]
    pub fn has_keyed(k: i64) -> bool {
        RawStorage::has_i64_key(k)
    }

    /// `delete_i64_key`.
    #[neo_method]
    pub fn del_keyed(k: i64) {
        RawStorage::delete_i64_key(k);
    }

    /// `RawKeyBuilder`: build a composite key on the stack, store under it,
    /// then `clear` and reuse — covers new/push_byte/push_bytes/push_i64_le/
    /// as_slice/len/is_empty/clear.
    #[neo_method]
    pub fn builder_key(prefix: i64, idx: i64) -> i64 {
        let mut kb = RawKeyBuilder::<24>::new();
        let _empty = kb.is_empty();
        kb.push_byte(prefix as u8);
        kb.push_bytes(b"bal:");
        kb.push_i64_le(idx);
        let len = kb.len() as i64;
        RawStorage::put_i64(kb.as_slice(), idx);
        let read = RawStorage::get_i64(kb.as_slice()).unwrap_or(-1);
        kb.clear();
        len.wrapping_mul(1000).wrapping_add(read)
    }
}

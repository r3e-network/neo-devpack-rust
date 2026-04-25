// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "StorageSmoke"
}"#
);

#[neo_contract]
pub struct StorageSmokeContract;

// Force the optimizer to keep this as a direct call site by reading the
// inputs through volatile pointer reads to a static so it can't constant-fold.
// We pick from two distinct values via a runtime parameter.
static INPUT_A: i64 = 100;
static INPUT_B: i64 = 200;

#[inline(never)]
#[no_mangle]
extern "C" fn helper_arg0(a: i64, b: i64) -> i64 {
    let _ = b;
    a * 1000 + 1
}

#[inline(never)]
#[no_mangle]
extern "C" fn helper_arg1(a: i64, b: i64) -> i64 {
    let _ = a;
    b * 1000 + 2
}

#[neo_contract]
impl StorageSmokeContract {
    pub fn new() -> Self {
        Self
    }

    /// Direct call to helper_arg0(100, 200). Expected: 100001 if args are
    /// passed correctly, 200001 if reversed.
    #[neo_method(safe, name = "directFirst")]
    pub fn direct_first() -> i64 {
        let a = unsafe { core::ptr::read_volatile(&INPUT_A) };
        let b = unsafe { core::ptr::read_volatile(&INPUT_B) };
        helper_arg0(a, b)
    }

    /// Direct call to helper_arg1(100, 200). Expected: 200002 correct,
    /// 100002 if reversed.
    #[neo_method(safe, name = "directSecond")]
    pub fn direct_second() -> i64 {
        let a = unsafe { core::ptr::read_volatile(&INPUT_A) };
        let b = unsafe { core::ptr::read_volatile(&INPUT_B) };
        helper_arg1(a, b)
    }

    #[neo_method(name = "setValue")]
    pub fn set_value(value: i64) {
        RawStorage::put_i64(b"v", value);
    }

    #[neo_method(safe, name = "getValue")]
    pub fn get_value() -> i64 {
        RawStorage::get_i64(b"v").unwrap_or(-1)
    }
}

impl Default for StorageSmokeContract {
    fn default() -> Self {
        Self::new()
    }
}

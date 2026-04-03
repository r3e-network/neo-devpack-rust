// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Fuzz target: keep syscall alias/hash resolution aligned between wasm-neovm and neo-devpack.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use neo_syscalls::{NeoVMSyscallLowering, NeoVMSyscallRegistry};
use wasm_neovm_fuzz::sanitize_symbol;

#[derive(Debug, Arbitrary)]
struct SyscallSurfaceInput<'a> {
    import_name: &'a str,
    descriptor: &'a str,
    hash: u32,
}

fuzz_target!(|input: SyscallSurfaceInput<'_>| {
    let registry = NeoVMSyscallRegistry::get_instance();
    let lowering = NeoVMSyscallLowering::new();

    if let Some(descriptor) = wasm_neovm::neo_syscalls::lookup_neo_syscall(input.import_name) {
        let translator_info = wasm_neovm::syscalls::lookup_extended(descriptor)
            .expect("resolved Neo alias must map to a known translator syscall");
        if let Some(devpack_info) = registry.get_syscall(descriptor) {
            assert_eq!(translator_info.hash, devpack_info.hash);
            assert_eq!(
                lowering.lower_syscall(descriptor).ok(),
                Some(translator_info.hash)
            );
        }
    }

    let descriptor = sanitize_symbol(input.descriptor, "System.Runtime.GetTime");
    if let Some(translator_info) = wasm_neovm::syscalls::lookup_extended(&descriptor) {
        if let Some(devpack_info) = registry.get_syscall(translator_info.name) {
            assert_eq!(translator_info.hash, devpack_info.hash);
        }
    }

    if let Some(translator_info) = wasm_neovm::syscalls::lookup_by_hash(input.hash) {
        if let Some(devpack_info) = registry.get_syscall_by_hash(input.hash) {
            assert_eq!(translator_info.name, devpack_info.name);
        }
    }
});

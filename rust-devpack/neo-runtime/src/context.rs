// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

use neo_syscalls::NeoVMSyscall;
use neo_types::*;

/// Lightweight view of the runtime context.
#[derive(Default)]
pub struct NeoRuntimeContext;

impl NeoRuntimeContext {
    pub fn new() -> Self {
        Self
    }

    pub fn trigger(&self) -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_trigger()
    }

    pub fn gas_left(&self) -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_gas_left()
    }

    pub fn invocation_counter(&self) -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_invocation_counter()
    }

    pub fn calling_script_hash(&self) -> NeoResult<NeoByteString> {
        NeoVMSyscall::get_calling_script_hash()
    }

    pub fn entry_script_hash(&self) -> NeoResult<NeoByteString> {
        NeoVMSyscall::get_entry_script_hash()
    }

    pub fn executing_script_hash(&self) -> NeoResult<NeoByteString> {
        NeoVMSyscall::get_executing_script_hash()
    }
}

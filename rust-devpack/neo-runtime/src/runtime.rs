// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

use neo_syscalls::NeoVMSyscall;
use neo_types::*;

use crate::NeoStorage;

/// Direct wrappers for the canonical System.Runtime syscalls.
pub struct NeoRuntime;

impl NeoRuntime {
    pub fn get_time() -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_time()
    }

    pub fn check_witness(account: &NeoByteString) -> NeoResult<NeoBoolean> {
        NeoVMSyscall::check_witness(account)
    }

    pub fn notify(event: &NeoString, state: &NeoArray<NeoValue>) -> NeoResult<()> {
        NeoVMSyscall::notify(event, state)
    }

    pub fn log(message: &NeoString) -> NeoResult<()> {
        NeoVMSyscall::log(message)
    }

    pub fn platform() -> NeoResult<NeoString> {
        NeoVMSyscall::platform()
    }

    pub fn get_trigger() -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_trigger()
    }

    pub fn get_invocation_counter() -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_invocation_counter()
    }

    pub fn get_random() -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_random()
    }

    pub fn get_network() -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_network()
    }

    pub fn get_address_version() -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_address_version()
    }

    pub fn get_gas_left() -> NeoResult<NeoInteger> {
        NeoVMSyscall::get_gas_left()
    }

    pub fn get_calling_script_hash() -> NeoResult<NeoByteString> {
        NeoVMSyscall::get_calling_script_hash()
    }

    pub fn get_entry_script_hash() -> NeoResult<NeoByteString> {
        NeoVMSyscall::get_entry_script_hash()
    }

    pub fn get_executing_script_hash() -> NeoResult<NeoByteString> {
        NeoVMSyscall::get_executing_script_hash()
    }

    pub fn get_notifications(script_hash: Option<&NeoByteString>) -> NeoResult<NeoArray<NeoValue>> {
        NeoVMSyscall::get_notifications(script_hash)
    }

    pub fn get_script_container() -> NeoResult<NeoArray<NeoValue>> {
        NeoVMSyscall::get_script_container()
    }

    pub fn get_storage_context() -> NeoResult<NeoStorageContext> {
        NeoStorage::get_context()
    }

    /// Burn the specified amount of GAS from the calling contract.
    pub fn burn_gas(gas: &NeoInteger) -> NeoResult<()> {
        NeoVMSyscall::burn_gas(gas)
    }

    /// Get the signers of the current transaction.
    pub fn current_signers() -> NeoResult<NeoArray<NeoValue>> {
        NeoVMSyscall::current_signers()
    }

    /// Load and execute a script dynamically.
    pub fn load_script(
        script: &NeoByteString,
        call_flags: &NeoInteger,
        args: &NeoArray<NeoValue>,
    ) -> NeoResult<()> {
        NeoVMSyscall::load_script(script, call_flags, args)
    }
}

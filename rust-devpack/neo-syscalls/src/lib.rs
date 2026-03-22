// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Neo N3 System Calls
//!
//! This crate provides bindings to Neo N3 system calls for smart contract development.

use neo_types::*;
use std::slice::Iter;

mod storage;
mod syscalls;
mod wrapper;

pub use syscalls::SYSCALLS;
pub use wrapper::{neovm_syscall, NeoVMSyscall};

/// Neo N3 System Call Registry
pub struct NeoVMSyscallRegistry {
    syscalls: &'static [NeoVMSyscallInfo],
}

impl NeoVMSyscallRegistry {
    pub const fn new(syscalls: &'static [NeoVMSyscallInfo]) -> Self {
        Self { syscalls }
    }

    pub fn get_syscall(&self, name: &str) -> Option<&NeoVMSyscallInfo> {
        self.syscalls.iter().find(|s| s.name == name)
    }

    pub fn get_syscall_by_hash(&self, hash: u32) -> Option<&NeoVMSyscallInfo> {
        self.syscalls.iter().find(|s| s.hash == hash)
    }

    pub fn has_syscall(&self, name: &str) -> bool {
        self.get_syscall(name).is_some()
    }

    pub fn get_instance() -> Self {
        Self::new(SYSCALLS)
    }

    pub fn iter(&self) -> Iter<'static, NeoVMSyscallInfo> {
        self.syscalls.iter()
    }

    pub fn names(&self) -> impl Iterator<Item = &'static str> {
        self.syscalls.iter().map(|info| info.name)
    }
}

/// Neo N3 System Call Information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeoVMSyscallInfo {
    pub name: &'static str,
    pub hash: u32,
    pub parameters: &'static [&'static str],
    pub return_type: &'static str,
    pub gas_cost: u32,
    pub description: &'static str,
}

/// Neo N3 System Call Lowering
pub struct NeoVMSyscallLowering;

impl Default for NeoVMSyscallLowering {
    fn default() -> Self {
        Self::new()
    }
}

impl NeoVMSyscallLowering {
    pub fn new() -> Self {
        Self
    }

    pub fn lower_syscall(&self, name: &str) -> NeoResult<u32> {
        let registry = NeoVMSyscallRegistry::get_instance();
        if let Some(syscall) = registry.get_syscall(name) {
            Ok(syscall.hash)
        } else {
            Err(NeoError::new(&format!("Unknown syscall: {}", name)))
        }
    }

    pub fn can_lower(&self, name: &str) -> bool {
        let registry = NeoVMSyscallRegistry::get_instance();
        registry.has_syscall(name)
    }
}

/// Neo N3 System Call Registry Instance
pub static SYSCALL_REGISTRY: NeoVMSyscallRegistry = NeoVMSyscallRegistry::new(SYSCALLS);

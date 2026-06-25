// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Pluggable host state for the executor: storage, witnesses, notifications,
//! logs, and a syscall dispatch table for the subset tests need.

use crate::exec::item::NeoItem;
use crate::syscalls;
use num_bigint::BigInt;
use std::collections::{BTreeMap, HashSet};

/// A fault raised by the engine.
#[derive(Debug, Clone)]
pub enum Fault {
    /// An ABORT/ASSERT opcode or a bounds violation.
    Abort(String),
    /// An unhandled / unimplemented opcode.
    Unsupported(String),
    /// A SYSCALL the host refused or doesn't model.
    Syscall(String),
    /// Stack underflow / type mismatch.
    State(String),
}

/// Host state for [`crate::exec::engine::Engine`]. Reset via [`Host::reset`].
#[derive(Debug, Default)]
pub struct Host {
    pub storage: BTreeMap<Vec<u8>, Vec<u8>>,
    pub witnesses: HashSet<[u8; 20]>,
    pub notifications: Vec<(String, Vec<NeoItem>)>,
    pub logs: Vec<String>,
    /// Script hashes: [calling, entry, executing], little-endian.
    pub script_hashes: [[u8; 20]; 3],
    /// Mocked time returned by System.Runtime.GetTime.
    pub time: i64,
    pub static_fields: Vec<NeoItem>,
}

impl Host {
    /// Clear all host state.
    pub fn reset(&mut self) {
        self.storage.clear();
        self.witnesses.clear();
        self.notifications.clear();
        self.logs.clear();
        self.script_hashes = [[0u8; 20]; 3];
        self.time = 0;
        self.static_fields.clear();
    }

    /// Set the executing script hash (used by GetExecutingScriptHash).
    pub fn set_executing_hash(&mut self, hash: [u8; 20]) {
        self.script_hashes[2] = hash;
    }

    /// Dispatch a SYSCALL by its 4-byte hash. Returns the optional return item
    /// (None for void syscalls). The host owns its own static fields, so they
    /// are not passed separately.
    pub fn syscall(&mut self, hash: u32, stack: &mut Vec<NeoItem>) -> Result<Option<NeoItem>, Fault> {
        let name = syscalls::lookup_by_hash(hash).map(|i| i.name).unwrap_or("");
        match name {
            "System.Storage.GetContext" => Ok(Some(NeoItem::int(0))),
            "System.Storage.Get" => {
                let _ctx = stack.pop().ok_or(Fault::State("Get: missing ctx".into()))?;
                let key = stack
                    .pop()
                    .and_then(|i| match i {
                        NeoItem::Buffer(b) => Some(b.borrow().clone()),
                        _ => None,
                    })
                    .ok_or(Fault::State("Get: missing key buffer".into()))?;
                let val = self.storage.get(&key).cloned().unwrap_or_default();
                Ok(Some(NeoItem::buffer(val)))
            }
            "System.Storage.Put" | "System.Storage.PutEx" => {
                let _ctx = stack.pop().ok_or(Fault::State("Put: missing ctx".into()))?;
                let key = stack
                    .pop()
                    .and_then(|i| match i {
                        NeoItem::Buffer(b) => Some(b.borrow().clone()),
                        _ => None,
                    })
                    .ok_or(Fault::State("Put: missing key buffer".into()))?;
                let val = stack
                    .pop()
                    .and_then(|i| match i {
                        NeoItem::Buffer(b) => Some(b.borrow().clone()),
                        _ => None,
                    })
                    .ok_or(Fault::State("Put: missing value buffer".into()))?;
                self.storage.insert(key, val);
                Ok(None)
            }
            "System.Storage.Delete" => {
                let _ctx = stack.pop().ok_or(Fault::State("Delete: missing ctx".into()))?;
                let key = stack
                    .pop()
                    .and_then(|i| match i {
                        NeoItem::Buffer(b) => Some(b.borrow().clone()),
                        _ => None,
                    })
                    .ok_or(Fault::State("Delete: missing key buffer".into()))?;
                self.storage.remove(&key);
                Ok(None)
            }
            "System.Runtime.CheckWitness" => {
                let acct = stack
                    .pop()
                    .and_then(|i| match i {
                        NeoItem::Buffer(b) => Some(b.borrow().clone()),
                        _ => None,
                    })
                    .ok_or(Fault::State("CheckWitness: missing account".into()))?;
                let mut key = [0u8; 20];
                if acct.len() == 20 {
                    key.copy_from_slice(&acct);
                }
                Ok(Some(NeoItem::Boolean(self.witnesses.contains(&key))))
            }
            "System.Runtime.GetTime" => Ok(Some(NeoItem::int(self.time))),
            "System.Runtime.Log" => {
                let msg = stack.pop().ok_or(Fault::State("Log: missing msg".into()))?;
                self.logs.push(format!("{msg:?}"));
                Ok(None)
            }
            "System.Runtime.Notify" => {
                // name (ByteString), state (Array)
                let state = stack.pop().ok_or(Fault::State("Notify: missing state".into()))?;
                let name_item = stack.pop().ok_or(Fault::State("Notify: missing name".into()))?;
                let name = match name_item {
                    NeoItem::Buffer(b) => String::from_utf8_lossy(&b.borrow()).into_owned(),
                    other => format!("{other:?}"),
                };
                let state_items = match state {
                    NeoItem::Array(a) | NeoItem::Struct(a) => a,
                    _ => Vec::new(),
                };
                self.notifications.push((name, state_items));
                Ok(None)
            }
            "System.Runtime.GetCallingScriptHash" => {
                Ok(Some(NeoItem::buffer(self.script_hashes[0].to_vec())))
            }
            "System.Runtime.GetEntryScriptHash" => {
                Ok(Some(NeoItem::buffer(self.script_hashes[1].to_vec())))
            }
            "System.Runtime.GetExecutingScriptHash" => {
                Ok(Some(NeoItem::buffer(self.script_hashes[2].to_vec())))
            }
            "System.Contract.Call" => {
                // contractHash, method, callFlags, args[]
                let _args = stack.pop();
                let _flags = stack.pop();
                let _method = stack.pop();
                let _hash = stack.pop();
                // Default mock: returns Null; tests can override via a custom
                // Host if they need call semantics.
                Ok(Some(NeoItem::Null))
            }
            _ => {
                Err(Fault::Syscall(format!("unmodelled syscall {name} (0x{hash:08x})")))
            }
        }
    }

    /// Helper: read a BigInt argument from a Notify state by index.
    pub fn notify_int(&self, idx: usize, arg: usize) -> Option<&BigInt> {
        self.notifications
            .get(idx)
            .and_then(|(_, args)| args.get(arg))
            .and_then(NeoItem::as_int)
    }
}

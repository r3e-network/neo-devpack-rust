// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Neo N3 Runtime facade
//!
//! This crate provides a lightweight façade over the Neo runtime surface so
//! integration tests and examples can exercise the canonical syscall
//! catalogue without depending on a full node implementation. The
//! implementation intentionally returns deterministic placeholder values –
//! enough to validate wiring and type conversions while remaining
//! self-contained for unit tests.
//!
//! # ⚠️ Contract panic safety
//!
//! In Neo N3, a runtime panic (`panic!`, `unwrap()` on `None`/`Err`, index out
//! of bounds, arithmetic overflow) causes the VM to **FAULT**, which reverts
//! the entire transaction. Unlike off-chain Rust, you **cannot** use panics
//! for control flow in on-chain contracts. Always use `NeoResult<T>` and
//! explicit error handling. The `#[neo_contract]` macro expects methods to
//! return `NeoResult<T>` precisely because panics are unrecoverable on-chain.

mod context;
mod contract;
#[cfg(feature = "crypto-hashes")]
mod crypto;
#[cfg(feature = "json")]
mod json;
mod runtime;
mod storage;

pub use context::NeoRuntimeContext;
pub use contract::NeoContractRuntime;
#[cfg(feature = "crypto-hashes")]
pub use crypto::NeoCrypto;
#[cfg(feature = "json")]
pub use json::NeoJSON;
pub use runtime::NeoRuntime;
pub use storage::{NeoStorage, RawKeyBuilder, RawStorage, RawStorageGet};

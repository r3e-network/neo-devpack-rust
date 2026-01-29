// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Neo N3 Runtime facade
//!
//! This crate provides a lightweight façade over the Neo runtime surface so
//! integration tests and examples can exercise the canonical syscall
//! catalogue without depending on a full node implementation. The
//! implementation intentionally returns deterministic placeholder values –
//! enough to validate wiring and type conversions while remaining
//! self-contained for unit tests.

mod context;
mod contract;
mod crypto;
mod json;
mod runtime;
mod storage;

pub use context::NeoRuntimeContext;
pub use contract::NeoContractRuntime;
pub use crypto::NeoCrypto;
pub use json::NeoJSON;
pub use runtime::NeoRuntime;
pub use storage::NeoStorage;

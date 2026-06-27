// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! A minimal, deterministic in-process NeoVM execution harness.
//!
//! Implements only the opcode subset emitted by this translator, so generated
//! bytecode can be *executed* in ms-fast unit tests rather than only checked
//! for opcode-shape. The harness is NOT a full NeoVM: it omits opcodes/features
//! the translator never emits, and its syscall dispatch is a pluggable mock
//! ([`Host`]). It is the trust anchor for the Phase A correctness fixes
//! (memory store/load, arithmetic, control flow) and a substrate for later
//! behavioural tests.
//!
//! Enable with the `exec` cargo feature.

#![cfg(feature = "exec")]
// The exec harness is a test/bench substrate, not a shipping API; relax the
// crate-wide missing-docs lint within this module.
#![allow(missing_docs)]

pub mod engine;
pub mod host;
pub mod item;

pub use engine::{Engine, ExecResult};
pub use host::{Fault, Host};
pub use item::NeoItem;

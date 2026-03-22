// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Core abstractions and shared traits
//!
//! This module provides the fundamental abstractions used throughout
//! the wasm-neovm translator.

pub mod bytecode;
pub mod encoding;
pub mod traits;

pub use bytecode::*;
pub use encoding::*;
pub use traits::*;

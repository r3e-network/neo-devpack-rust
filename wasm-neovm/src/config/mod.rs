// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Centralized configuration management
//!
//! This module provides unified configuration handling for the wasm-neovm
//! translator, consolidating all configuration options in one place.

pub mod options;
pub mod validation;

pub use options::*;
pub use validation::*;

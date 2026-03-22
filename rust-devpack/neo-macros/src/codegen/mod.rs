// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Code generation utilities for Neo N3 procedural macros.
//!
//! This module provides shared utilities for generating code in Neo N3 macros,
//! including custom section generation and type mapping.

pub mod custom_section;

pub use custom_section::{manifest_overlay_tokens, manifest_type_from_syn};

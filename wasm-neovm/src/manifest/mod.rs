// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Contract manifest generation for Neo N3
//!
//! This module handles the creation and manipulation of Neo N3 contract manifests,
//! which describe the contract's ABI, permissions, and other metadata.
//!
//! # Key Types
//!
//! - [`ManifestBuilder`]: Builder pattern for constructing manifests
//! - [`RenderedManifest`]: Final rendered manifest ready for serialization
//! - [`ManifestMethod`]: Description of a contract method
//! - [`ManifestParameter`]: Description of a method parameter

mod build;
mod builder;
mod merge;
mod parity;
mod types;

#[cfg(test)]
mod tests;

pub use build::build_manifest;
pub use builder::ManifestBuilder;
pub use merge::{merge_manifest, propagate_safe_flags};
pub use parity::{collect_method_names, ensure_manifest_methods_match};
pub use types::{ManifestMethod, ManifestParameter, RenderedManifest};

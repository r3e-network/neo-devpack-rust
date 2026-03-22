// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Neo N3 Contract Testing Framework
//!
//! This crate provides utilities for testing Neo N3 smart contracts
//! with a mock runtime environment.
//!
//! # Usage
//!
//! ```ignore
//! use neo_test::*;
//!
//! // Create a test environment
//! let mut env = TestEnvironment::new();
//!
//! // Set up contract state
//! env.set_storage(b"owner", b"AV4GGdKS2C7j1GqC3w5y4qX5");
//!
//! // Add witness
//! env.add_witness(b"AV4GGdKS2C7j1GqC3w5y4qX5");
//!
//! // Assert
//! env.assert_storage().assert_contains(b"owner");
//! env.assert_runtime().assert_witness(b"AV4GGdKS2C7j1GqC3w5y4qX5");
//! ```

mod assertions;
mod environment;
mod mock_runtime;

#[cfg(test)]
mod tests;

pub use assertions::*;
pub use environment::*;
pub use mock_runtime::*;

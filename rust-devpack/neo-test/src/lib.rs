// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Neo N3 Contract Testing Framework
//!
//! This crate provides utilities for testing Neo N3 smart contracts
//! with a mock runtime environment.
//!
//! # Usage
//!
//! ```rust
//! use neo_test::*;
//!
//! #[test]
//! fn test_my_contract() {
//!     let mut env = TestEnvironment::new();
//!     
//!     // Set up contract state
//!     env.set_storage(b"owner", b"AV4GGdKS2C7j1GqC3w5y4qX5qZ5qZ5qZ5");
//!     
//!     // Call contract method
//!     let result = env.call_method("balanceOf", &[address.to_arg()]);
//!     
//!     // Assert results
//!     result.assert_ok();
//!     result.assert_returns(100);
//! }
//! ```

mod assertions;
mod environment;
mod mock_runtime;

pub use assertions::*;
pub use environment::*;
pub use mock_runtime::*;

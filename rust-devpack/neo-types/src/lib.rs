// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Neo N3 Core Types
//!
//! This crate provides the core types and data structures for Neo N3 smart contract development.

mod array;
mod boolean;
mod bytestring;
mod error;
mod hash;
mod integer;
mod iterator;
mod manifest;
mod map;
mod stack_item;
mod storage;
mod string;
mod traits;
mod value;

pub use array::NeoArray;
pub use boolean::NeoBoolean;
pub use bytestring::NeoByteString;
pub use error::{NeoError, NeoResult};
pub use hash::{Hash160, Hash256};
pub use integer::NeoInteger;
pub use iterator::NeoIterator;
pub use manifest::{
    NeoContractABI, NeoContractEvent, NeoContractManifest, NeoContractMethod, NeoContractParameter,
    NeoContractPermission,
};
pub use map::NeoMap;
pub use stack_item::{
    serialise_array, serialise_notification, serialise_value, MAX_NOTIFICATION_SIZE, MAX_STACK_SIZE,
};
pub use storage::NeoStorageContext;
pub use string::NeoString;
pub use traits::{NeoContract, NeoContractEntry, NeoContractMethodTrait};
pub use value::{NeoStruct, NeoValue};

// Re-export num-bigint so users don't need a direct dependency
// to interop with `NeoInteger` via `BigInt`. The version is pinned
// to the same major as `num-bigint = "0.4"` in this crate's Cargo.toml.
pub use num_bigint::BigInt;

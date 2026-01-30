// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

use std::string::String;
use std::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Neo N3 Contract Manifest
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NeoContractManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub email: String,
    pub description: String,
    pub abi: NeoContractABI,
    pub permissions: Vec<NeoContractPermission>,
    pub trusts: Vec<String>,
    pub supported_standards: Vec<String>,
}

/// Neo N3 Contract ABI
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NeoContractABI {
    pub hash: String,
    pub methods: Vec<NeoContractMethod>,
    pub events: Vec<NeoContractEvent>,
}

/// Neo N3 Contract Method
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NeoContractMethod {
    pub name: String,
    pub parameters: Vec<NeoContractParameter>,
    pub return_type: String,
    pub offset: u32,
    pub safe: bool,
}

/// Neo N3 Contract Event
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NeoContractEvent {
    pub name: String,
    pub parameters: Vec<NeoContractParameter>,
}

/// Neo N3 Contract Parameter
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NeoContractParameter {
    pub name: String,
    pub r#type: String,
}

/// Neo N3 Contract Permission
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NeoContractPermission {
    pub contract: String,
    pub methods: Vec<String>,
}

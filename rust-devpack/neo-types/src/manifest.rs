// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

use std::string::String;
use std::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ManifestExtraMetadata {
    #[serde(default)]
    version: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    description: String,
}

#[cfg(feature = "serde")]
fn default_version() -> String {
    "1.0.0".to_string()
}

#[cfg(feature = "serde")]
fn default_author() -> String {
    "neo-devpack".to_string()
}

/// Neo N3 Contract Manifest
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct NeoContractManifest {
    pub name: String,
    #[cfg_attr(feature = "serde", serde(default = "default_version"))]
    pub version: String,
    #[cfg_attr(feature = "serde", serde(default = "default_author"))]
    pub author: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub email: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub description: String,
    pub abi: NeoContractABI,
    pub permissions: Vec<NeoContractPermission>,
    pub trusts: Vec<String>,
    #[cfg_attr(
        feature = "serde",
        serde(alias = "supportedstandards", default)
    )]
    pub supported_standards: Vec<String>,
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for NeoContractManifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawManifest {
            name: String,
            #[serde(default = "default_version")]
            version: String,
            #[serde(default = "default_author")]
            author: String,
            #[serde(default)]
            email: String,
            #[serde(default)]
            description: String,
            abi: NeoContractABI,
            permissions: Vec<NeoContractPermission>,
            trusts: Vec<String>,
            #[serde(alias = "supportedstandards", default)]
            supported_standards: Vec<String>,
            #[serde(default)]
            extra: Option<ManifestExtraMetadata>,
        }

        let raw = RawManifest::deserialize(deserializer)?;
        let mut version = raw.version;
        let mut author = raw.author;
        let mut email = raw.email;
        let mut description = raw.description;

        if let Some(extra) = raw.extra {
            if version == default_version() && !extra.version.is_empty() {
                version = extra.version;
            }
            if author == default_author() && !extra.author.is_empty() {
                author = extra.author;
            }
            if email.is_empty() && !extra.email.is_empty() {
                email = extra.email;
            }
            if description.is_empty() && !extra.description.is_empty() {
                description = extra.description;
            }
        }

        Ok(Self {
            name: raw.name,
            version,
            author,
            email,
            description,
            abi: raw.abi,
            permissions: raw.permissions,
            trusts: raw.trusts,
            supported_standards: raw.supported_standards,
        })
    }
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

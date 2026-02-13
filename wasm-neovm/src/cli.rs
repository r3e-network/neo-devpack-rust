use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Parser;

use wasm_neovm::SourceChain;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Translate a Wasm module into NeoVM NEF artefacts"
)]
pub(crate) struct Cli {
    /// Path to the WebAssembly module compiled from Rust
    #[arg(short, long)]
    pub(crate) input: PathBuf,

    /// Output NEF path (default: <input_basename>.nef)
    #[arg(long)]
    pub(crate) nef: Option<PathBuf>,

    /// Output manifest path (default: <input_basename>.manifest.json)
    #[arg(long)]
    pub(crate) manifest: Option<PathBuf>,

    /// Contract name stored in the manifest
    #[arg(long, default_value = "Contract")]
    pub(crate) name: String,

    /// Path to a JSON file providing manifest overlay data
    #[arg(long = "manifest-overlay")]
    pub(crate) manifest_overlay: Option<PathBuf>,

    /// Source URL recorded in the NEF header
    #[arg(long = "source-url")]
    pub(crate) source_url: Option<String>,

    /// Path to an existing manifest to compare against (translation fails when they differ)
    #[arg(long = "compare-manifest")]
    pub(crate) compare_manifest: Option<PathBuf>,

    /// Source blockchain for cross-chain compilation (neo, solana, move)
    #[arg(long = "source-chain", default_value = "neo")]
    pub(crate) source_chain: String,

    /// Enable verbose logging
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,
}

impl Cli {
    pub(crate) fn parse_source_chain(&self) -> Result<SourceChain> {
        SourceChain::from_str(&self.source_chain).ok_or_else(|| {
            anyhow!(
                "unknown source chain '{}' (expected one of: neo, native, solana, sol, move, aptos, sui)",
                self.source_chain
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_source_chain_accepts_aliases() {
        let cli = Cli {
            input: PathBuf::from("in.wasm"),
            nef: None,
            manifest: None,
            name: "Contract".to_string(),
            manifest_overlay: None,
            source_url: None,
            compare_manifest: None,
            source_chain: "sol".to_string(),
            verbose: 0,
        };
        assert_eq!(cli.parse_source_chain().unwrap(), SourceChain::Solana);

        let cli = Cli {
            source_chain: "aptos".to_string(),
            ..cli
        };
        assert_eq!(cli.parse_source_chain().unwrap(), SourceChain::Move);
    }

    #[test]
    fn parse_source_chain_rejects_unknown_value() {
        let cli = Cli {
            input: PathBuf::from("in.wasm"),
            nef: None,
            manifest: None,
            name: "Contract".to_string(),
            manifest_overlay: None,
            source_url: None,
            compare_manifest: None,
            source_chain: "ethereum".to_string(),
            verbose: 0,
        };

        assert!(cli.parse_source_chain().is_err());
    }
}

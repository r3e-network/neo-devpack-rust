use std::path::PathBuf;

use clap::Parser;
use log::warn;

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
}

impl Cli {
    pub(crate) fn parse_source_chain(&self) -> SourceChain {
        SourceChain::from_str(&self.source_chain).unwrap_or_else(|| {
            warn!(
                "Unknown source chain '{}', defaulting to 'neo'",
                self.source_chain
            );
            SourceChain::Neo
        })
    }
}

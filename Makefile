# Top-level automation for building and translating Rust → NeoVM examples

MAKEFLAGS += --no-builtin-rules
.DEFAULT_GOAL := help

WASM_TARGET       := wasm32-unknown-unknown
OUTDIR            := build
TRANSLATOR        := cargo run --manifest-path wasm-neovm/Cargo.toml --quiet --

HELLO_WASM        := contracts/hello-world/target/$(WASM_TARGET)/release/hello_world.wasm
HELLO_NEF         := $(OUTDIR)/HelloWorld.nef
HELLO_MANIFEST    := $(OUTDIR)/HelloWorld.manifest.json
NEP17_WASM        := contracts/nep17-token/target/$(WASM_TARGET)/release/nep17_token.wasm
NEP17_NEF         := $(OUTDIR)/NEP17.nef
NEP17_MANIFEST    := $(OUTDIR)/NEP17.manifest.json
AMM_WASM          := contracts/constant-product/target/$(WASM_TARGET)/release/constant_product.wasm
AMM_NEF           := $(OUTDIR)/AMM.nef
AMM_MANIFEST      := $(OUTDIR)/AMM.manifest.json
NEP11_WASM        := contracts/nep11-nft/target/$(WASM_TARGET)/release/nep11_nft.wasm
NEP11_NEF         := $(OUTDIR)/NEP11.nef
NEP11_MANIFEST    := $(OUTDIR)/NEP11.manifest.json
MULTISIG_WASM     := contracts/multisig-wallet/target/$(WASM_TARGET)/release/multisig_wallet.wasm
MULTISIG_NEF      := $(OUTDIR)/MultisigWallet.nef
MULTISIG_MANIFEST := $(OUTDIR)/MultisigWallet.manifest.json
ESCROW_WASM       := contracts/escrow/target/$(WASM_TARGET)/release/escrow.wasm
ESCROW_NEF        := $(OUTDIR)/Escrow.nef
ESCROW_MANIFEST   := $(OUTDIR)/Escrow.manifest.json
CROWD_WASM        := contracts/crowdfunding/target/$(WASM_TARGET)/release/crowdfunding.wasm
CROWD_NEF         := $(OUTDIR)/Crowdfunding.nef
CROWD_MANIFEST    := $(OUTDIR)/Crowdfunding.manifest.json
GOV_WASM          := contracts/governance-dao/target/$(WASM_TARGET)/release/governance_dao.wasm
GOV_NEF           := $(OUTDIR)/GovernanceDAO.nef
GOV_MANIFEST      := $(OUTDIR)/GovernanceDAO.manifest.json
ORACLE_WASM       := contracts/oracle-consumer/target/$(WASM_TARGET)/release/oracle_consumer.wasm
ORACLE_NEF        := $(OUTDIR)/OracleConsumer.nef
ORACLE_MANIFEST   := $(OUTDIR)/OracleConsumer.manifest.json
MARKET_WASM       := contracts/nft-marketplace/target/$(WASM_TARGET)/release/nft_marketplace.wasm
MARKET_NEF        := $(OUTDIR)/NFTMarketplace.nef
MARKET_MANIFEST   := $(OUTDIR)/NFTMarketplace.manifest.json

.PHONY: help examples hello-world nep17-token constant-product nep11-nft multisig-wallet escrow crowdfunding governance-dao oracle-consumer nft-marketplace c-hello fmt lint test integration-tests spec clean

help:
	@echo "Usage: make <target>"
	@echo
	@echo "Primary targets:"
	@echo "  examples        Build and translate all sample contracts"
	@echo "  hello-world     Generate HelloWorld.nef and manifest"
	@echo "  nep17-token     Generate NEP17.nef and manifest"
	@echo "  constant-product Generate AMM.nef and manifest"
	@echo "  nep11-nft       Generate NEP11.nef and manifest"
	@echo "  multisig-wallet Generate MultisigWallet.nef and manifest"
	@echo "  escrow          Generate Escrow.nef and manifest"
	@echo "  crowdfunding    Generate Crowdfunding.nef and manifest"
	@echo "  governance-dao  Generate GovernanceDAO.nef and manifest"
	@echo "  oracle-consumer Generate OracleConsumer.nef and manifest"
	@echo "  nft-marketplace Generate NFTMarketplace.nef and manifest"
	@echo "  c-hello        Build the sample C contract and translate it"
	@echo
	@echo "Maintenance targets:"
	@echo "  fmt             Run cargo fmt across the workspace"
	@echo "  lint            Run cargo clippy across the workspace"
	@echo "  test            Execute cargo test for translator + devpack"
	@echo "  integration-tests  Run optional Neo Express integration harness"
	@echo "  spec            Build the LaTeX specification in spec/"
	@echo "  clean           Remove generated build artefacts"

examples: hello-world nep17-token constant-product nep11-nft multisig-wallet escrow crowdfunding governance-dao oracle-consumer nft-marketplace c-hello

hello-world: $(HELLO_NEF) $(HELLO_MANIFEST)
	@echo "✔ hello-world artifacts are in $(OUTDIR)/"

nep17-token: $(NEP17_NEF) $(NEP17_MANIFEST)
	@echo "✔ NEP-17 artifacts are in $(OUTDIR)/"

constant-product: $(AMM_NEF) $(AMM_MANIFEST)
	@echo "✔ Constant-product AMM artifacts are in $(OUTDIR)/"

nep11-nft: $(NEP11_NEF) $(NEP11_MANIFEST)
	@echo "✔ NEP-11 artifacts are in $(OUTDIR)/"

multisig-wallet: $(MULTISIG_NEF) $(MULTISIG_MANIFEST)
	@echo "✔ Multisig wallet artifacts are in $(OUTDIR)/"

escrow: $(ESCROW_NEF) $(ESCROW_MANIFEST)
	@echo "✔ Escrow artifacts are in $(OUTDIR)/"

crowdfunding: $(CROWD_NEF) $(CROWD_MANIFEST)
	@echo "✔ Crowdfunding artifacts are in $(OUTDIR)/"

governance-dao: $(GOV_NEF) $(GOV_MANIFEST)
	@echo "✔ Governance DAO artifacts are in $(OUTDIR)/"

oracle-consumer: $(ORACLE_NEF) $(ORACLE_MANIFEST)
	@echo "✔ Oracle consumer artifacts are in $(OUTDIR)/"

nft-marketplace: $(MARKET_NEF) $(MARKET_MANIFEST)
	@echo "✔ NFT marketplace artifacts are in $(OUTDIR)/"

$(HELLO_NEF) $(HELLO_MANIFEST): $(HELLO_WASM) | $(OUTDIR)
	$(TRANSLATOR) \
	  --input $(HELLO_WASM) \
	  --nef $(HELLO_NEF) \
	  --manifest $(HELLO_MANIFEST) \
	  --name HelloWorld

$(NEP17_NEF) $(NEP17_MANIFEST): $(NEP17_WASM) | $(OUTDIR)
	$(TRANSLATOR) \
	  --input $(NEP17_WASM) \
	  --nef $(NEP17_NEF) \
	  --manifest $(NEP17_MANIFEST) \
	  --name SampleNEP17

$(AMM_NEF) $(AMM_MANIFEST): $(AMM_WASM) | $(OUTDIR)
	$(TRANSLATOR) \
	  --input $(AMM_WASM) \
	  --nef $(AMM_NEF) \
	  --manifest $(AMM_MANIFEST) \
	  --name ConstantProductAMM

$(NEP11_NEF) $(NEP11_MANIFEST): $(NEP11_WASM) | $(OUTDIR)
	$(TRANSLATOR) \
	  --input $(NEP11_WASM) \
	  --nef $(NEP11_NEF) \
	  --manifest $(NEP11_MANIFEST) \
	  --name SampleNEP11

$(MULTISIG_NEF) $(MULTISIG_MANIFEST): $(MULTISIG_WASM) | $(OUTDIR)
	$(TRANSLATOR) \
	  --input $(MULTISIG_WASM) \
	  --nef $(MULTISIG_NEF) \
	  --manifest $(MULTISIG_MANIFEST) \
	  --name SampleMultisig

$(ESCROW_NEF) $(ESCROW_MANIFEST): $(ESCROW_WASM) | $(OUTDIR)
	$(TRANSLATOR) \
	  --input $(ESCROW_WASM) \
	  --nef $(ESCROW_NEF) \
	  --manifest $(ESCROW_MANIFEST) \
	  --name NeoEscrow

$(CROWD_NEF) $(CROWD_MANIFEST): $(CROWD_WASM) | $(OUTDIR)
	$(TRANSLATOR) \
	  --input $(CROWD_WASM) \
	  --nef $(CROWD_NEF) \
	  --manifest $(CROWD_MANIFEST) \
	  --name NeoCrowdfund

$(GOV_NEF) $(GOV_MANIFEST): $(GOV_WASM) | $(OUTDIR)
	$(TRANSLATOR) \
	  --input $(GOV_WASM) \
	  --nef $(GOV_NEF) \
	  --manifest $(GOV_MANIFEST) \
	  --name NeoGovernanceDAO

$(ORACLE_NEF) $(ORACLE_MANIFEST): $(ORACLE_WASM) | $(OUTDIR)
	$(TRANSLATOR) \
	  --input $(ORACLE_WASM) \
	  --nef $(ORACLE_NEF) \
	  --manifest $(ORACLE_MANIFEST) \
	  --name NeoOracleConsumer

$(MARKET_NEF) $(MARKET_MANIFEST): $(MARKET_WASM) | $(OUTDIR)
	$(TRANSLATOR) \
	  --input $(MARKET_WASM) \
	  --nef $(MARKET_NEF) \
	  --manifest $(MARKET_MANIFEST) \
	  --name NeoNFTMarketplace

$(HELLO_WASM):
	cargo build --manifest-path contracts/hello-world/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(NEP17_WASM):
	cargo build --manifest-path contracts/nep17-token/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(AMM_WASM):
	cargo build --manifest-path contracts/constant-product/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(NEP11_WASM):
	cargo build --manifest-path contracts/nep11-nft/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(MULTISIG_WASM):
	cargo build --manifest-path contracts/multisig-wallet/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(ESCROW_WASM):
	cargo build --manifest-path contracts/escrow/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(CROWD_WASM):
	cargo build --manifest-path contracts/crowdfunding/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(GOV_WASM):
	cargo build --manifest-path contracts/governance-dao/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(ORACLE_WASM):
	cargo build --manifest-path contracts/oracle-consumer/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(MARKET_WASM):
	cargo build --manifest-path contracts/nft-marketplace/Cargo.toml --release --target $(WASM_TARGET) --quiet

c-hello:
	scripts/build_c_contract.sh contracts/c-hello

$(OUTDIR):
	@mkdir -p $(OUTDIR)

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets --all-features

test:
	cargo test --manifest-path wasm-neovm/Cargo.toml
	cargo test --manifest-path rust-devpack/Cargo.toml

integration-tests:
	@echo "Running integration tests (requires NEO_EXPRESS_RPC)..."
	cargo test --manifest-path integration-tests/Cargo.toml -- --ignored

spec:
	$(MAKE) -C spec

clean:
	rm -rf $(OUTDIR)
	rm -rf contracts/hello-world/target contracts/nep17-token/target contracts/constant-product/target \
	       contracts/nep11-nft/target contracts/multisig-wallet/target contracts/escrow/target \
	       contracts/crowdfunding/target contracts/governance-dao/target contracts/oracle-consumer/target \
	       contracts/nft-marketplace/target
	rm -rf wasm-neovm/target rust-devpack/target
	$(MAKE) -C spec clean

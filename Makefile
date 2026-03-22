# Top-level automation for building and translating Rust → NeoVM examples

MAKEFLAGS += --no-builtin-rules
.DEFAULT_GOAL := help

WASM_TARGET       := wasm32-unknown-unknown
OUTDIR            := build
TRANSLATOR        := cargo run --manifest-path wasm-neovm/Cargo.toml --quiet --

CONTRACT_RUSTFLAGS := -C opt-level=z -C strip=symbols -C panic=abort -C target-feature=-simd128,-reference-types,-multivalue,-tail-call
WASM_SNIP         ?= wasm-snip

HELLO_WASM        := contracts/hello-world/target/$(WASM_TARGET)/release/hello_world_neo.wasm
HELLO_NEF         := $(OUTDIR)/HelloWorld.nef
HELLO_MANIFEST    := $(OUTDIR)/HelloWorld.manifest.json
NEP17_WASM        := contracts/nep17-token/target/$(WASM_TARGET)/release/nep17_token_neo.wasm
NEP17_NEF         := $(OUTDIR)/NEP17.nef
NEP17_MANIFEST    := $(OUTDIR)/NEP17.manifest.json
AMM_WASM          := contracts/constant-product/target/$(WASM_TARGET)/release/constant_product_neo.wasm
AMM_NEF           := $(OUTDIR)/AMM.nef
AMM_MANIFEST      := $(OUTDIR)/AMM.manifest.json
NEP11_WASM        := contracts/nep11-nft/target/$(WASM_TARGET)/release/nep11_nft_neo.wasm
NEP11_NEF         := $(OUTDIR)/NEP11.nef
NEP11_MANIFEST    := $(OUTDIR)/NEP11.manifest.json
NEP17_SNIP_WASM   := $(OUTDIR)/NEP17.snip.wasm
NEP11_SNIP_WASM   := $(OUTDIR)/NEP11.snip.wasm
AMM_SNIP_WASM     := $(OUTDIR)/AMM.snip.wasm
MULTISIG_WASM     := contracts/multisig-wallet/target/$(WASM_TARGET)/release/multisig_wallet_neo.wasm
MULTISIG_NEF      := $(OUTDIR)/MultisigWallet.nef
MULTISIG_MANIFEST := $(OUTDIR)/MultisigWallet.manifest.json
ESCROW_WASM       := contracts/escrow/target/$(WASM_TARGET)/release/escrow_neo.wasm
ESCROW_NEF        := $(OUTDIR)/Escrow.nef
ESCROW_MANIFEST   := $(OUTDIR)/Escrow.manifest.json
CROWD_WASM        := contracts/crowdfunding/target/$(WASM_TARGET)/release/crowdfunding_neo.wasm
CROWD_NEF         := $(OUTDIR)/Crowdfunding.nef
CROWD_MANIFEST    := $(OUTDIR)/Crowdfunding.manifest.json
GOV_WASM          := contracts/governance-dao/target/$(WASM_TARGET)/release/governance_dao_neo.wasm
GOV_NEF           := $(OUTDIR)/GovernanceDAO.nef
GOV_MANIFEST      := $(OUTDIR)/GovernanceDAO.manifest.json
ORACLE_WASM       := contracts/oracle-consumer/target/$(WASM_TARGET)/release/oracle_consumer_neo.wasm
ORACLE_NEF        := $(OUTDIR)/OracleConsumer.nef
ORACLE_MANIFEST   := $(OUTDIR)/OracleConsumer.manifest.json
MARKET_WASM       := contracts/nft-marketplace/target/$(WASM_TARGET)/release/nft_marketplace_neo.wasm
MARKET_NEF        := $(OUTDIR)/NFTMarketplace.nef
MARKET_MANIFEST   := $(OUTDIR)/NFTMarketplace.manifest.json
SOLANA_HELLO_WASM := contracts/solana-hello/target/$(WASM_TARGET)/release/solana_hello_neo.wasm
SOLANA_HELLO_NEF  := $(OUTDIR)/solana_hello.nef
SOLANA_HELLO_MANIFEST := $(OUTDIR)/solana_hello.manifest.json
MOVE_COIN_WASM    := contracts/move-coin/target/$(WASM_TARGET)/release/move_coin_neo.wasm
MOVE_COIN_NEF     := $(OUTDIR)/MoveCoin.nef
MOVE_COIN_MANIFEST := $(OUTDIR)/MoveCoin.manifest.json
UNISWAP_WASM      := contracts/uniswap-v2/target/$(WASM_TARGET)/release/uniswap_v2_neo.wasm
UNISWAP_NEF       := $(OUTDIR)/UniswapV2.nef
UNISWAP_MANIFEST  := $(OUTDIR)/UniswapV2.manifest.json
STAKING_WASM      := contracts/staking-rewards/target/$(WASM_TARGET)/release/staking_rewards_neo.wasm
STAKING_NEF       := $(OUTDIR)/StakingRewards.nef
STAKING_MANIFEST  := $(OUTDIR)/StakingRewards.manifest.json
TIMELOCK_WASM     := contracts/timelock-vault/target/$(WASM_TARGET)/release/timelock_vault_neo.wasm
TIMELOCK_NEF      := $(OUTDIR)/TimelockVault.nef
TIMELOCK_MANIFEST := $(OUTDIR)/TimelockVault.manifest.json
FLASHLOAN_WASM    := contracts/flashloan-pool/target/$(WASM_TARGET)/release/flashloan_pool_neo.wasm
FLASHLOAN_NEF     := $(OUTDIR)/FlashLoanPool.nef
FLASHLOAN_MANIFEST := $(OUTDIR)/FlashLoanPool.manifest.json
UNISWAP_SNIP_WASM := $(OUTDIR)/UniswapV2.snip.wasm
STAKING_SNIP_WASM := $(OUTDIR)/StakingRewards.snip.wasm
TIMELOCK_SNIP_WASM := $(OUTDIR)/TimelockVault.snip.wasm
FLASHLOAN_SNIP_WASM := $(OUTDIR)/FlashLoanPool.snip.wasm
PACKAGE_MANIFESTS := \
	wasm-neovm/Cargo.toml \
	rust-devpack/Cargo.toml \
	rust-devpack/neo-types/Cargo.toml \
	rust-devpack/neo-syscalls/Cargo.toml \
	rust-devpack/neo-runtime/Cargo.toml \
	rust-devpack/neo-macros/Cargo.toml \
	rust-devpack/neo-test/Cargo.toml \
	move-neovm/Cargo.toml \
	solana-compat/Cargo.toml

.PHONY: help examples cross-chain hello-world nep17-token constant-product nep11-nft uniswap-v2 staking-rewards timelock-vault flashloan-pool multisig-wallet escrow crowdfunding governance-dao oracle-consumer nft-marketplace solana-hello move-coin c-hello fmt lint test verify-contract-tests test-contracts test-cross-chain integration-tests smoke-neoxp security-check package-check spec clean fuzz fuzz-translate fuzz-translate-config fuzz-nef fuzz-numeric fuzz-all

help:
	@echo "Usage: make <target>"
	@echo
	@echo "Primary targets:"
	@echo "  examples        Build and translate all sample contracts"
	@echo "  hello-world     Generate HelloWorld.nef and manifest"
	@echo "  nep17-token     Generate NEP17.nef and manifest"
	@echo "  constant-product Generate AMM.nef and manifest"
	@echo "  nep11-nft       Generate NEP11.nef and manifest"
	@echo "  uniswap-v2      Generate UniswapV2.nef and manifest"
	@echo "  staking-rewards Generate StakingRewards.nef and manifest"
	@echo "  timelock-vault  Generate TimelockVault.nef and manifest"
	@echo "  flashloan-pool  Generate FlashLoanPool.nef and manifest"
	@echo "  multisig-wallet Generate MultisigWallet.nef and manifest"
	@echo "  escrow          Generate Escrow.nef and manifest"
	@echo "  crowdfunding    Generate Crowdfunding.nef and manifest"
	@echo "  governance-dao  Generate GovernanceDAO.nef and manifest"
	@echo "  oracle-consumer Generate OracleConsumer.nef and manifest"
	@echo "  nft-marketplace Generate NFTMarketplace.nef and manifest"
	@echo "  cross-chain     Build Solana/Move examples (solana-hello, move-coin)"
	@echo "  solana-hello    Generate solana_hello.nef and manifest (cross-chain sample)"
	@echo "  move-coin       Generate MoveCoin.nef and manifest (cross-chain sample)"
	@echo "  c-hello        Build the sample C contract and translate it"
	@echo "  c-hello-optional Build the C sample when wasm-ld is present"
	@echo
	@echo "Maintenance targets:"
	@echo "  fmt             Run cargo fmt across the workspace"
	@echo "  lint            Run cargo clippy across the workspace"
	@echo "  test            Execute translator/devpack/neo-test + contract unit suites"
	@echo "  verify-contract-tests Ensure each Rust contract crate defines tests"
	@echo "  test-contracts  Run unit tests for all Rust sample contracts"
	@echo "  test-cross-chain Run wasm-neovm cross-chain test suites"
	@echo "  integration-tests  Run optional Neo Express integration harness"
	@echo "  smoke-neoxp     Run local Neo Express deploy/invoke smoke checks"
	@echo "  security-check Run cargo-audit/cargo-deny checks (requires cargo-audit/cargo-deny)"
	@echo "  package-check   Verify all publishable crates can be packaged"
	@echo "  unused-deps     Check for unused dependencies (requires cargo-machete)"
	@echo "  outdated        Check for outdated dependencies (requires cargo-outdated)"
	@echo "  version-check   Verify version consistency across workspace"
	@echo "  doc             Generate API documentation"
	@echo "  quality-check   Run all quality checks (fmt, lint, test, security, version)"
	@echo "  spec            Build the LaTeX specification in spec/"
	@echo "  clean           Remove generated build artefacts"
	@echo
	@echo "Fuzz targets (requires cargo-fuzz + nightly):"
	@echo "  fuzz             Run primary fuzz target (translate) for 5 minutes"
	@echo "  fuzz-translate   Fuzz the WASM translator with arbitrary bytes"
	@echo "  fuzz-translate-config  Fuzz translator with varied config fields"
	@echo "  fuzz-nef         Fuzz NEF serialization"
	@echo "  fuzz-numeric     Fuzz numeric encoding (push_biginteger/push_bytevec)"
	@echo "  fuzz-all         Run all fuzz targets sequentially"

examples: hello-world nep17-token constant-product nep11-nft uniswap-v2 staking-rewards timelock-vault flashloan-pool multisig-wallet escrow crowdfunding governance-dao oracle-consumer nft-marketplace c-hello-optional cross-chain

cross-chain: solana-hello move-coin

hello-world: $(HELLO_NEF) $(HELLO_MANIFEST)
	@echo "✔ hello-world artifacts are in $(OUTDIR)/"

nep17-token: $(NEP17_NEF) $(NEP17_MANIFEST)
	@echo "✔ NEP-17 artifacts are in $(OUTDIR)/"

constant-product: $(AMM_NEF) $(AMM_MANIFEST)
	@echo "✔ Constant-product AMM artifacts are in $(OUTDIR)/"

nep11-nft: $(NEP11_NEF) $(NEP11_MANIFEST)
	@echo "✔ NEP-11 artifacts are in $(OUTDIR)/"

uniswap-v2: $(UNISWAP_NEF) $(UNISWAP_MANIFEST)
	@echo "✔ Uniswap V2 artifacts are in $(OUTDIR)/"

staking-rewards: $(STAKING_NEF) $(STAKING_MANIFEST)
	@echo "✔ Staking rewards artifacts are in $(OUTDIR)/"

timelock-vault: $(TIMELOCK_NEF) $(TIMELOCK_MANIFEST)
	@echo "✔ Timelock vault artifacts are in $(OUTDIR)/"

flashloan-pool: $(FLASHLOAN_NEF) $(FLASHLOAN_MANIFEST)
	@echo "✔ Flashloan pool artifacts are in $(OUTDIR)/"

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
	$(WASM_SNIP) --snip-rust-fmt-code --snip-rust-panicking-code $(NEP17_WASM) -o $(NEP17_SNIP_WASM)
	$(TRANSLATOR) \
	  --input $(NEP17_SNIP_WASM) \
	  --nef $(NEP17_NEF) \
	  --manifest $(NEP17_MANIFEST) \
	  --name SampleNEP17

$(AMM_NEF) $(AMM_MANIFEST): $(AMM_WASM) | $(OUTDIR)
	$(WASM_SNIP) --snip-rust-fmt-code --snip-rust-panicking-code $(AMM_WASM) -o $(AMM_SNIP_WASM)
	$(TRANSLATOR) \
	  --input $(AMM_SNIP_WASM) \
	  --nef $(AMM_NEF) \
	  --manifest $(AMM_MANIFEST) \
	  --name ConstantProductAMM

$(NEP11_NEF) $(NEP11_MANIFEST): $(NEP11_WASM) | $(OUTDIR)
	$(WASM_SNIP) --snip-rust-fmt-code --snip-rust-panicking-code $(NEP11_WASM) -o $(NEP11_SNIP_WASM)
	$(TRANSLATOR) \
	  --input $(NEP11_SNIP_WASM) \
	  --nef $(NEP11_NEF) \
	  --manifest $(NEP11_MANIFEST) \
	  --name SampleNEP11

$(UNISWAP_NEF) $(UNISWAP_MANIFEST): $(UNISWAP_WASM) | $(OUTDIR)
	$(WASM_SNIP) --snip-rust-fmt-code --snip-rust-panicking-code $(UNISWAP_WASM) -o $(UNISWAP_SNIP_WASM)
	$(TRANSLATOR) \
	  --input $(UNISWAP_SNIP_WASM) \
	  --nef $(UNISWAP_NEF) \
	  --manifest $(UNISWAP_MANIFEST) \
	  --name UniswapV2Router

$(STAKING_NEF) $(STAKING_MANIFEST): $(STAKING_WASM) | $(OUTDIR)
	$(WASM_SNIP) --snip-rust-fmt-code --snip-rust-panicking-code $(STAKING_WASM) -o $(STAKING_SNIP_WASM)
	$(TRANSLATOR) \
	  --input $(STAKING_SNIP_WASM) \
	  --nef $(STAKING_NEF) \
	  --manifest $(STAKING_MANIFEST) \
	  --name StakingRewards

$(TIMELOCK_NEF) $(TIMELOCK_MANIFEST): $(TIMELOCK_WASM) | $(OUTDIR)
	$(WASM_SNIP) --snip-rust-fmt-code --snip-rust-panicking-code $(TIMELOCK_WASM) -o $(TIMELOCK_SNIP_WASM)
	$(TRANSLATOR) \
	  --input $(TIMELOCK_SNIP_WASM) \
	  --nef $(TIMELOCK_NEF) \
	  --manifest $(TIMELOCK_MANIFEST) \
	  --name TimelockVault

$(FLASHLOAN_NEF) $(FLASHLOAN_MANIFEST): $(FLASHLOAN_WASM) | $(OUTDIR)
	$(WASM_SNIP) --snip-rust-fmt-code --snip-rust-panicking-code $(FLASHLOAN_WASM) -o $(FLASHLOAN_SNIP_WASM)
	$(TRANSLATOR) \
	  --input $(FLASHLOAN_SNIP_WASM) \
	  --nef $(FLASHLOAN_NEF) \
	  --manifest $(FLASHLOAN_MANIFEST) \
	  --name FlashLoanPool

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

solana-hello: $(SOLANA_HELLO_NEF) $(SOLANA_HELLO_MANIFEST)
	@echo "✔ solana-hello artifacts are in $(OUTDIR)/"

move-coin: $(MOVE_COIN_NEF) $(MOVE_COIN_MANIFEST)
	@echo "✔ move-coin artifacts are in $(OUTDIR)/"

$(SOLANA_HELLO_NEF) $(SOLANA_HELLO_MANIFEST): $(SOLANA_HELLO_WASM) | $(OUTDIR)
	$(TRANSLATOR) \
	  --input $(SOLANA_HELLO_WASM) \
	  --nef $(SOLANA_HELLO_NEF) \
	  --manifest $(SOLANA_HELLO_MANIFEST) \
	  --name solana-hello \
	  --source-chain solana

$(MOVE_COIN_NEF) $(MOVE_COIN_MANIFEST): $(MOVE_COIN_WASM) | $(OUTDIR)
	$(TRANSLATOR) \
	  --input $(MOVE_COIN_WASM) \
	  --nef $(MOVE_COIN_NEF) \
	  --manifest $(MOVE_COIN_MANIFEST) \
	  --name MoveCoin \
	  --source-chain move

$(HELLO_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	cargo build --manifest-path contracts/hello-world/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(NEP17_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	cargo build --manifest-path contracts/nep17-token/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(AMM_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	cargo build --manifest-path contracts/constant-product/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(NEP11_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	cargo build --manifest-path contracts/nep11-nft/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(UNISWAP_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	cargo build --manifest-path contracts/uniswap-v2/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(STAKING_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	cargo build --manifest-path contracts/staking-rewards/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(TIMELOCK_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	cargo build --manifest-path contracts/timelock-vault/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(FLASHLOAN_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	cargo build --manifest-path contracts/flashloan-pool/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(MULTISIG_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	cargo build --manifest-path contracts/multisig-wallet/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(ESCROW_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	cargo build --manifest-path contracts/escrow/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(CROWD_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	cargo build --manifest-path contracts/crowdfunding/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(GOV_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	cargo build --manifest-path contracts/governance-dao/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(ORACLE_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	cargo build --manifest-path contracts/oracle-consumer/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(MARKET_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	cargo build --manifest-path contracts/nft-marketplace/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(SOLANA_HELLO_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	  cargo build --manifest-path contracts/solana-hello/Cargo.toml --release --target $(WASM_TARGET) --quiet

$(MOVE_COIN_WASM):
	RUSTFLAGS="$(CONTRACT_RUSTFLAGS)" \
	  cargo build --manifest-path contracts/move-coin/Cargo.toml --release --target $(WASM_TARGET) --quiet

c-hello:
	scripts/build_c_contract.sh contracts/c-hello

c-hello-optional:
	@CLANG_BIN="$${CLANG:-clang}"; \
	  major=""; \
	  if command -v "$$CLANG_BIN" >/dev/null 2>&1; then \
	    major="$$( "$$CLANG_BIN" --version 2>/dev/null | sed -n 's/.*version \\([0-9][0-9]*\\)\\..*/\\1/p' | head -n 1 )"; \
	  fi; \
	  if command -v wasm-ld >/dev/null 2>&1; then \
	    $(MAKE) c-hello; \
	  elif [ -n "$$major" ] && command -v "wasm-ld-$$major" >/dev/null 2>&1; then \
	    $(MAKE) c-hello; \
	  else \
	    echo "skipping c-hello: missing wasm-ld (install lld to enable)"; \
	  fi

$(OUTDIR):
	@mkdir -p $(OUTDIR)

fmt:
	cargo fmt --manifest-path wasm-neovm/Cargo.toml
	cargo fmt --manifest-path move-neovm/Cargo.toml
	cargo fmt --manifest-path solana-compat/Cargo.toml
	cargo fmt --manifest-path rust-devpack/Cargo.toml
	cargo fmt --manifest-path rust-devpack/neo-test/Cargo.toml
	cargo fmt --manifest-path integration-tests/Cargo.toml

lint:
	cargo clippy --manifest-path wasm-neovm/Cargo.toml --all-targets --all-features -- -D warnings
	cargo clippy --manifest-path move-neovm/Cargo.toml --all-targets --all-features -- -D warnings
	cargo clippy --manifest-path solana-compat/Cargo.toml --all-targets --all-features -- -D warnings
	cargo clippy --manifest-path rust-devpack/Cargo.toml --all-targets --all-features -- -D warnings
	cargo clippy --manifest-path rust-devpack/neo-test/Cargo.toml --all-targets --all-features -- -D warnings
	cargo clippy --manifest-path integration-tests/Cargo.toml --all-targets --all-features -- -D warnings

test:
	cargo test --manifest-path wasm-neovm/Cargo.toml
	cargo test --manifest-path rust-devpack/Cargo.toml
	cargo test --manifest-path rust-devpack/neo-test/Cargo.toml
	$(MAKE) test-contracts

verify-contract-tests:
	scripts/verify_contract_tests.sh

verify-neo-n3-conformance:
	bash scripts/check_neo_n3_conformance_matrix.sh

test-contracts:
	$(MAKE) verify-contract-tests
	@set -e; \
	for manifest in $$(find contracts -name Cargo.toml ! -path 'contracts/Cargo.toml' | sort); do \
		echo "==> cargo test --manifest-path $$manifest"; \
		cargo test --manifest-path "$$manifest" --quiet; \
	done

test-cross-chain:
	cargo test --manifest-path wasm-neovm/Cargo.toml --test cross_chain_tests --test solana_move_integration

integration-tests:
	@echo "Running integration tests (requires NEO_EXPRESS_RPC)..."
	cargo test --manifest-path integration-tests/Cargo.toml -- --ignored

smoke-neoxp:
	scripts/neoxp_smoke.sh

security-check:
	scripts/run_cargo_audit.sh wasm-neovm/Cargo.lock
	scripts/run_cargo_audit.sh move-neovm/Cargo.lock
	scripts/run_cargo_audit.sh rust-devpack/Cargo.lock
	scripts/run_cargo_audit.sh solana-compat/Cargo.lock
	scripts/run_cargo_audit.sh integration-tests/Cargo.lock
	scripts/run_cargo_deny.sh wasm-neovm/Cargo.toml
	scripts/run_cargo_deny.sh move-neovm/Cargo.toml
	scripts/run_cargo_deny.sh rust-devpack/Cargo.toml

package-check:
	@set -e; \
	for manifest in $(PACKAGE_MANIFESTS); do \
		echo "==> cargo package --manifest-path $$manifest --allow-dirty"; \
		cargo package --manifest-path "$$manifest" --allow-dirty; \
	done

# Check for unused dependencies (requires cargo-machete)
unused-deps:
	@echo "Checking for unused dependencies (install with: cargo install cargo-machete)..."
	cargo machete --with-metadata

# Check for outdated dependencies (requires cargo-outdated)
outdated:
	@echo "Checking for outdated dependencies (install with: cargo install cargo-outdated)..."
	cargo outdated --workspace --root-deps-only

# Check version consistency
version-check:
	@echo "Checking version consistency..."
	@scripts/check_versions.sh

# Generate documentation
doc:
	cargo doc --manifest-path wasm-neovm/Cargo.toml --no-deps --document-private-items
	cargo doc --manifest-path rust-devpack/Cargo.toml --no-deps

# Run all quality checks
quality-check: fmt lint test security-check package-check version-check
	@echo "✅ All quality checks passed!"

spec:
	$(MAKE) -C spec

fuzz: fuzz-translate  ## Run primary fuzz target (translate) for 5 minutes
fuzz-translate:
	cd wasm-neovm && cargo +nightly fuzz run fuzz_translate -- -max_total_time=300
fuzz-translate-config:
	cd wasm-neovm && cargo +nightly fuzz run fuzz_translate_config -- -max_total_time=300
fuzz-nef:
	cd wasm-neovm && cargo +nightly fuzz run fuzz_nef -- -max_total_time=300
fuzz-numeric:
	cd wasm-neovm && cargo +nightly fuzz run fuzz_numeric -- -max_total_time=300
fuzz-all: fuzz-translate fuzz-translate-config fuzz-nef fuzz-numeric  ## Run all fuzz targets sequentially

clean:
	rm -rf $(OUTDIR)
	rm -rf contracts/hello-world/target contracts/nep17-token/target contracts/constant-product/target \
	       contracts/nep11-nft/target contracts/uniswap-v2/target contracts/staking-rewards/target \
	       contracts/timelock-vault/target contracts/flashloan-pool/target contracts/multisig-wallet/target contracts/escrow/target \
	       contracts/crowdfunding/target contracts/governance-dao/target contracts/oracle-consumer/target \
	       contracts/nft-marketplace/target contracts/solana-hello/target contracts/move-coin/target
	rm -rf wasm-neovm/target rust-devpack/target
	$(MAKE) -C spec clean

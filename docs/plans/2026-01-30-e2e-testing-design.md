# Neo N3 Smart Contracts E2E Testing Design

## Overview

Design for comprehensive end-to-end (E2E) testing of all 10 Neo N3 smart contracts using `neo-testengine` for integration tests and Rust built-in testing for unit tests.

## Goals

1. Add E2E tests for all 10 contracts
2. Use both unit tests (Rust built-in) and integration tests (neo-testengine)
3. Add cross-contract integration tests for complex workflows
4. Ensure production readiness with comprehensive test coverage

## Architecture

```
contracts/
├── <contract>/
│   ├── src/
│   │   └── lib.rs              # Contract source code
│   ├── Cargo.toml              # Dependencies
│   └── tests/                  # E2E test directory
│       ├── integration.rs      # neo-testengine tests
│       ├── cross_contract.rs   # Cross-contract tests
│       └── mod.rs              # Module declaration
```

## Testing Strategy

### Test Types

| Type | Framework | Target | Coverage |
|------|-----------|--------|----------|
| Unit Tests | Rust `#[test]` | Individual functions | 70% |
| Integration Tests | neo-testengine | Deployed contracts | 90% |
| Cross-contract Tests | neo-testengine | Multi-contract interaction | 100% |

### Test Pyramid

```
        /\
       /  \
      /E2E \          Cross-contract (5-10%)
     /      \
    /Integr \         Integration (30-40%)
   /ation    \
  /Tests      \
 /            \
/Unit Tests---/  Unit tests (50-60%)
```

## Test Scenarios by Contract

### 1. hello-world

| Test Case | Description | Expected Result |
|-----------|-------------|-----------------|
| greet_basic | Call greet() | Returns "Hello, World!" |
| greet_custom | Call greet(name) | Returns "Hello, {name}!" |
| greet_empty | Call greet("") | Returns "Hello, !" |

### 2. nep17-token

| Test Case | Description | Expected Result |
|-----------|-------------|-----------------|
| init_owner | Initialize with owner | Owner has total supply |
| init_invalid | Initialize with amount=0 | Returns false |
| transfer_basic | Transfer tokens | Balance updated, event emitted |
| transfer_self | Transfer to self | Returns false |
| transfer_no_funds | Transfer without balance | Returns false |
| transfer_witness | Transfer without witness | Returns false |
| balanceOf | Query balance | Returns correct balance |
| totalSupply | Query supply | Returns total supply |
| onNEP17Payment | Receive tokens | Balance increases |

### 3. nep11-nft

| Test Case | Description | Expected Result |
|-----------|-------------|-----------------|
| mint_single | Mint one NFT | Token created, balance=1 |
| mint_multiple | Mint multiple NFTs | Multiple tokens created |
| transfer_basic | Transfer NFT | Owner updated, event emitted |
| transfer_self | Transfer to self | Returns false |
| approve | Approve operator | Operator set |
| transferApproved | Transfer approved | Operator can transfer |
| balanceOf | Query owner balance | Returns token count |
| tokensOf | Query owned tokens | Returns token IDs |
| ownerOf | Query token owner | Returns owner address |

### 4. constant-product (AMM)

| Test Case | Description | Expected Result |
|-----------|-------------|-----------------|
| init_pool | Initialize pool | Pool created with initial liquidity |
| addLiquidity | Add liquidity | LP tokens minted |
| swap_token0_for_token1 | Swap | Output amount calculated with fee |
| swap_token1_for_token0 | Swap | Output amount calculated with fee |
| removeLiquidity | Remove liquidity | Base tokens returned, LP burned |
| getAmountOut | Quote swap | Returns expected output |
| token0Balance | Query token0 | Returns balance |
| token1Balance | Query token1 | Returns balance |

### 5. crowdfunding

| Test Case | Description | Expected Result |
|-----------|-------------|-----------------|
| create_campaign | Create new campaign | Campaign created |
| pledge | Pledge tokens | Pledge recorded, balance updated |
| pledge_invalid | Pledge to ended campaign | Returns false |
| claim | Campaign success, creator claims | Creator receives funds |
| refund | Campaign failed, backers refund | Backers receive refund |
| cancel_early | Creator cancels early | Campaign canceled |
| getCampaign | Query campaign details | Returns all fields |

### 6. escrow

| Test Case | Description | Expected Result |
|-----------|-------------|-----------------|
| create_escrow | Create escrow | Escrow created with conditions |
| deposit | Seller deposits asset | Asset escrowed |
| confirm_deposit | Buyer confirms deposit | Deposit confirmed |
| complete | Buyer completes | Seller paid, asset transferred |
| refund_buyer | Conditions not met, refund | Buyer refunded |
| cancel | Early cancellation | Parties refunded |
| getEscrow | Query escrow state | Returns all details |

### 7. governance-dao

| Test Case | Description | Expected Result |
|-----------|-------------|-----------------|
| create_proposal | Create proposal | Proposal created |
| vote_for | Vote for proposal | Vote recorded |
| vote_against | Vote against proposal | Vote recorded |
| execute | Proposal passes, execute | Proposal executed |
| execute_early | Execute before end | Returns false |
| delegate | Delegate voting power | Delegation recorded |
| getProposal | Query proposal | Returns all details |
| getVote | Query vote | Returns vote choice |

### 8. multisig-wallet

| Test Case | Description | Expected Result |
|-----------|-------------|-----------------|
| create_multisig | Create multisig wallet | Wallet created with threshold |
| propose_transaction | Propose transaction | Transaction proposed |
| approve_single | Approve (threshold=1) | Transaction approved |
| approve_multi | Approve (threshold>1) | Approval recorded |
| execute_approved | Execute approved tx | Transaction executed |
| execute_no_approvals | Execute without approvals | Returns false |
| revoke | Revoke approval | Approval removed |
| getTransaction | Query transaction | Returns all details |

### 9. nft-marketplace

| Test Case | Description | Expected Result |
|-----------|-------------|-----------------|
| list_item | List NFT for sale | Item listed with price |
| buy_item | Buy listed item | NFT transferred, seller paid |
| make_offer | Make offer on NFT | Offer recorded |
| accept_offer | Accept offer | NFT transferred, offerer pays |
| cancel_listing | Cancel listing | Listing removed |
| update_price | Update listing price | Price updated |
| getListing | Query listing | Returns all details |

### 10. oracle-consumer

| Test Case | Description | Expected Result |
|-----------|-------------|-----------------|
| configure | Configure oracle | Oracle URL and keys set |
| request_data | Request external data | Request queued |
| callback | Oracle callback | Data stored |
| getData | Query fetched data | Returns data |
| getRequest | Query request state | Returns request details |

## Cross-Contract Integration Tests

### 1. NFT Marketplace + NEP-17 Token

```
Test: Purchase NFT with NEP-17 Tokens

Setup:
- Deploy NEP-17 token contract
- Deploy NFT contract
- Deploy NFT Marketplace
- User A mints NFT, lists on marketplace
- User B gets NEP-17 tokens

Steps:
1. User B calls marketplace.buy() with NEP-17 payment
2. Marketplace transfers NEP-17 from buyer to seller
3. Marketplace transfers NFT from seller to buyer

Assertion:
- Buyer has NFT
- Seller received NEP-17 tokens
- Events emitted correctly
```

### 2. Crowdfunding + NEP-17 Token

```
Test: Campaign with NEP-17 Token Pledges

Setup:
- Deploy NEP-17 token
- Deploy Crowdfunding

Steps:
1. Creator creates campaign accepting NEP-17
2. Backers pledge using NEP-17 transfer
3. Campaign reaches goal
4. Creator claims funds
5. Alternatively: Campaign fails, backers refund

Assertion:
- Campaign tracks pledges correctly
- Creator receives NEP-17 on success
- Backers refunded on failure
```

### 3. Escrow with Multi-Asset

```
Test: NFT + NEP-17 Token Escrow

Setup:
- Deploy NEP-17 token
- Deploy NFT
- Deploy Escrow

Steps:
1. Seller creates escrow for NFT with NEP-17 price
2. Seller deposits NFT to escrow
3. Buyer pays NEP-17 to escrow
4. Buyer confirms
5. Escrow transfers NFT to buyer, NEP-17 to seller

Assertion:
- NFT transferred to buyer
- NEP-17 transferred to seller
- Both parties cannot double-spend
```

## neo-testengine Configuration

### Dependencies

```toml
[dev-dependencies]
neo-testengine = "0.2"  # Check latest version
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Test Engine Setup

```rust
use neo_testengine::{TestEngine, Account, Contract};

fn setup_test_engine() -> TestEngine {
    let mut engine = TestEngine::new();
    engine.add_neo_genesis_accounts(10); // 10 genesis accounts with 10000 NEO each
    engine.add_gas_genesis_accounts(10); // 10 genesis accounts with 10000000 GAS each
    engine
}

fn deploy_contract<T: serde::Serialize>(
    engine: &mut TestEngine,
    contract_path: &str,
    params: &T,
) -> Contract {
    let nef = std::fs::read(contract_path).unwrap();
    engine.deploy(&nef, params)
}
```

### Example Test

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use neo_testengine::{TestEngine, Account};

    #[test]
    fn test_nep17_transfer() {
        let mut engine = TestEngine::new();

        // Setup accounts
        let owner = engine.accounts[0].clone();
        let alice = engine.accounts[1].clone();
        let bob = engine.accounts[2].clone();

        // Deploy NEP-17 contract
        let contract = engine.deploy_contract("./target/nef.contracts.nep17.nef");

        // Initialize with 1,000,000 tokens to owner
        let init_result = contract.call("init", &[
            owner.address.as_parameter(),
            1_000_000_i64
        ]);
        assert!(init_result.is_ok());

        // Transfer 100 tokens from owner to alice
        let transfer_result = contract.call("transfer", &[
            owner.address.as_parameter(),
            alice.address.as_parameter(),
            100_i64,
            None::<i64>
        ]);
        assert!(transfer_result.is_ok());

        // Check balances
        let alice_balance = contract.call("balanceOf", &[alice.address.as_parameter()]);
        assert_eq!(alice_balance.unwrap(), 100_i64);
    }
}
```

## Test Coverage Requirements

### Minimum Coverage

| Metric | Target |
|--------|--------|
| Line Coverage | 80% |
| Function Coverage | 90% |
| Branch Coverage | 70% |
| Path Coverage | 60% |

### Critical Paths

All contracts must test:
- Initialization paths
- Authorization/witness checks
- State transitions
- Error conditions
- Event emissions

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: E2E Tests

on:
  push:
    branches: [master]
  pull_request:
    paths: [contracts/]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          target: wasm32-unknown-unknown
      - name: Build Contracts
        run: |
          for dir in contracts/*/; do
            cargo build --manifest-path "$dir"Cargo.toml" --target wasm32-unknown-unknown --release
          done
      - name: Run Unit Tests
        run: |
          for dir in contracts/*/; do
            cargo test --manifest-path "$dir"Cargo.toml" --lib
          done
      - name: Run Integration Tests
        run: |
          for dir in contracts/*/; do
            cargo test --manifest-path "$dir"Cargo.toml" --test integration
          done
      - name: Run Cross-Contract Tests
        run: |
          cargo test --test cross_contract
```

## Implementation Order

### Phase 1: Core Contracts
1. hello-world (simplest)
2. nep17-token (foundational)
3. nep11-nft (complex NFT logic)

### Phase 2: DeFi Contracts
4. constant-product (AMM)
5. crowdfunding
6. escrow

### Phase 3: Governance & Utility
7. governance-dao
8. multisig-wallet
9. nft-marketplace

### Phase 4: Advanced
10. oracle-consumer
11. Cross-contract integration tests

## Success Criteria

1. All 10 contracts have E2E tests
2. Test coverage > 80% for all contracts
3. All critical paths tested
4. Cross-contract scenarios pass
5. Tests run in CI/CD pipeline
6. No regressions in existing functionality

## Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| Phase 1 | 2 days | 3 contracts tested |
| Phase 2 | 3 days | 3 contracts tested |
| Phase 3 | 3 days | 3 contracts tested |
| Phase 4 | 2 days | 1 contract + integration tests |
| CI/CD | 1 day | GitHub Actions workflow |

**Total: ~11 days**

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| neo-testengine API changes | High | Pin to specific version |
| NEF compatibility | Medium | Test with compiled NEF files |
| Cross-contract complexity | Medium | Start with simple scenarios |
| Test execution time | Low | Parallel test execution |

## References

- [neo-testengine GitHub](https://github.com/neo-project/neo-testengine)
- [Neo N3 Documentation](https://docs.neo.org/)
- [NEP-17 Standard](https://github.com/neo-project/proposals/blob/master/nep-17.mediawiki)
- [NEP-11 Standard](https://github.com/neo-project/proposals/blob/master/nep-11.mediawiki)

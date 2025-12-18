# Neo Express Integration Tests

The repository includes an optional integration test crate under `integration-tests/` to help
exercise generated NEF artefacts against a live [Neo Express](https://github.com/neo-project/neo-express)
instance.

## Prerequisites

1. Build the sample contracts and translate them to NEF + manifest:
   ```bash
   make examples          # Neo-native samples
   make cross-chain       # Solana + experimental Move samples
   ```
2. Start Neo Express (or point to an existing instance) and note its RPC endpoint
   (for example `http://localhost:50012`).

## Running the Tests

1. Export the RPC endpoint so the test harness knows where to connect:
   ```bash
   export NEO_EXPRESS_RPC=http://localhost:50012
   ```
2. Execute the integration suite (tests are ignored by default so they must be
   invoked explicitly):
   ```bash
   cargo test --manifest-path integration-tests/Cargo.toml -- --ignored
   ```

The harness verifies that the translated artefacts exist and surfaces their sizes.
Augment the test with project-specific deployment logic (for example by invoking
`neo-express contract deploy`) as you plug the suite into CI.

## Deployment Helper Script

For convenience the repo ships with `scripts/neoexpress_deploy.sh`. After setting
`NEO_EXPRESS_RPC` (and optionally `NEO_EXPRESS_CLI` when the CLI is not on
`PATH`), deploy a compiled contract with:

```bash
scripts/neoexpress_deploy.sh build/HelloWorld.nef build/HelloWorld.manifest.json HelloWorld
```

Pass additional `neo-express contract deploy` flags after the contract name, for
example `--account <script-hash>` to specify the signer.

## Sample Deploy + Invoke Flow

Once a contract is deployed, Neo Express prints the resulting script hash. Use
that hash with `neo-express contract invoke` to exercise the exported methods.
All examples expose at least one *safe* method, so you can validate the deployment
without additional witness configuration:

| Contract | Safe Validation Call |
| --- | --- |
| Hello World (neo) | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> hello` |
| NEP-17 Token | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> totalSupply` |
| Constant-product AMM | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> getReserves` |
| NEP-11 NFT | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> totalSupply` |
| Multisig Wallet | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> getConfig` |
| Escrow | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> getState` |
| Crowdfunding | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> getCampaign` |
| Governance DAO | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> getConfig` |
| Oracle Consumer | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> getConfig` |
| NFT Marketplace | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> getListing --integer 1` |
| Solana Hello (cross-chain) | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> hello` |
| Move Coin (cross-chain, experimental) | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> balance --binary <address>` |

For view calls that expect parameters (for example `balanceOf(owner)` or
`contributionOf(address)`), supply the argument in the same form you would pass
to the contract (integers are accepted as little-endian byte arrays or decimal
values; refer to `neo-express contract invoke --help` for encoding options).

State-changing operations (minting, transferring, voting, etc.) require a signer.
Provide the account by adding `--wallet`/`--account` flags to the `contract invoke`
command or by extending the deployment script to mint initial state before the
validation calls.

With these commands you can stand up a Neo Express node, deploy any of the sample
contracts, and confirm that the exported entry points respond as expected.

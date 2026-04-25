# Neo Express Integration Tests

The repository includes a Neo Express integration test crate under `integration-tests/` to help
exercise generated NEF artefacts against a live [Neo Express](https://github.com/neo-project/neo-express)
instance. The default local `make test` path does not cover runtime deployment; use the
Neo Express targets below when validating end-to-end behavior.

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
   make integration-tests
   ```

The harness verifies that the translated artefacts exist and surfaces their sizes.
Augment the test with project-specific deployment logic (for example by invoking
`neo-express contract deploy`) as you plug the suite into CI.

For the CI-equivalent runtime validation path, run:

```bash
make smoke-neoxp
```

This path provisions Neo Express, deploys the bundled sample contracts, and performs
direct invoke checks for the runtime-safe smoke subset via `scripts/neoxp_smoke.sh`.
Stateful witness-heavy examples are deployed in this job and covered functionally by
their Rust test suites.

## CI Smoke Job

The CI workflow includes a gating `Neo Express Smoke` job that provisions Neo Express,
builds/translates all sample contracts, deploys them, and runs invoke checks via
`scripts/neoxp_smoke.sh`.

The job runs on every CI execution, and failures now fail the workflow.

This increases CI duration because it installs Neo Express tooling and runs
full build/deploy checks across the contract suite plus direct invokes for the
runtime-safe smoke subset.

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
that hash with `neo-express contract invoke` to exercise exported methods that do
not require witness or state setup. Some advanced examples are deploy-validated in
`make smoke-neoxp` and should be exercised through their Rust tests until a richer
Neo Express fixture provisions witnesses and contract state.

| Contract | Safe Validation Call |
| --- | --- |
| Hello World (neo) | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> hello` |
| NEP-17 Token | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> totalSupply` |
| Constant-product AMM | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> getReserves` |
| NEP-11 NFT | `neo-express contract invoke --rpc $NEO_EXPRESS_RPC <hash> totalSupply` |
| Multisig Wallet | Deploy validation plus Rust contract tests for witness-gated flows |
| Escrow | Deploy validation plus Rust contract tests for stateful flows |
| Crowdfunding | Deploy validation plus Rust contract tests for witness-gated flows |
| Governance DAO | Deploy validation plus Rust contract tests for stateful flows |
| Oracle Consumer | Deploy validation plus Rust contract tests for witness-gated callbacks |
| NFT Marketplace | Deploy validation plus Rust contract tests for stateful flows |
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

import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://neo-rust.vercel.app',
  integrations: [
    starlight({
      title: 'neo-llvm',
      logo: {
        src: './src/assets/logo.svg',
      },
      social: {
        github: 'https://github.com/r3e-network/neo-llvm',
        discord: 'https://discord.gg/neo',
      },
      customCss: ['./src/styles/custom.css'],
      expressiveCode: {
        themes: ['github-dark'],
        styleOverrides: {
          borderColor: 'var(--sl-color-gray-4)',
          borderRadius: '8px',
        },
      },
      editLink: {
        baseUrl: 'https://github.com/r3e-network/neo-llvm/edit/master/website/',
      },
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Introduction', slug: 'introduction' },
            { label: 'Installation', slug: 'installation' },
            { label: 'Quick Start', slug: 'quickstart' },
            { label: 'Project Structure', slug: 'project-structure' },
          ],
        },
        {
          label: 'Core Concepts',
          items: [
            { label: 'Compilation Pipeline', slug: 'concepts/pipeline' },
            { label: 'Contract Anatomy', slug: 'concepts/contract-anatomy' },
            { label: 'NEF & Manifest', slug: 'concepts/nef-manifest' },
            { label: 'Manifest Overlays', slug: 'concepts/manifest-overlays' },
            { label: 'NeoVM Execution', slug: 'concepts/neovm-execution' },
          ],
        },
        {
          label: 'SDK Reference',
          items: [
            { label: 'Overview', slug: 'sdk/overview' },
            {
              label: 'Macros',
              items: [
                { label: '#[neo_contract]', slug: 'sdk/macros/neo-contract' },
                { label: '#[neo_method]', slug: 'sdk/macros/neo-method' },
                { label: '#[neo_event]', slug: 'sdk/macros/neo-event' },
                { label: '#[neo_safe]', slug: 'sdk/macros/neo-safe' },
                { label: '#[neo_entry]', slug: 'sdk/macros/neo-entry' },
                { label: '#[neo_storage]', slug: 'sdk/macros/neo-storage' },
                { label: 'Manifest Macros', slug: 'sdk/macros/manifest-macros' },
              ],
            },
            {
              label: 'Types',
              items: [
                { label: 'Primitives', slug: 'sdk/types/primitives' },
                { label: 'Strings', slug: 'sdk/types/strings' },
                { label: 'Collections', slug: 'sdk/types/collections' },
                { label: 'Iterator', slug: 'sdk/types/iterator' },
                { label: 'Value', slug: 'sdk/types/value' },
                { label: 'Error Handling', slug: 'sdk/types/error-handling' },
                { label: 'Storage Context', slug: 'sdk/types/storage-context' },
              ],
            },
            {
              label: 'Runtime',
              items: [
                { label: 'NeoRuntime', slug: 'sdk/runtime/neo-runtime' },
                { label: 'NeoStorage', slug: 'sdk/runtime/neo-storage' },
                { label: 'Contract Runtime', slug: 'sdk/runtime/contract-runtime' },
                { label: 'Crypto', slug: 'sdk/runtime/crypto' },
                { label: 'JSON', slug: 'sdk/runtime/json' },
                { label: 'Context', slug: 'sdk/runtime/context' },
              ],
            },
            { label: 'Syscalls', slug: 'sdk/syscalls' },
            { label: 'Testing', slug: 'sdk/testing' },
          ],
        },
        {
          label: 'Guides',
          items: [
            { label: 'Building Contracts', slug: 'guides/building' },
            { label: 'Storage Patterns', slug: 'guides/storage' },
            { label: 'Events', slug: 'guides/events' },
            { label: 'Permissions & Trust', slug: 'guides/permissions' },
            { label: 'Cross-Contract Calls', slug: 'guides/contract-calls' },
            { label: 'Testing Strategy', slug: 'guides/testing' },
            { label: 'Deployment', slug: 'guides/deployment' },
            { label: 'Error Handling', slug: 'guides/error-handling' },
            { label: 'Gas Optimization', slug: 'guides/gas-optimization' },
          ],
        },
        {
          label: 'Token Standards',
          items: [
            { label: 'NEP-17 (Fungible)', slug: 'standards/nep17' },
            { label: 'NEP-11 (NFT)', slug: 'standards/nep11' },
          ],
        },
        {
          label: 'Examples',
          items: [
            { label: 'Overview', slug: 'examples/overview' },
            { label: 'Hello World', slug: 'examples/hello-world', badge: 'Beginner' },
            { label: 'NEP-17 Token', slug: 'examples/nep17-token', badge: 'Beginner' },
            { label: 'NEP-11 NFT', slug: 'examples/nep11-nft', badge: 'Beginner' },
            { label: 'Escrow', slug: 'examples/escrow', badge: 'Intermediate' },
            { label: 'Multisig Wallet', slug: 'examples/multisig-wallet', badge: 'Intermediate' },
            { label: 'Crowdfunding', slug: 'examples/crowdfunding', badge: 'Intermediate' },
            { label: 'Timelock Vault', slug: 'examples/timelock-vault', badge: 'Intermediate' },
            { label: 'Staking Rewards', slug: 'examples/staking-rewards', badge: 'Intermediate' },
            { label: 'Oracle Consumer', slug: 'examples/oracle-consumer', badge: 'Intermediate' },
            { label: 'Governance DAO', slug: 'examples/governance-dao', badge: 'Advanced' },
            { label: 'NFT Marketplace', slug: 'examples/nft-marketplace', badge: 'Advanced' },
            { label: 'Flashloan Pool', slug: 'examples/flashloan-pool', badge: 'Advanced' },
          ],
        },
        {
          label: 'Cross-Chain',
          collapsed: true,
          items: [
            { label: 'Overview', slug: 'cross-chain/overview' },
            { label: 'Solana Compatibility', slug: 'cross-chain/solana' },
            { label: 'Move Support', slug: 'cross-chain/move' },
            { label: 'Syscall Mapping', slug: 'cross-chain/syscall-mapping' },
          ],
        },
        {
          label: 'Architecture',
          collapsed: true,
          items: [
            { label: 'Translator Internals', slug: 'architecture/translator' },
            { label: 'Opcode Mapping', slug: 'architecture/opcode-mapping' },
            { label: 'Memory Model', slug: 'architecture/memory' },
            { label: 'Table Handling', slug: 'architecture/tables' },
            { label: 'NEF Format', slug: 'architecture/nef-format' },
          ],
        },
        {
          label: 'Contributing',
          items: [
            { label: 'Dev Setup', slug: 'contributing/setup' },
            { label: 'Contribution Guide', slug: 'contributing/guide' },
            { label: 'Security Policy', slug: 'contributing/security' },
            { label: 'Changelog', slug: 'contributing/changelog' },
          ],
        },
      ],
    }),
  ],
});

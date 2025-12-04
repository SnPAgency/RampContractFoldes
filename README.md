# SnappiPay Ramp Smart Contract Suite

> Multi-chain on/off-ramp infrastructure for the SnappiPay payment network, spanning EVM, Solana, StarkNet, Soroban, Aptos, Supra, and Hedera.

## Table of Contents
- [Overview](#overview)
- [Repository Layout](#repository-layout)
- [Core Concepts](#core-concepts)
- [Cross-Chain Architecture](#cross-chain-architecture)
  - [Asset Lifecycle](#asset-lifecycle)
  - [Control Plane](#control-plane)
  - [Security Primitives](#security-primitives)
  - [Events & Observability](#events--observability)
- [Platform Implementations](#platform-implementations)
  - [Ethereum & Hedera (RampSol)](#ethereum--hedera-rampsol)
  - [Solana (ramp_solana)](#solana-ramp_solana)
  - [Stellar Soroban (ramp_stellar)](#stellar-soroban-ramp_stellar)
  - [StarkNet Cairo (ramp_stark)](#starknet-cairo-ramp_stark)
  - [Aptos Move (RampAptos)](#aptos-move-rampaptos)
  - [Supra Move (RampSup)](#supra-move-rampsup)
- [Build & Test Automation](#build--test-automation)
- [Deployment Playbooks](#deployment-playbooks)
- [Security & Quality](#security--quality)
- [Extensibility Roadmap](#extensibility-roadmap)
- [Appendix: Event Matrix](#appendix-event-matrix)

## Overview
The SnappiPay Ramp suite standardises fiat ↔ crypto flows across heterogeneous execution environments. Each on-chain implementation offers feature parity wherever the target chain allows:

- Canonical operations for onboarding (off-chain to on-chain) and settlement (on-chain to off-chain).
- Asset allow-list and fee governance managed by a designated controller.
- Revenue accounting routed to a protocol vault for periodic settlement.
- Upgradeable, pausable, and owner-gated control flows for safe operations.

Cross-chain parity is preserved through shared naming, event semantics, and off-chain orchestration that interprets per-chain events. All implementations live in this monorepo to simplify audits, CI, and shared tooling.

## Repository Layout
```
RampContractFoldes/
├── RampAptos/         # Aptos Move package (aptos CLI)
├── RampSol/           # EVM + Hedera Solidity package (Foundry)
├── RampSup/           # Supra Move package
├── ramp_solana/       # Solana native program (Rust + native SBF)
├── ramp_stark/        # StarkNet Cairo 1.0 contracts
├── ramp_stellar/      # Soroban contract (Rust, no_std)
├── build_all.sh       # Cross-project build orchestrator
├── install_deps.sh    # Toolchain bootstrap script
├── Makefile           # Aggregate build/test targets
└── README.md          # This document
```

## Core Concepts
- **On-ramp deposit**: Users transfer on-chain liquidity; contract emits metadata to map the payment rail (`OnrampMedium`, `Region`, optional payload) for off-chain reconciliation.
- **Off-ramp withdrawal**: Controller sends funds from protocol liquidity pools to satisfy fiat payouts while ensuring revenue separation.
- **Asset governance**: Maintain an allow-list with per-asset fee schedules, dynamic enable/disable, and revenue withdrawal to the vault.
- **Vault management**: Dedicated address per chain holding accumulated fees; changeable by the owner in a paused/upgraded-safe manner.
- **Revenue tracking**: Fees are tracked per asset and sent to the vault without co-mingling with customer balances.
- **Lifecycle safety**: Operations guarded by pausable + ownable patterns, reentrancy safety (EVM), and upgrade hooks where supported.

## Cross-Chain Architecture

### Asset Lifecycle
1. **Initialization**: Contracts bootstrap with controller/vault information and optional fee defaults.
2. **Asset onboarding**: Controllers whitelist ERC20/FA/PDA token identifiers, optionally seeding liquidity from funders.
3. **Customer deposits**: On-chain transfers + event metadata representing off-chain payment instructions.
4. **Settlement**: Controllers withdraw customer funds; revenues accumulate separately.
5. **Revenue extraction**: Periodic vault transfers keep protocol income segregated.

### Control Plane
- **Owner/Admin**: Single authority per chain; gated operations include pausing, upgrades, and fee/vault changes.
- **Pausing**: All user-facing state mutations respect paused state, enabling incident response.
- **Upgradeability**: Implemented via OpenZeppelin upgradeable libraries (EVM) & components (`UpgradeableComponent` in StarkNet, `UpgradeableInternal` in Soroban). Move/Solana rely on redeploy workflows.

### Security Primitives
- EVM: OpenZeppelin `PausableUpgradeable`, `OwnableUpgradeable`, transient reentrancy guard, strict math via `Math.mulDiv`.
- Solana: PDA-derived state, `mollusk_svm` integration for deterministic instruction testing, strict signer checks.
- Soroban: `stellar_access::ownable`, `stellar_contract_utils::pausable`, fee validation.
- StarkNet: OpenZeppelin Cairo components, typed storage maps, assert-based error handling.
- Move-based chains: `simple_map` allow-lists, `primary_fungible_store` custodial vaults, explicit error codes.

### Events & Observability
- Unified event names (`RampDeposit`, `RampWithdraw`, `AssetAllowedAdded`, `AssetFeeChanged`, etc.).
- Off-chain indexers consume events to trigger fiat settlement, using mediums & regions enumerations.
- Native platform telemetry: Solana & StarkNet expose typed events; Move modules emit event structs; Soroban topics align with asset/vault identifiers.

## Platform Implementations

### Ethereum & Hedera (`RampSol`)
- **Path**: `RampSol/src`
- **Language/Framework**: Solidity 0.8.30, Foundry toolchain, OpenZeppelin upgradeable stack.
- **Key contracts**: `RampContract.sol`, `IRampContract.sol`, `helpers/errors.sol`.
- **Storage**: Allowed asset map + list, per-asset `AssetInfo` structure, revenue counters, native asset symbol, upgradeable proxies.
- **Flows**: Deposits (ERC20, native, permit), withdrawals, asset/vault/fee governance, revenue extraction.
- **Deployment tooling**: Foundry scripts under `RampSol/script/` with environment-specific broadcast configs (Ethereum testnets, Hedera mirror). Shell helpers in `RampSol/deploy/` and `RampSol/run/`.
- **Testing**: `forge test`, plus scenario suites under `RampSol/test`. Native gas snapshots for performance.

### Solana (`ramp_solana`)
- **Path**: `ramp_solana/src`
- **Language/Framework**: Native Solana program (no Anchor), Borsh state serialization, `mollusk_svm` emulator for deterministic tests.
- **Key modules**: `instructions/*` (one module per instruction), `state.rs` (PDA layout & business logic), `processors.rs` dispatcher, `models` enumerations (`Medium`, `Region`).
- **Testing harness**: Extensive suites in `src/lib.rs` using Mollusk to provision SPL Token mints, associated token accounts, and run instruction pipelines end-to-end.
- **Build/Test**: `cargo build-bpf` / `cargo test`. `Mollusk` ensures deterministic simulation without a validator.

### Stellar Soroban (`ramp_stellar`)
- **Path**: `ramp_stellar/contracts/ramp-stellar/src`
- **Language/Framework**: Rust `no_std` Soroban contract using Stellar-provided macros.
- **Features**: Ownable + pausable macros, upgrade guard, asset fee tracking via `AssetInfo`, explicit medium/region enums compatible with EVM counterparts.
- **Events**: `emit_onramp_deposit_event`, `emit_asset_fee_percentage_changed`, `emit_vault_address_changed`, enabling off-chain ingestion from Soroban event streams.
- **Build/Test**: `cargo build --target wasm32-unknown-unknown`, `cargo test -p ramp-stellar`. `Makefile` inside directory for Soroban-specific flows.

### StarkNet Cairo (`ramp_stark`)
- **Path**: `ramp_stark/src`
- **Language/Framework**: Cairo 1.0 with OpenZeppelin components, built via Scarb.
- **Storage**: Typed maps for allowed tokens, fee schedule, revenue, vault; tracker map for enumerating allowed assets.
- **Interface**: Implements `IRampStack` exposing deposit/withdraw, asset management, revenue withdrawal, vault updates, and funding helper.
- **Build/Test**: `scarb build`, `snforge test` (tests under `ramp_stark/tests`). Deployment scripts in `ramp_stark/deploy`.

### Aptos Move (`RampAptos`)
- **Path**: `RampAptos/sources`
- **Language/Framework**: Move (Aptos), compiled via `aptos move`.
- **Modules**: `ramp.move`, `add_asset.move`, `test_token.move`. Generated docs under `RampAptos/doc/ramp.md`.
- **Highlights**: Global storage resource gating operations, `VaultStore` tracks fee + FA reference, events mirror the EVM schema, entry functions for initialize/add/remove asset, deposit/withdraw, fee management, contract toggling.
- **Testing**: `aptos move test`. Deployment scripts in `RampAptos/deploy`.

### Supra Move (`RampSup`)
- **Path**: `RampSup/sources`
- **Language/Framework**: Move (Supra network flavour).
- **Shared architecture**: Mirrors Aptos module with Supra-specific addresses/constants and CLI wrappers.
- **Docs**: Generated Move docs in `RampSup/doc/ramp.md`. Shell tooling under `RampSup/supra`.
- **Testing**: Use Supra CLI (`supra move tool test`) or cross-compile with aptos CLI if configured.

## Build & Test Automation
- **Top-level script**: `./install_deps.sh` installs Rust, Foundry, Aptos CLI, Scarb, Supra CLI (where available).
- **Unified builds**: `./build_all.sh` or `make all` compile & test every target sequentially, collecting exit statuses.
- **Selective targets**:
  - `make sol` → `forge build && forge test`
  - `make solana` → `cargo build-bpf && cargo test`
  - `make stark` → `scarb build && snforge test`
  - `make aptos` → `aptos move test`
  - `make supra` → Supra CLI Tooling
- CI integration via `.github/workflows/` builds affected packages, caches toolchains, and surfaces per-chain status checks.

## Deployment Playbooks
- **EVM/Hedera**: Foundry scripts (`01_Deploy.s.sol`, `03_DeployRampHedera.sol`) run via `forge script` with chain-specific broadcast settings in `RampSol/broadcast/`. Shell wrappers in `RampSol/deploy/`.
- **Solana**: Program keypairs stored under `ramp_solana/keys`, example client scripts in `ramp_solana/examples` covering initialize, mint, on/off-ramp operations.
- **StarkNet**: `ramp_stark/deploy/*.sh` encapsulate class declaration, upgrade, asset operations via `starkli` or `sncast`.
- **Aptos/Supra**: `deploy/` directories include `deploy.sh`, `upgrade.sh`, `run_script.sh` orchestrating module publishing and script execution.
- **Soroban**: `ramp_stellar/contracts/ramp-stellar/Makefile` provides build + test + deploy targets leveraging `soroban-cli`.

## Security & Quality
- **Static safety**: Heavy reliance on platform-specific audited libraries (OpenZeppelin for Solidity & Cairo, Stellar utils, Move std modules).
- **Operational safeguards**: Pauses, strict owner checks, numeric bounds on fees (`fee_percentage` guard rails).
- **Reentrancy & allowance safety**: EVM contract uses `ReentrancyGuardTransient` and explicit allowance checks; Solana/Soroban transfer flows verify allowances/balances prior to state mutation.
- **Comprehensive tests**: 
  - Solana program uses Mollusk SVM to emulate full instruction flows (initialize → asset ops → deposit/withdraw).
  - StarkNet tests under `ramp_stark/tests` cover deposit/withdraw, asset lifecycle.
  - Move unit tests validate map handling and error codes.
  - Solidity tests ensure event emission, fee accounting, and revert paths.
- **Upgrades**: Only callable when paused (Soroban) or by owner (EVM/StarkNet). Deployment scripts enforce version tracking.

## Extensibility Roadmap
- **New chain targets**: Add a directory mirroring existing structure, implement standard events & control plane, and wire into `build_all.sh` + CI.
- **Fee plugins**: Replace static percentages with dynamic fee strategies (volume-based tiers) behind upgradeable interfaces.
- **Cross-chain relayer**: Introduce unified off-chain relayer to consume events from all chains and trigger settlements with consistent SLAs.
- **Analytics**: Extend event matrix with Prometheus-friendly exporters or push data into common data warehouse.

## Appendix: Event Matrix

| Event | Solidity (`RampSol`) | Solana (`ramp_solana`) | Soroban (`ramp_stellar`) | StarkNet (`RampStark`) | Aptos (`RampAptos`) | Supra (`RampSup`) |
|-------|----------------------|------------------------|--------------------------|------------------------|---------------------|-------------------|
| Asset allowed | `AssetAllowedAdded(asset, funder, fee, initialBalance)` | `AssetAddedEvent` | `emit_asset_added` | `AssetAllowedAdded` | `AssetAddedEvent` | `AssetAddedEvent` |
| Asset removed | `AssetAllowedRemoved(asset, recipient, balance)` | `AssetRemovedEvent` | `emit_asset_removed` | `AssetAllowedRemoved` | `AssetRemovedEvent` | `AssetRemovedEvent` |
| Deposit | `RampDeposit(asset, sender, amountAfterFee, symbol, medium, region, data)` | `OffRampDepositInstruction` (event) | `emit_onramp_deposit_event` | `RampDeposit` | `RampDeposit` | `RampDeposit` |
| Withdrawal | `RampWithdraw(asset, recipient, amount)` | `OnRampWithdrawInstruction` | `emit_off_ramp_event` | `RampWithdraw` | `RampWithdraw` | `RampWithdraw` |
| Revenue | `AssetRevenueWithdrawn(asset, amount)` | N/A (ledger balance tracked) | `emit_asset_revenue_withdrawn` | `RevenueWithdrawn` | Vault balance tracked | Vault balance tracked |
| Vault change | `VaultChanged(oldVault, newVault)` | `SetOwnerInstruction` updates | `emit_vault_address_changed` | `VaultChanged` | `VaultChanged` | `VaultChanged` |
| Fee change | `setNewFeePercentage` (eventless setter) | `SetAssetFeeInstruction` | `emit_asset_fee_percentage_changed` | `AssetFeeChanged` | `AssetFeeChanged` | `AssetFeeChanged` |
| Pause/Resume | `PausableUpgradeable` events | Instruction-level checks | `pausable` events | `PausableEvent` | Global state toggle | Global state toggle |

---

For additional implementation details, inspect per-chain READMEs and generated docs, or run `make help` for available automation targets.

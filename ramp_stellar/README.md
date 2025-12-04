# Ramp Soroban Architecture

This package contains the Soroban (Stellar) implementation of the SnappiPay ramp. It makes heavy use of the Stellar smart contract macros for ownable, pausable, and upgradeable behaviour while keeping business logic aligned with the other chains.

## Repository Layout
```
ramp_stellar/
├── Cargo.toml
├── contracts/ramp-stellar/
│   ├── Cargo.toml
│   ├── Makefile
│   └── src/
│       ├── errors.rs
│       ├── events.rs
│       ├── lib.rs
│       └── test.rs
└── README.md
```

## Storage Model
- `RampContractState::VaultAddress` – Vault that receives protocol revenue.
- `RampContractState::MaxAssets` – Upper bound on concurrently tracked tokens.
- `RampContractState::AssetsInfo(Address)` – Maps asset addresses to `AssetInfo`.

`AssetInfo` tracks `is_added`, `asset_fee_percentage`, and `asset_revenue`. All structures are declared with `#[contracttype]`, allowing strongly typed reads/writes via `env.storage().instance()`.

## Contract Entry Points
### Initialisation
`__constructor(env, admin, vault_address, max_assets)` establishes the owner, sets the asset cap, and seeds the vault reference.

### Asset Governance
- `add_asset` – Validates fee bounds, consumes delegated liquidity using `token::Client::transfer_from`, records the asset, and emits `AssetAdded`.
- `remove_asset` – Transfers available liquidity (excluding revenue) to `balance_recipient`, resets storage, and emits `AssetRemoved`.
- `change_asset_fee_percentage` – Updates per-asset fee tiers, emitting `AssetFeePercentageChanged`.
- `withdraw_asset_revenue` – Sends accumulated protocol revenue to the vault and zeros the counter.

### Ownership & Vault Management
- `change_owner` – Reassigns controller privileges.
- `change_vault_address` – Stores the new vault and extends storage TTL to keep the entry alive.
- `get_vault_address` – Public view helper.

### On/Off-Ramp Flows
- `onramp_deposit` – Transfers tokens from the sender into the contract, calculates protocol fee (`asset_fee_percentage * amount / 100`), accrues revenue, and emits `OnRampDepositEvent`. Supports metadata (`OnrampMedium`, `Region`, arbitrary `Bytes` payload).
- `off_ramp_withdraw` – Confirms sufficient liquidity (excluding revenue), transfers tokens to the recipient, and emits `OffRampWithdrawEvent`.

### Native Safeguards
- Pausing/unpausing is provided via the `Pausable` implementation from `stellar_contract_utils`.
- Upgrades are mediated by `UpgradeableInternal::_require_auth`, which demands the contract be paused and the caller be the owner before code hashes can change.

## Events & Telemetry
`events.rs` centralises emission logic. Notable events:
- `AssetAddedEvent`, `AssetRemovedEvent`
- `AssetFeeChangedEvent`
- `RevenueWithdrawnEvent`
- `OnRampDepositEvent`, `OffRampWithdrawEvent`
- `VaultAddressChangedEvent`

Each event encodes topics to aid indexing (asset address, owner, vault, etc.) and includes business payloads for off-chain reconciliation.

## Error Handling
`errors.rs` exposes `RampContractError`, covering conditions such as:
- `InvalidFeePercentage`
- `AssetAlreadyExists` / `AssetNotFound`
- `AssetNotAllowed`
- `InsufficientFunds`
- `VaultAddressNotFound`

Functions return `Result<T, RampContractError>` so clients can map errors deterministically.

## Security Considerations
- `#[only_owner]` macros ensure only the controller can mutate configuration or withdraw revenue.
- `#[when_not_paused]` wraps state-changing functions; deposits/withdrawals halt when paused.
- Fee bounds (0–60 inclusive) protect users from erroneous configurations.
- Upgrade authentication requires pausing first; this pattern prevents hot upgrades while funds are moving.
- Liquidity withdrawals always exclude revenue to maintain accounting integrity.

## Testing Strategy
- `src/test.rs` uses Soroban’s test runner to simulate contract calls, asserting event payloads and state changes.
- Snapshot fixtures under `test_snapshots/` store canonical results, assisting external services in decoding emitted events.

## Deployment Workflow
1. Build the Wasm artefact: `make build` or `cargo build --target wasm32-unknown-unknown --release`.
2. Run the unit tests: `make test`.
3. Deploy with `soroban contract deploy --wasm target/wasm32-unknown-unknown/release/ramp_stellar.wasm`.
4. Invoke `__constructor` with admin, vault, and max asset values.
5. Onboard assets and configure fee tiers.
6. Monitor emitted events to drive fiat settlement and revenue reconciliation.

## Operational Checklist
- Rotate the owner with `change_owner` and pause/unpause surrounding upgrades.
- Periodically call `withdraw_asset_revenue` to sweep revenue into the vault.
- Use `change_vault_address` to update bank details while preserving TTL.
- Track the `MaxAssets` limit when onboarding new tokens; exceeding it will surface errors until old assets are removed.
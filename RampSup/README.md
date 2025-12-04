## RampSup Architecture

RampSup contains the Supra Move implementation of the SnappiPay ramp. It closely mirrors the Aptos module but targets the Supra framework APIs and coin primitives.

### Directory Layout
```
RampSup/
├── Move.toml                 # Package manifest
├── sources/
│   └── ramp.move             # Ramp module for Supra networks
├── doc/                      # Generated docs (optional)
├── supra/                    # Network configuration & workspace files
└── build/                    # Compiled artifacts and metadata
```

### Module Summary (`RampSup::ramp`)
Imports the Supra framework equivalents of the Aptos APIs:
- `supra_framework::object` for named objects and extend references.
- `supra_framework::fungible_asset` for FA custody.
- `supra_framework::coin` and `primary_fungible_store` for native coin management.
- `aptos_std::simple_map` for deterministic maps.

The module reuses the same error codes and storage layout as the Aptos version, ensuring feature parity across Move-based chains.

### Storage Resources
- **`GlobalStorage`** (`has key`): tracks activation flag, owner, FA vault map (`SimpleMap<Object<Metadata>, VaultStore>`), coin vault map (`SimpleMap<address, CoinVault>`), and a `global_extend_ref` for derived signers.
- **`VaultStore`** (`has store, drop`): wraps a `FungibleStore` object, stores the fee percentage, extend reference, and per-asset revenue counter.
- **`CoinVault`** (`has store, drop`): records stored amount, accumulated revenue, and fee tier for each coin type.

The named object is anchored by `RAMP_APTOS` to keep address derivation consistent with other deployments, simplifying cross-chain tooling.

### Events
Supra’s event system is used to publish operational telemetry:
- `AssetAddedEvent`, `AssetRemovedEvent`, `AssetFeeChanged`
- `CoinAddedEvent`, `CoinRemovedEvent`
- `RampDeposit`, `RampWithdraw` (for FAs)
- `ContractStateChangedEvent`, `OwnerChangedEvent`, `VaultChanged`

Coin-specific on/off-ramp events mirror the FA flows where applicable.

### Entry Functions
- `init_module(owner)` – Creates the named object, writes `GlobalStorage`, sets the owner, and prepares extend references.
- `add_asset`, `remove_asset` – Manage FA vault lifecycle, including creating/destroying stores and transferring balances.
- `set_fee` – Updates fee percentages per asset.
- `on_ramp_deposit`, `off_ramp_withdraw` – Handle FA deposits/withdrawals with fee accounting and revenue tracking.
- `add_coin`, `remove_coin`, `offramp_coin`, `onramp_coin` – Equivalent flows for native coins with generics over `CoinType`.
- `set_owner`, `set_contract_state` – Governance utilities.
- View helpers (`get_allowed_assets`, `is_asset_allowed`, etc.) expose read-only insights.

All mutating entry points `acquire` the necessary resources to satisfy Move's borrow rules.

### Revenue Accounting & Safeguards
- Revenue accrues in `VaultStore.asset_revenue` and `CoinVault.coin_revenue`; operational withdrawals verify balances excluding this revenue.
- Fee percentages are validated before being stored and used consistently during on/off-ramp operations.
- Ownership assertions (`signer::address_of(owner)`) protect privileged flows.
- `is_active` provides a kill switch for maintenance and is enforced in operational entry functions.

### Deployment Checklist
1. Publish the package using the Supra CLI or `scarb`-compatible tooling configured in `supra/move_workspace/`.
2. Execute `init_module` once to create the named object and global storage.
3. Add assets or coins, configure fees, and seed liquidity if required.
4. Use `supra` CLI helpers to run scripts for deposits/withdrawals as part of operational runbooks.
5. Monitor events to feed settlement and analytics pipelines.

### Operational Notes
- Keep the `global_extend_ref` secure; it allows the module to derive signers for store management.
- Regularly withdraw accumulated revenue from FA and coin vaults to treasury accounts.
- When rotating ownership, emit `OwnerChangedEvent` and confirm `GlobalStorage.owner` updates accordingly.
- Pause interactions via `set_contract_state` before performing upgrades or moving liquidity off-chain.


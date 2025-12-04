## RampAptos Architecture

The Aptos implementation of the SnappiPay ramp is authored in Move and packaged in this directory. It provides custodial vaulting for fungible assets (FAs) and native coins while mirroring the core workflows exposed on other chains.

### Repository Layout
```
RampAptos/
├── Move.toml                 # Package manifest
├── sources/
│   ├── ramp.move             # Main module
│   ├── add_asset.move        # Script helpers
│   └── test_token.move       # Test FA definitions
├── doc/                      # Generated Move documentation
├── deploy/                   # Shell helpers wrapping `aptos` CLI
└── build/                    # Compiled bytecode & metadata
```

### Module Overview
`module RampAptos::ramp` exposes all state and entry functions. Key imports include:
- `aptos_framework::object` for named object creation and `ExtendRef` management.
- `aptos_framework::fungible_asset` for FA stores.
- `aptos_framework::coin` and `primary_fungible_store` for native coin custody.
- `aptos_std::simple_map` for deterministic key/value storage.

### Permanent Storage
1. **`GlobalStorage` (resource, `has key`)**
   - `is_active`: Master switch gating state changes.
   - `owner`: Controller address.
   - `vault_address`: `SimpleMap<Object<Metadata>, VaultStore>` mapping FA metadata objects to their vaults.
   - `coin_vaults`: `SimpleMap<address, CoinVault>` recording per-coin configuration.
   - `global_extend_ref`: Enables derived signers for vault operations.

2. **`VaultStore` (resource)**
   - `store`: `Object<FungibleStore>` where FA balances live.
   - `asset_fee_percentage`: Current fee tier.
   - `vault_extend_ref`: Extend reference allowing the contract to operate on the store.
   - `asset_revenue`: Accumulated protocol earnings for the asset.

3. **`CoinVault` (resource)**
   - `store_amount`: Custodied coin balance.
   - `coin_revenue`: Revenue accumulated from coin flows.
   - `coin_fee_percentage`: Fee tier for the coin.

4. **`RampEventStore` (resource)**
   Holds event handles for every significant state change:
   - Asset/coin added & removed.
   - Deposits (`RampDeposit`, `CoinOfframpEvent`) and withdrawals (`RampWithdraw`, `CoinOnrampEvent`).
   - Governance updates (`OwnerChangedEvent`, `ContractStateChangedEvent`, `VaultChanged`, `AssetFeeChanged`).

The resources are stored under a named object address `object::create_object_address(&@RampAptos, RAMP_APTOS)`, giving the module a stable home regardless of deployment address.

### Errors & Guards
Custom error codes (constants `ENO_CONTRACT_STATE`, `ENOT_OWNER`, etc.) cover:
- Missing global state.
- Unauthorized caller.
- Asset/coin not registered.
- Fee invariants and insufficient balances.

Assertions with `error::permission_denied`, `error::invalid_argument`, and `error::not_found` enforce these boundaries.

### Entry Function Highlights
| Function | Purpose |
|----------|---------|
| `init_module` | Creates the global storage object, sets the owner, and initialises event handles. |
| `add_asset` / `remove_asset` | Manage FA vaults: create stores, set fees, deposit seed liquidity, remove and sweep balances. |
| `on_ramp_deposit` / `off_ramp_withdraw` | Customer on/off-ramp for FAs; withdraw ensures revenue is preserved before dispersing liquidity. |
| `set_fee` | Updates per-asset fee percentages. |
| `set_contract_state` | Pauses/resumes the protocol by toggling `is_active`. |
| `set_owner` | Transfers control to a new address. |
| `get_allowed_assets` / `is_asset_allowed` | View helpers for clients. |
| `add_coin`, `remove_coin`, `offramp_coin`, `onramp_coin` | Equivalent flows for native coin types (using generic `CoinType`). |

Each entry function `acquires` the resources it mutates (`GlobalStorage`, `RampEventStore`) to satisfy the Move borrow checker.

### Revenue Accounting
- For FAs, `VaultStore.asset_revenue` increments on deposits and is deducted from total balance when validating withdrawals.
- For coins, `CoinVault.coin_revenue` tracks protocol revenue separately from customer funds.
- Revenue sweep functionality can be composed by owners using the stored fees and extend references.

### Events & Integration
Events carry sufficient metadata for off-chain reconciliation:
- `RampDeposit` includes asset identifier, name, amount, sender, fiat medium/region, and user payload (`data`).
- `RampWithdraw` and `CoinOnrampEvent` capture on-ramp deliveries with recipient addresses.
- Governance and configuration changes emit dedicated events for monitoring systems.

### Access Control & Activation
- Ownership checks ensure only the controller can modify state (`add_asset`, `remove_asset`, `set_fee`, `set_owner`, etc.).
- `is_active` is inspected in operational entry points to block activity during maintenance or upgrades.
- Extend references (`vault_extend_ref`, `global_extend_ref`) restrict who can operate on underlying FAs, ensuring only module-generated signers perform sensitive actions.

### Deployment & Tooling
1. Initialise the module: `aptos move publish --package-dir .`.
2. Execute `init_module` once to create global storage.
3. Use scripts under `deploy/` (`deploy.sh`, `run_script.sh`) for consistent CLI invocations.
4. Reference generated documentation in `doc/ramp.md` for auto-generated signatures and struct layouts.
5. Run tests via `aptos move test` before publishing upgrades.

### Operational Checklist
- Keep a registry of allowed assets/coins via `get_allowed_assets` and the `coin_vaults` map.
- When changing fees or owners, monitor emitted events to update operational dashboards.
- Regularly withdraw accumulated FA and coin revenue to treasury accounts.
- Toggle `set_contract_state` to pause the ramp before performing major upgrades or migrations.


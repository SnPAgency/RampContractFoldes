## Ramp StarkNet Architecture

This package houses the StarkNet Cairo implementation of the SnappiPay ramp. It leverages OpenZeppelin Cairo components for ownable, pausable, and upgradeable behaviour while maintaining the shared on/off-ramp workflow.

### Directory Layout
```
ramp_stark/
├── Scarb.toml              # Package manifest
├── src/
│   ├── ramp.cairo          # Main contract module
│   ├── token.cairo         # Auxiliary ERC20 for tests
│   ├── lib.cairo           # Library exports
│   ├── interfaces.cairo    # Interface aggregator
│   ├── interfaces/         # IRampStack interface definition
│   └── errors.cairo        # Error selectors
├── tests/                  # `snforge` test suites
└── deploy/                 # Shell helpers wrapping `starkli`/`sncast`
```

### Core Contract (`src/ramp.cairo`)
The `RampStark` module is declared with `#[starknet::contract]` and composes several OpenZeppelin components:
- `OwnableComponent` – controller-only operations.
- `PausableComponent` – toggles deposit/withdraw flows.
- `UpgradeableComponent` – enables class hash upgrades under owner control.

`IERC20Dispatcher` and `IERC20MetadataDispatcher` provide token interfaces compatible with StarkNet ERC-20 contracts.

### Storage Layout
```cairo
struct Storage {
    allowed_tokens: Map<ContractAddress, bool>,
    allowed_tokens_tracker: u8,
    allowed_tokens_map: Map<u8, ContractAddress>,
    project_revenue_per_asset: Map<ContractAddress, u256>,
    fee_per_asset: Map<ContractAddress, u8>,
    vault_address: ContractAddress,
    asset_fee_percentage: Map<ContractAddress, u256>, // legacy compatibility
    pausable: PausableComponent::Storage,
    ownable: OwnableComponent::Storage,
    upgradeable: UpgradeableComponent::Storage,
}
```

Highlights:
- `allowed_tokens` + `allowed_tokens_map` maintain both membership and iteration order.
- `allowed_tokens_tracker` stores the current count, enabling O(1) append/remove.
- `project_revenue_per_asset` accumulates protocol revenue without commingling customer liquidity.
- `fee_per_asset` carries the current fee percentage for each token.
- `vault_address` is the destination for revenue withdrawals.

### External Interface (`IRampStack`)
Defined in `src/interfaces/ramp_interface.cairo`, the interface exposes:
- Asset governance: `add_allowed_asset`, `remove_allowed_asset`, `set_fee`, `set_new_vault`.
- Operations: `off_ramp_deposit`, `on_ramp_withdraw`, `fund_asset`, `withdraw_asset_revenue`.
- Queries: `get_allowed_assets`, `get_asset_revenue`, `get_asset_fee_percentage`, `vault`, `is_asset_allowed`.
- Enumerations: `OnrampMedium`, `Region` mirroring the global taxonomy.

### Key Workflows
1. **Initialisation (`constructor`)**  
   Sets the controller (`OwnableComponent::initializer`) and stores the vault address.

2. **Asset Onboarding (`add_allowed_asset`)**
   - Validates fee range (>0 and <5).
   - Pulls any pre-approved allowance from `funder`.
   - Appends the asset to the tracking map and marks it allowed.
   - Emits `AssetAllowedAdded`.

3. **Asset Removal (`remove_allowed_asset`)**
   - Ensures the asset is allowed.
  - Transfers remaining balance to `token_receiver`.
  - Swaps the last list entry into the removed slot to maintain a tight array.
  - Emits `AssetAllowedRemoved`.

4. **Deposits (`off_ramp_deposit`)**
   - Uses `IERC20Dispatcher.transfer_from` to collect tokens.
   - Computes fees as `(fee_per_asset * amount) / 100`.
   - Accumulates revenue and emits a `RampDeposit` event carrying medium, region, and metadata.

5. **Withdrawals (`on_ramp_withdraw`)**
   - Confirms the contract’s balance minus recorded revenue exceeds the withdrawal amount.
   - Transfers tokens to the recipient and emits `RampWithdraw`.

6. **Revenue Extraction (`withdraw_asset_revenue`)**
   - Transfers the accumulated revenue to the vault address.
   - Emits `RevenueWithdrawn` with the amount collected.

7. **Administration**
   - `set_new_vault` updates the vault address while guarding against no-op changes.
   - `set_fee` updates per-asset fees, ensuring the asset is allowed and the new fee differs from the old.
   - `fund_asset` allows the owner to push liquidity directly into the contract when required.

### Events
Declared directly in `ramp.cairo` as part of the contract enum:
- `AssetAllowedAdded`, `AssetAllowedRemoved`
- `RampDeposit`, `RampWithdraw`
- `VaultChanged`
- `AssetFeeChanged`
- `RevenueWithdrawn`
- Component events (`PausableEvent`, `OwnableEvent`, `UpgradeableEvent`)

These events supply downstream systems with full context (asset, funder, medium, region, fee changes).

### Security & Access Control
- Owner-guarded operations use `self.ownable.assert_only_owner()`.
- `pause`/`unpause` are exposed through the embedded pausable implementation and are owner-only.
- Upgrade paths (`UpgradeableImpl::upgrade`) require owner permission.
- Revenue separation ensures `on_ramp_withdraw` cannot spend revenue by checking `balance - revenue > amount`.
- All token transfers use dispatcher return values and assert success.

### Testing
- `tests/test_ramps.cairo` exercises asset lifecycle, deposit/withdraw flows, fee adjustments, and pause logic.
- `tests/test_contract.cairo` provides unit coverage for constructor and helper functions.
- Tests rely on `snforge` with mocks defined under `tests/` to simulate ERC-20 behaviour.

### Deployment Workflow
1. Build with `scarb build`.
2. Declare and deploy the class using `sncast` or the scripts under `deploy/`.
3. Run the initialisation script to set controller and vault addresses.
4. Add assets, configure fees, and fund liquidity.
5. Automate operations using the provided shell scripts (`add_asset.sh`, `deposit.sh`, etc.), which wrap `sncast` invocations.

### Operational Checklist
- Pause the contract before invoking `upgrade.sh` to align with `UpgradeableInternal` constraints.
- Re-run `snforge test` after modifying storage layout or component usage.
- Monitor `RampDeposit`/`RampWithdraw` events from StarkNet indexers for settlement workflows.
- Periodically sweep `RevenueWithdrawn` amounts to reconcile protocol earnings with the vault.


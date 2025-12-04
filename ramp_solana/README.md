## Ramp Solana Architecture

This package hosts the native Solana implementation of the SnappiPay ramp program. It is written against the low-level Solana SDK (no Anchor) so that instruction packing and state layout mirror the constraints of the on-chain runtime.

### Directory Layout
- `src/state.rs` – Borsh-serialised state structs (`RampState`, `AssetEntry`, `AssetInfo`).
- `src/instructions/` – One module per instruction, exposing strongly typed argument structs.
- `src/processors.rs` – Instruction dispatcher and enum that routes incoming payloads to handlers.
- `src/models/` – Shared enums (`Medium`, `Region`) for event metadata.
- `src/errors.rs` – Custom error catalog translated into `ProgramError::Custom`.
- `examples/` – Client-side reference flows (initialise, add asset, deposit, withdraw) using `solana-program-test` friendly APIs.
- `keys/` – Development keypairs used by scripts and tests.

### Accounts & State
`RampState` is stored in a program-derived account (PDA) seeded with `[b"ramp", <payer>]`. It contains:
- `owner` – Controller Pubkey authorised for governance actions.
- `is_active` – Safety switch; most mutating instructions early-return if the program is not active.
- `native_fee_percentage` / `native_revenue` – Fee tier and revenue accumulator for SOL deposits.
- `vault_address` – Destination for protocol revenue withdrawals.
- `asset_entries` – Fixed array (length 10) of `AssetEntry` structs, each holding an ERC-20 equivalent mint and its `AssetInfo` (`asset_fee_percentage`, `asset_revenue`).

All state is Borsh encoded; rent-exemption is enforced during initialisation using the calculated serialised length.

### Instruction Set
| Instruction | Handler | Purpose |
|-------------|---------|---------|
| `InitializeProgram` | `initialize_program::initialize_program` | Creates the PDA account, sets ownership, vault, native fee, and activates the ramp. |
| `SetActive` | `set_active::set_active` | Toggles `is_active` with owner auth. |
| `SetOwner` | `set_owner::set_owner` | Reassigns the controller. |
| `AddAssets` / `AddAssets2022` | `add_assets::*` | Onboards SPL Token or Token-2022 mints, creates ATA accounts, records fee tier, and optionally seeds liquidity via allowance transfers. |
| `RemoveAssets` | `remove_assets::remove_assets` | Removes an asset entry and sweeps remaining balance to the supplied recipient. |
| `SetAssetFee` | `set_asset_fee::set_asset_fee` | Updates per-asset fee percentage with bounds checking. |
| `SetNativeFeePercentage` | `set_native_fee_percentage::set_native_fee_percentage` | Adjusts native fee tier. |
| `OffRampDeposit` / `OffRampDepositToken22` | `off_ramp_deposit::*` | Transfers tokens from customer ATA into ramp ATA, computes fees, and accumulates revenue. |
| `OffRampDepositNative` | `off_ramp_deposit_native::off_ramp_deposit_native` | Accepts SOL via the program-owned account. |
| `OnRampWithdraw` / `OnRampWithdrawNative` | `onramp_withdraw::*` | Sends tokens/SOL to customers while ensuring revenue is not accidentally spent. |

Every handler shares the following patterns:
- Owner-signed or PDA-signed checks via expected signer/order in the account list.
- `RampState` is deserialised at the start, mutated with domain-specific logic, and serialised back before returning.
- Transfers use SPL Token program CPI (`transfer`, `transfer_from`) or system program CPI (`create_account`, `transfer`).

### Error Handling
`RampError` enumerates all failure modes (invalid fee, unauthorised, insufficient funds, etc.) and is converted to the canonical `ProgramError::Custom(u32)` codes. This keeps on-chain logic terse while allowing client tooling to map integer codes back to human-readable strings.

### Fee & Revenue Accounting
- `AssetInfo::add_revenue` saturates addition to defend against overflow.
- Native deposits call `RampState::update_native_revenue`.
- Withdraw flows subtract the outstanding revenue before transferring to ensure protocol earnings are preserved until an explicit withdrawal instruction is executed.

### Events & Off-Chain Metadata
While Solana does not emit EVM-style events, the program publishes rich instruction data for indexers:
- `OffRampDepositInstruction` includes `Region`, `Medium`, and an arbitrary `data` payload captured in `models::off_ramp_models`.
- Test snapshots under `contracts/ramp-stellar/test_snapshots` provide canonical binary encodings for every path, helping external services decode instruction logs safely.

### Security Considerations
- **Authorisation**: Critical instructions verify the signer matches `owner`. PDAs are derived with a bump provided during init to avoid collisions.
- **Program Activity**: `is_active` prevents operations while paused or before initialisation completes.
- **Fee Bounds**: Asset fees are validated within each instruction (`fee_percentage > 100` errors) to avoid unintentional >100% fees.
- **Account Validation**: Instruction modules check account ownership, rent exemption, and associated token account derivations to protect against malicious account substitution.

### Testing Strategy
- `src/lib.rs` houses integration tests using the `mollusk_svm` lightweight validator. Scenarios cover all instructions: adding assets, fee updates, deposits, withdrawals, pausing, and owner change flows.
- Example clients in `examples/` match the instruction APIs and can be used as blueprints for actual Solana client integrations.

### Deployment Notes
1. Derive the ramp PDA for each admin: `Pubkey::find_program_address(&[b"ramp", admin], program_id)`.
2. Run the `initialize_program` example or invoke the instruction from a client to bootstrap the PDA account with sufficient rent.
3. For each SPL token:
   - Create and fund the admin’s ATA with liquidity.
   - Invoke `AddAssets` providing fee tiers and the ramp PDA as the program-owned recipient.
4. Configure fees and activation as needed.
5. Monitor deposits/withdrawals by decoding instruction data or leveraging generated snapshots for binary schema references.

### Operational Checklist
- Rotate the `owner` key by executing `SetOwner` with the current owner signer.
- When pausing, call `SetActive { is_active: false }` to block deposits/withdrawals.
- Periodically withdraw protocol revenue by sending native/asset withdrawals to the vault address.
- Keep track of the fixed-size `asset_entries` array (max 10 assets). If more slots are required, a future upgrade must expand the array length in `RampState` and redeploy with a compatible layout.


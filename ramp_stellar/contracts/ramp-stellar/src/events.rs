use soroban_sdk::{contracttype, Address, Bytes, Env, Symbol};

use crate::{OnrampMedium, Region};

// ######### EVENTS #########



#[contracttype]
#[derive(Debug, Clone)]
pub struct AssetAddedEvent{
    pub funder: Address,
    pub fee_percentage: i128,
    pub initial_balance: i128,
} 

/// Emits an event indicating that a new asset has been added to the contract
/// 
/// # Arguments
/// 
/// * `env` - The environment in which the event is emitted
/// * `asset` - The address of the asset that was added
/// * `fee_percentage` - The fee percentage for the asset
/// 
/// # Events
/// 
/// * topics - [ASSET_ADDED, asset]
/// * data - [AssetAddedEvent]
pub fn emit_asset_added(env: &Env, asset: Address, funder: Address, fee_percentage: i128, initial_balance: i128) {
    let topics = (Symbol::new(env, "ASSET_ADDED"), asset);

    env.events().publish(topics, AssetAddedEvent {
        funder: funder,
        fee_percentage: fee_percentage,
        initial_balance: initial_balance,
    });
}


#[contracttype]
#[derive(Debug)]
pub struct AssetRemovedEvent {
    pub balance_receipient: Address,
    pub amount: i128
}
/// Emits an event indicating that an asset has been removed from the contract
/// 
/// # Arguments
/// 
/// * `env` - The environment in which the event is emitted
/// * `asset` - The address of the asset that was removed
/// * `balance_recipient` - The address of the recipient of the asset balance
/// * `amount` - The amount of the asset that was removed
/// 
/// # Events
/// 
/// * topics - [ASSET_REMOVED, asset]
/// * data - [balance_recipient, amount]
pub fn emit_asset_removed(env: &Env, asset: Address, balance_recipient: Address, amount: i128) {
    let topics = (Symbol::new(env, "ASSET_REMOVED"), asset);

    env.events().publish(topics, AssetRemovedEvent {
        balance_receipient: balance_recipient,
        amount: amount
    });
}

/// Emits an event indicating that the vault address has been changed
/// 
/// # Arguments
/// 
/// * `env` - The environment in which the event is emitted
/// * `old_vault_address` - The old vault address
/// * `new_vault_address` - The new vault address
/// 
/// # Events
/// 
/// * topics - [VAULT_ADDRESS_CHANGED, old_vault_address]
/// * data - [new_vault_address]
pub fn emit_vault_address_changed(env: &Env, old_vault_address: Address, new_vault_address: Address) {
    let topics = (Symbol::new(env, "VAULT_ADDRESS_CHANGED"), old_vault_address);

    env.events().publish(topics, new_vault_address);
}


#[contracttype]
#[derive(Debug)]
struct AssetFeeChangedEvent {
    old_fee: i128,
    new_fee: i128,
}
/// Emits an event indicating that the asset fee percentage has been changed
/// 
/// # Arguments
/// 
/// * `env` - The environment in which the event is emitted
/// * `asset` - The address of the asset that was changed
/// * `old_fee_percentage` - The old fee percentage
/// * `new_fee_percentage` - The new fee percentage
/// 
/// # Events
/// 
/// * topics - [ASSET_FEE_PERCENTAGE_CHANGED, asset]
/// * data - [AssetFeeChangedEvent]
pub fn emit_asset_fee_percentage_changed(env: &Env, asset: Address, old_fee_percentage: i128, new_fee_percentage: i128) {
    let topics = (Symbol::new(env, "ASSET_FEE_PERCENTAGE_CHANGED"), asset);

    env.events().publish(topics, AssetFeeChangedEvent {
        old_fee: old_fee_percentage,
        new_fee: new_fee_percentage
    });
}

#[contracttype]
#[derive(Debug)]
struct RevenueWithdrawnEvent {
    recipient: Address,
    amount: i128
}

/// Emits an event indicating that the asset revenue has been changed
/// 
/// # Arguments
/// 
/// * `env` - The environment in which the event is emitted
/// * `asset` - The address of the asset that was changed
/// * `old_revenue` - The old revenue
/// * `new_revenue` - The new revenue
/// 
/// # Events
/// 
/// * topics - [ASSET_REVENUE_CHANGED, asset]
/// * data - [RevenueWithdrawnEvent]
pub fn emit_asset_revenue_withdrawn(env: &Env, asset: Address, receiver: Address, revenue: i128) {
    let topics = (Symbol::new(env, "ASSET_REVENUE_CHANGED"), asset);

    env.events().publish(topics, RevenueWithdrawnEvent {
        recipient: receiver,
        amount: revenue
    });
}


#[contracttype]
#[derive(Debug)]
struct OnRampDepositEvent {
    amount: i128,
    medium: OnrampMedium,
    region: Region,
    data: Bytes
}

/// Emits an event indicating that an onramp has occured
/// 
/// # Arguments
/// 
/// * `env` - The environment in which the function gets run
/// * `asset` - The address of the asset whose fee is getting changed
/// * `amount` - amount deposited
/// * `sender` - The user making the deposit
/// * `medium` - The medium to recieve fiat
/// * `region` - The region where the transaction originated
/// * `data` - User data
/// 
/// # Event emitted
/// 
/// * topics - [ONRAMP, asset, sender]
/// * data - [OnRampDepositEvent]
///
pub fn emit_onramp_deposit_event(
    env: &Env,
    asset: Address,
    sender: Address,
    amount: i128,
    medium: OnrampMedium,
    region: Region,
    data: Bytes
) {
    let topics = (Symbol::new(env, "ONRAMP"), asset, sender);

    env.events().publish(topics, OnRampDepositEvent {
        amount: amount,
        region: region,
        medium: medium,
        data: data
    });
}

#[contracttype]
#[derive(Debug)]
struct OffRampWithdrawEvent {
    amount: i128,
}

/// Emits an event indicating that an offramp has occured
/// 
/// # Arguments
/// 
/// * `env` - The environment in which the function gets run
/// * `recipient` - The user making the deposit
/// * `asset` - The address of the asset whose fee is getting changed
/// * `amount` - amount deposited
/// 
/// # Event emitted
/// 
/// * topics - [OFFRAMP, asset, recipient]
/// * data - [OffRampWithdrawEvent]
///
pub fn emit_off_ramp_event(
    env: &Env,
    asset: Address,
    recipient: Address,
    amount: i128
) {
    let topics = (Symbol::new(env, "OFFRAMP"), asset, recipient);
    env.events().publish(topics, OffRampWithdrawEvent {
        amount: amount
    });

}
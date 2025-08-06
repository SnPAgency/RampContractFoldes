use soroban_sdk::{Env, Address, Symbol};

// ######### EVENTS #########

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
/// * data - [fee_percentage]
pub fn emit_asset_added(env: &Env, asset: Address, fee_percentage: u128) {
    let topics = (Symbol::new(env, "ASSET_ADDED"), asset);

    env.events().publish(topics, fee_percentage);
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
pub fn emit_asset_removed(env: &Env, asset: Address, balance_recipient: Address, amount: u128) {
    let topics = (Symbol::new(env, "ASSET_REMOVED"), asset);

    env.events().publish(topics, (balance_recipient, amount));
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
/// * data - [old_fee_percentage, new_fee_percentage]
pub fn emit_asset_fee_percentage_changed(env: &Env, asset: Address, old_fee_percentage: u128, new_fee_percentage: u128) {
    let topics = (Symbol::new(env, "ASSET_FEE_PERCENTAGE_CHANGED"), asset);

    env.events().publish(topics, (old_fee_percentage, new_fee_percentage));
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
/// * data - [old_revenue, new_revenue]
pub fn emit_asset_revenue_changed(env: &Env, asset: Address, old_revenue: u128, new_revenue: u128) {
    let topics = (Symbol::new(env, "ASSET_REVENUE_CHANGED"), asset);

    env.events().publish(topics, (old_revenue, new_revenue));
}

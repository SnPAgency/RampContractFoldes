#![no_std]
use soroban_sdk::{contracttype, contract, contractimpl, Env, String, Address};
use stellar_access::ownable::{self as ownable, Ownable};
use stellar_contract_utils::pausable::{self as pausable, Pausable};
use stellar_contract_utils::upgradeable::UpgradeableInternal;
use stellar_macros::{default_impl, only_owner, when_not_paused, when_paused, Upgradeable};

mod events;
mod errors;

use events::*;
use errors::RampContractError;

/**
 * @title RampContract
 * @dev This contract is an implementation of the Ramp protocol.
 * It will include functions and state variables related to the Ramp protocol.
 * The Ramp protocol is designed to facilitate the exchange of assets in a decentralized manner.
 * This contract will be expanded in future iterations to include the full functionality of the Ramp protocol.
 * The Ramp protocol aims to provide a secure and efficient way to exchange assets with minimal reliance on
 * centralized intermediaries.
 */


/**
 * @dev Contract state Variables
 */

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RampContractState {
    // Key for the vault address of the contract
    VaultAddress,
    // Key for the max assets of the contract
    MaxAssets,
    // Key for the assets entries of the contract
    AssetsEntries,
}


/**
 * @dev Contract DataStructures
 */

// information about an asset used
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
struct AssetEntry {
    asset: Address,
    info: AssetInfo,

}

//additional information about the asset
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
struct AssetInfo {
    asset_fee_percentage: u128,
    asset_revenue: u128,
}


#[contract]
pub struct RampContract;

#[contractimpl]
impl RampContract {
    pub fn __constructor(env: Env, admin: Address, max_assets: u32) {
        ownable::set_owner(&env, &admin);
        env.storage().instance().set(&RampContractState::MaxAssets, &max_assets);
    }

    #[only_owner]
    #[when_not_paused]
    pub fn add_asset(env: &Env, asset: Address, fee_percentage: u128) {
        emit_asset_added(env, asset, fee_percentage);
    }

    #[only_owner]
    #[when_not_paused]
    pub fn remove_asset(env: &Env, asset: Address, balance_recipient: Address) {
        emit_asset_removed(env, asset, balance_recipient, 0);
    }

    #[only_owner]
    #[when_not_paused]
    pub fn change_owner(env: &Env, new_owner: Address) {
        ownable::set_owner(env, &new_owner);
    }

    //This function changes the vault address of the contract
    #[only_owner]
    #[when_not_paused]
    pub fn change_vault_address(env: &Env, new_vault_address: Address) {

        let vault_address_key = RampContractState::VaultAddress;
        let mut current_vault = env
        .storage()
        .instance()
        .get(&vault_address_key)
        .unwrap_or(Address::from_string(&String::from_str(&env, "0x0000000000000000000000000000000000000000")));

        let old_vault_address = current_vault.clone();
        current_vault = new_vault_address.clone();
        env.storage().instance().set(&vault_address_key, &current_vault);
        env.storage().instance().extend_ttl(50, 100);

        emit_vault_address_changed(env, old_vault_address, new_vault_address);
    }

    #[only_owner]
    #[when_not_paused]
    pub fn change_asset_revenue(env: &Env, asset: Address, new_revenue: u128) {
        
        emit_asset_revenue_changed(env, asset,0, new_revenue);
    }

    #[only_owner]
    #[when_not_paused]
    pub fn change_asset_fee_percentage(env: &Env, asset: Address, new_fee_percentage: u128) {
        emit_asset_fee_percentage_changed(env, asset, 0, new_fee_percentage);
    }
}
//Ownable implimatation for the contract    
#[default_impl]
#[contractimpl]
impl Ownable for RampContract {
    
}

//Pausable implimatation for the contract
#[contractimpl]
impl Pausable for RampContract {
    fn paused(e: &Env) -> bool {
        pausable::paused(e)
    }

    #[only_owner]
    fn pause(e: &Env, _caller: Address) {
        pausable::pause(e);
    }

    #[only_owner]
    fn unpause(e: &Env, _caller: Address) {
        pausable::unpause(e);
    }
    
}


//Upgradeable implimatation for the contract
impl UpgradeableInternal for RampContract {
    #[when_paused]
    fn _require_auth(env: &Env, _operator: &Address) {
        ownable::enforce_owner_auth(env);
    }
}


mod test;

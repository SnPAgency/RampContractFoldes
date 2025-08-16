#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, panic_with_error, token, Address, Bytes, Env};
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
    AssetsInfo(Address),
}

// Onramp Medium
// Defines where ofchain funds should be sent
//ie medium: Safaricom, Airtel, and PayStack
#[contracttype]
#[derive(Debug, Clone, PartialEq, Eq)]
enum OnrampMedium {
    Primary,
    Secondary,
    Tertiary
}

// Region
#[contracttype]
#[derive(Debug, Clone, PartialEq, Eq)]
enum Region {
    KEN,
    RWN,
    NGA,
    SA,
    EGY,
    GHN
}

/**
 * @dev Contract DataStructures
 */

//additional information about the asset
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
struct AssetInfo {
    is_added: bool,
    asset_fee_percentage: i128,
    asset_revenue: i128,
}

impl Default for AssetInfo {
    fn default() -> Self {
        AssetInfo {
            is_added: false,
            asset_fee_percentage: 0,
            asset_revenue: 0
        }
    }
}

#[contract]
pub struct RampContract;

#[contractimpl]
impl RampContract {
    pub fn __constructor(env: Env, admin: Address, vault_address: Address, max_assets: u32) {
        ownable::set_owner(&env, &admin);
        env.storage().instance().set(&RampContractState::MaxAssets, &max_assets);
        env.storage().instance().set(&RampContractState::VaultAddress, &vault_address);
    }

    /// function add_asset 
    /// adds a new asset to the contract
    /// 
    /// # Arguments
    /// 
    /// * `env` - The environment in which the function gets run
    /// * `asset` - The address of the asset that was added
    /// * `funder` - The address providing intial asset liquidity
    /// * `fee_percentage` - The old fee percentage
    /// 
    /// # Event emitted
    /// 
    /// * topics - [ASSET_ADDED, asset]
    /// * data - [AssetAddedEvent]
    #[only_owner]
    #[when_not_paused]
    pub fn add_asset(env: &Env, asset: Address, funder: Address, fee_percentage: i128) -> Result<(), RampContractError> {
        if fee_percentage < 0 || fee_percentage > 60 {
            return Err(RampContractError::InvalidFeePercentage);
        }
        let asset_key = RampContractState::AssetsInfo(asset.clone());

        let mut current_asset_info: AssetInfo = env.storage().instance().get(&asset_key).unwrap_or_default();



        if !current_asset_info.is_added {

            let token = token::Client::new(env, &asset);
            let current_address =  env.current_contract_address();
            let initial_balance = token.allowance(&funder, &current_address);

            if initial_balance > 0 {
                token.transfer_from(&current_address, &funder, &current_address, &initial_balance);
            }

            current_asset_info.is_added = true;
            current_asset_info.asset_fee_percentage = fee_percentage;

            env.storage().instance().set(&asset_key, &current_asset_info);

            emit_asset_added(env, asset, funder, fee_percentage, initial_balance);
            Ok(())
        } else {
            Err(RampContractError::AssetAlreadyExists)
        }
    }

    /// function remove_asset 
    /// removes an existing asset to the contract
    /// 
    /// # Arguments
    /// 
    /// * `env` - The environment in which the function gets run
    /// * `asset` - The address of the asset that gets removed
    /// * `funder` - The address reciving current asset liquidity
    /// 
    /// # Event emitted
    /// 
    /// * topics - [ASSET_REMOVED, asset]
    /// * data - [AssetRemovedEvent]
    #[only_owner]
    #[when_not_paused]
    pub fn remove_asset(env: &Env, asset: Address, balance_recipient: Address) -> Result<(), RampContractError> {
        let asset_key = RampContractState::AssetsInfo(asset.clone());

        let asset_info: AssetInfo = env.storage().instance().get(&asset_key).unwrap_or_default();

        if asset_info.is_added {
            let token = token::Client::new(env, &asset);

            let balance = token.balance(&env.current_contract_address()) - asset_info.asset_revenue;

            if balance > 0 {
                token.transfer(&env.current_contract_address(), &balance_recipient, &balance);
            }
            env.storage().instance().set(&asset_key, &AssetInfo::default());
            emit_asset_removed(env, asset, balance_recipient, balance);
            Ok(())
        } else {
            Err(RampContractError::AssetNotFound)
        }
    }

    #[only_owner]
    #[when_not_paused]
    pub fn change_owner(env: &Env, new_owner: Address) {
        ownable::set_owner(env, &new_owner);
    }


    /// function change_vault_address 
    /// changesthe current vault address
    /// 
    /// # Arguments
    /// 
    /// * `env` - The environment in which the function gets run
    /// * `new_vault_address` - The new vault address
    /// 
    /// # Event emitted
    /// 
    /// * topics - [VAULT_ADDRESS_CHANGED, asset]
    /// * data - [old_vault_address, new_vault_address]
    #[only_owner]
    #[when_not_paused]
    pub fn change_vault_address(env: &Env, new_vault_address: Address) {
        let vault_address_key = RampContractState::VaultAddress;
        let mut current_vault: Address = env
        .storage()
        .instance()
        .get(&vault_address_key).unwrap();

        let old_vault_address = current_vault.clone();
        current_vault = new_vault_address.clone();
        env.storage().instance().set(&vault_address_key, &current_vault);
        env.storage().instance().extend_ttl(50, 100);

        emit_vault_address_changed(env, old_vault_address, new_vault_address);
    }

    pub fn get_vault_address(env: &Env) -> Address {
        let vault_address_key = RampContractState::VaultAddress;
        env
        .storage()
        .instance()
        .get(&vault_address_key)
        .unwrap_or_else(|| panic_with_error!(env, RampContractError::VaultAddressNotFound))
    }

    pub fn get_asset_fee_percentage(env: &Env, asset: Address) -> i128 {
        let asset_key = RampContractState::AssetsInfo(asset.clone());
        let asset_info: AssetInfo = env.storage().instance().get(&asset_key).unwrap_or_default();
        asset_info.asset_fee_percentage
    }

    pub fn get_asset_revenue(env: &Env, asset: Address) -> i128 {
        let asset_key = RampContractState::AssetsInfo(asset.clone());
        let asset_info: AssetInfo = env.storage().instance().get(&asset_key).unwrap_or_default();
        asset_info.asset_revenue
    }

    /// function withdraw_asset_revenue 
    /// sends an asset's generated revenue to the vault
    /// 
    /// # Arguments
    /// 
    /// * `env` - The environment in which the function gets run
    /// * `asset` - The address of the asset whose revenue is withdrawn
    /// 
    /// # Event emitted
    /// 
    /// * topics - [ASSET_FEE_PERCENTAGE_CHANGED, asset]
    /// * data - [RevenueWithdrawnEvent]
    #[only_owner]
    #[when_not_paused]
    pub fn withdraw_asset_revenue(env: &Env, asset: Address) -> Result<(), RampContractError> {
        let asset_key = RampContractState::AssetsInfo(asset.clone());

        let mut current_asset_info: AssetInfo = env.storage().instance().get(&asset_key).unwrap_or_default();

        if current_asset_info.is_added {
            let vault_address_key = RampContractState::VaultAddress;

            let current_vault = env
                .storage()
                .instance()
                .get(&vault_address_key)
                .unwrap();

            let token = token::Client::new(env, &asset);

            let balance = token.balance(&env.current_contract_address());
            let current_revenue = current_asset_info.asset_revenue;
            if balance > current_revenue {
                token.transfer(&env.current_contract_address(), &current_vault, &current_asset_info.asset_fee_percentage);
                current_asset_info.asset_fee_percentage = 0;
                env.storage().instance().set(&asset_key, &current_asset_info);
                emit_asset_revenue_withdrawn(env, asset, current_vault, current_revenue);
                Ok(())
            } else {
                Err(RampContractError::InsufficientFunds)
            }
        }
        else {
            Err(RampContractError::AssetNotFound)
        }
    }

    /// function change_asset_fee_percentage 
    /// changes the fee percentage charged on the asset
    /// 
    /// # Arguments
    /// 
    /// * `env` - The environment in which the function gets run
    /// * `asset` - The address of the asset whose fee is getting changed
    /// * `new_fee_percentage` - the new fee on the asset
    /// 
    /// # Event emitted
    /// 
    /// * topics - [ASSET_FEE_PERCENTAGE_CHANGED, asset]
    /// * data - [AssetFeeChangedEvent]
    #[only_owner]
    #[when_not_paused]
    pub fn change_asset_fee_percentage(env: &Env, asset: Address, new_fee_percentage: i128) -> Result<(), RampContractError> {
        let asset_key = RampContractState::AssetsInfo(asset.clone());

        let mut current_asset_info: AssetInfo = env.storage().instance().get(&asset_key).unwrap_or_default();
        if new_fee_percentage < 0 || new_fee_percentage > 60 {
            return Err(RampContractError::InvalidFeePercentage);
        }
        if current_asset_info.is_added {
            let old_fee = current_asset_info.asset_fee_percentage;
            current_asset_info.asset_fee_percentage = new_fee_percentage;
            env.storage().instance().set(&asset_key, &current_asset_info);
            emit_asset_fee_percentage_changed(env, asset, old_fee, new_fee_percentage);
            Ok(())
        } else {
            Err(RampContractError::AssetNotFound)
        }
    }

    /// function onramp_deposit 
    /// deposits funds into the contract from the sender 
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
    #[when_not_paused]
    pub fn onramp_deposit(
        env: &Env,
        asset: Address,
        amount: i128,
        sender: Address,
        medium: OnrampMedium,
        region: Region,
        data: Bytes
    ) -> Result<(), RampContractError>{
        let asset_key = RampContractState::AssetsInfo(asset.clone());

        let mut current_asset_info: AssetInfo = env.storage().instance().get(&asset_key).unwrap_or_default();

        if !current_asset_info.is_added {
            Err(RampContractError::AssetNotAllowed)
        } else {
            let token = token::Client::new(env, &asset);
            let current_address =  env.current_contract_address();

            let fee = (current_asset_info.asset_fee_percentage * amount) / 100;

            let amount_min_fee = amount - fee;
            token.transfer_from(&current_address, &sender, &current_address, &amount);

            current_asset_info.asset_revenue += fee;

            env.storage().instance().set(&asset_key, &current_asset_info);
            emit_onramp_deposit_event(env, asset, sender, amount_min_fee, medium, region, data);
            Ok(())
        }
    }

    /// function off_ramp_withdraw
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
    #[when_not_paused]
    pub fn off_ramp_withdraw(
        env: &Env,
        asset: Address,
        recipient: Address,
        amount: i128
    ) -> Result<(), RampContractError> {
        let asset_key = RampContractState::AssetsInfo(asset.clone());

        let current_asset_info: AssetInfo = env.storage().instance().get(&asset_key).unwrap_or_default();
        if !current_asset_info.is_added {
            Err(RampContractError::AssetNotAllowed)
        } else {
            let token = token::Client::new(env, &asset);
            let current_address =  env.current_contract_address();

            let token_balance = token.balance(&current_address);

            if !(token_balance - current_asset_info.asset_revenue) > amount {
                Err(RampContractError::InsufficientFunds)
            }else {
                token.transfer(&current_address, &recipient, &amount);
                emit_off_ramp_event(env, asset, recipient, amount);
                Ok(())
            }
        }

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
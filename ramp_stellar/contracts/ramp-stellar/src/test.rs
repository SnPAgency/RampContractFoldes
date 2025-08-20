#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Events}, Address, Env, IntoVal, Symbol};
use soroban_sdk::vec;

mod ramp_token {
    // Compatible with OpenZeppelin Stellar Soroban Contracts ^0.4.1
    use soroban_sdk::{Address, contract, contractimpl, Env, String};
    use stellar_access::ownable::{self as ownable, Ownable};
    use stellar_macros::{default_impl, only_owner};
    use stellar_tokens::fungible::{Base, burnable::FungibleBurnable, FungibleToken};

    #[contract]
    pub struct RampToken;

    #[contractimpl]
    impl RampToken {
        pub fn __constructor(e: &Env, owner: Address) {
            Base::set_metadata(e, 18, String::from_str(e, "RampToken"), String::from_str(e, "RAMPT"));
            ownable::set_owner(e, &owner);
        }

        #[only_owner]
        pub fn mint(e: &Env, account: Address, amount: i128) {
            Base::mint(e, &account, amount);
        }
    }

    #[default_impl]
    #[contractimpl]
    impl FungibleToken for RampToken {
        type ContractType = Base;

    }

    //
    // Extensions
    //

    #[default_impl]
    #[contractimpl]
    impl FungibleBurnable for RampToken {}

    //
    // Utils
    //

    #[default_impl]
    #[contractimpl]
    impl Ownable for RampToken {}
}

#[test]
fn test_add_asset() {
    let env = Env::default();
    env.mock_all_auths();
    let test_admin = Address::generate(&env);
    let vault_address = Address::generate(&env);
    let contract_id = env.register(RampContract, (test_admin.clone(), vault_address, 10u32));
    let token_id = env.register(ramp_token::RampToken, (test_admin.clone(),));
    let client = RampContractClient::new(&env, &contract_id);
    let token_client = ramp_token::RampTokenClient::new(&env, &token_id);

    token_client.mint(&test_admin, &10000i128);

    token_client.approve(&test_admin, &contract_id, &1000i128, &(env.ledger().sequence() + 100));
    
    client.add_asset( &token_id, &test_admin, &1i128);
    let topics = (Symbol::new(&env, "ASSET_ADDED"), token_id.clone()).into_val(&env);
    let data = AssetAddedEvent {
        funder: test_admin.clone(),
        fee_percentage: 1i128,
        initial_balance: 1000i128,
    }.into_val(&env);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                token_id.clone(),
                (Symbol::new(&env, "transfer"), test_admin.clone(), contract_id.clone()).into_val(&env),
                1000i128.into_val(&env),
            ),
            (   
                contract_id.clone(),
                topics,
                data,
            ),
        ]
    );
    assert_eq!(token_client.balance(&contract_id), 1000i128);
}

#[test]
fn test_remove_asset() {
    let env = Env::default();
    env.mock_all_auths();
    let test_admin = Address::generate(&env);
    let vault_address = Address::generate(&env);
    let contract_id = env.register(RampContract, (test_admin.clone(), vault_address, 10u32));
    let token_id = env.register(ramp_token::RampToken, (test_admin.clone(),));
    let client = RampContractClient::new(&env, &contract_id);
    let token_client = ramp_token::RampTokenClient::new(&env, &token_id);

    token_client.mint(&test_admin, &10000i128);

    token_client.approve(&test_admin, &contract_id, &1000i128, &(env.ledger().sequence() + 100));
    
    client.add_asset( &token_id, &test_admin, &1i128);

    assert_eq!(token_client.balance(&contract_id), 1000i128);

    client.remove_asset(&token_id, &test_admin);

    let topics = (Symbol::new(&env, "ASSET_REMOVED"), token_id.clone()).into_val(&env);
    let data = AssetRemovedEvent {
        balance_receipient: test_admin.clone(),
        amount: 1000i128,
    }.into_val(&env);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                token_id.clone(),
                (Symbol::new(&env, "transfer"), contract_id.clone(), test_admin.clone()).into_val(&env),
                1000i128.into_val(&env),
            ),
            (   
                contract_id.clone(),
                topics,
                data,
            ),
        ]
    );
    assert_eq!(token_client.balance(&contract_id), 0i128);
}

#[test]
fn test_change_owner() {
    let env = Env::default();
    env.mock_all_auths();
    let test_admin = Address::generate(&env);
    let vault_address = Address::generate(&env);
    let contract_id = env.register(RampContract, (test_admin.clone(), vault_address, 10u32));
    let client = RampContractClient::new(&env, &contract_id);
    assert_eq!(client.get_owner(), Some(test_admin));
    let new_owner = Address::generate(&env);
    client.transfer_ownership(&new_owner, &(env.ledger().sequence() + 100));
    client.accept_ownership();
    assert_eq!(client.get_owner(), Some(new_owner));
}

#[test]
fn test_change_vault_address() {
    let env = Env::default();
    env.mock_all_auths();
    let test_admin = Address::generate(&env);
    let vault_address = Address::generate(&env);
    let contract_id = env.register(
        RampContract,
        (
            test_admin.clone(),
            vault_address.clone(),
            10u32
        )
    );

    let client = RampContractClient::new(&env, &contract_id);
    assert_eq!(client.get_vault_address(), vault_address);
    let new_vault_address = Address::generate(&env);
    client.change_vault_address(&new_vault_address);
    assert_eq!(client.get_vault_address(), new_vault_address);
}


#[test]
fn test_change_asset_fee_percentage() {
    let env = Env::default();
    env.mock_all_auths();
    let test_admin = Address::generate(&env);
    let vault_address = Address::generate(&env);
    let contract_id = env.register(RampContract, (test_admin.clone(), vault_address, 10u32));
    let token_id = env.register(ramp_token::RampToken, (test_admin.clone(),));
    let client = RampContractClient::new(&env, &contract_id);
    let token_client = ramp_token::RampTokenClient::new(&env, &token_id);
    token_client.mint(&test_admin, &10000i128);

    token_client.approve(&test_admin, &contract_id, &1000i128, &(env.ledger().sequence() + 100));
    client.add_asset(&token_id, &test_admin, &1i128);
    let new_fee_percentage = 3i128;
    client.change_asset_fee_percentage(&token_id, &new_fee_percentage);
    assert_eq!(client.get_asset_fee_percentage(&token_id), new_fee_percentage);
}

#[test]
fn test_pause() {
    let env = Env::default();
    env.mock_all_auths();
    let test_admin = Address::generate(&env);
    let vault_address = Address::generate(&env);
    let contract_id = env.register(RampContract, (test_admin.clone(), vault_address, 10u32));
    let client = RampContractClient::new(&env, &contract_id);
    assert_eq!(client.paused(), false);
    client.pause(&test_admin);
    assert_eq!(client.paused(), true);
}

#[test]
fn test_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let test_admin = Address::generate(&env);
    let vault_address = Address::generate(&env);
    let contract_id = env.register(RampContract, (test_admin.clone(), vault_address, 10u32));
    let client = RampContractClient::new(&env, &contract_id);
    assert_eq!(client.paused(), false);
    client.pause(&test_admin);
    assert_eq!(client.paused(), true);
    client.unpause(&test_admin);
    assert_eq!(client.paused(), false);
}

#[test]
fn test_onramp_deposit() {
    let env = Env::default();
    env.mock_all_auths();
    let test_admin = Address::generate(&env);
    let test_user = Address::generate(&env);
    let vault_address = Address::generate(&env);
    let contract_id = env.register(RampContract, (test_admin.clone(), vault_address, 10u32));
    let token_id = env.register(ramp_token::RampToken, (test_admin.clone(),));
    let client = RampContractClient::new(&env, &contract_id);
    let token_client = ramp_token::RampTokenClient::new(&env, &token_id);

    let user_init_balance = 10000i128;

    token_client.mint(&test_admin, &user_init_balance);

    assert_eq!(token_client.balance(&test_admin), user_init_balance);
    token_client.mint(&test_user, &user_init_balance);

    assert_eq!(token_client.balance(&test_user), user_init_balance);
    
    token_client.approve(&test_admin, &contract_id, &user_init_balance, &(env.ledger().sequence() + 100));
    client.add_asset(&token_id, &test_admin, &1i128);
    assert_eq!(token_client.balance(&contract_id), user_init_balance);

    let onramp_amount = 1000i128;
    let fee = client.get_asset_fee_percentage(&token_id);

    let fee_amount = onramp_amount * fee / 100;
    token_client.approve(&test_user, &contract_id, &onramp_amount, &(env.ledger().sequence() + 100));
    client.onramp_deposit(&token_id, &onramp_amount, &test_user, &OnrampMedium::Primary, &Region::KEN, &Bytes::from_slice(&env, b"test"));

    let topics = (Symbol::new(&env, "ONRAMP"), token_id.clone(), test_user.clone()).into_val(&env);
    let data = OnRampDepositEvent {
        amount: onramp_amount - fee_amount,
        medium: OnrampMedium::Primary,
        region: Region::KEN,
        data: Bytes::from_slice(&env, b"test")
    }.into_val(&env);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                token_id.clone(),
                (Symbol::new(&env, "transfer"), test_user.clone(), contract_id.clone()).into_val(&env),
                onramp_amount.into_val(&env),
            ),
            (
                contract_id.clone(),
                topics,
                data,
            ),
        ]
    );
    assert_eq!(token_client.balance(&contract_id), user_init_balance + onramp_amount);
    assert_eq!(token_client.balance(&test_user), user_init_balance - onramp_amount);
}

#[test]
fn test_withdraw_asset_revenue() {
    let env = Env::default();
    env.mock_all_auths();
    let test_admin = Address::generate(&env);
    let test_user = Address::generate(&env);
    let vault_address = Address::generate(&env);
    let contract_id = env.register(RampContract, (test_admin.clone(), vault_address.clone(), 10u32));
    let token_id = env.register(ramp_token::RampToken, (test_admin.clone(),));
    let client = RampContractClient::new(&env, &contract_id);
    let token_client = ramp_token::RampTokenClient::new(&env, &token_id);

    let user_init_balance = 10000i128;

    token_client.mint(&test_admin, &user_init_balance);

    assert_eq!(token_client.balance(&test_admin), user_init_balance);
    token_client.mint(&test_user, &user_init_balance);

    assert_eq!(token_client.balance(&test_user), user_init_balance);
    
    token_client.approve(&test_admin, &contract_id, &user_init_balance, &(env.ledger().sequence() + 100));
    client.add_asset(&token_id, &test_admin, &1i128);
    assert_eq!(token_client.balance(&contract_id), user_init_balance);

    let onramp_amount = 1000i128;
    token_client.approve(&test_user, &contract_id, &onramp_amount, &(env.ledger().sequence() + 100));
    client.onramp_deposit(
        &token_id,
        &onramp_amount,
        &test_user,
        &OnrampMedium::Primary,
        &Region::KEN,
        &Bytes::from_slice(&env, b"test")
    );

    assert_eq!(token_client.balance(&contract_id), user_init_balance + onramp_amount);
    assert_eq!(token_client.balance(&test_user), user_init_balance - onramp_amount);
    let asset_revenue = client.get_asset_revenue(&token_id);

    let topics = (Symbol::new(&env, "ASSET_REVENUE_WITHDRAWN"), token_id.clone()).into_val(&env);
    let data = RevenueWithdrawnEvent {
        recipient: client.get_vault_address(),
        amount: asset_revenue
    }.into_val(&env);
    client.withdraw_asset_revenue(&token_id);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                token_id.clone(),
                (Symbol::new(&env, "transfer"), contract_id.clone(), client.get_vault_address()).into_val(&env),
                asset_revenue.into_val(&env),
            ),
            (
                contract_id.clone(),
                topics,
                data,
            ),
        ]
    );

    assert_eq!(token_client.balance(&vault_address), asset_revenue);
    assert_eq!(client.get_asset_revenue(&token_id), 0);
}

#[test]
fn test_offramp_withdraw() {
    let env = Env::default();
    env.mock_all_auths();
    let test_admin = Address::generate(&env);
    let test_user = Address::generate(&env);
    let vault_address = Address::generate(&env);
    let contract_id = env.register(RampContract, (test_admin.clone(), vault_address, 10u32));
    let token_id = env.register(ramp_token::RampToken, (test_admin.clone(),));
    let client = RampContractClient::new(&env, &contract_id);
    let token_client = ramp_token::RampTokenClient::new(&env, &token_id);

    let user_init_balance = 10000i128;

    token_client.mint(&test_admin, &user_init_balance);

    assert_eq!(token_client.balance(&test_admin), user_init_balance);
    token_client.mint(&test_user, &user_init_balance);

    assert_eq!(token_client.balance(&test_user), user_init_balance);
    
    token_client.approve(&test_admin, &contract_id, &user_init_balance, &(env.ledger().sequence() + 100));
    client.add_asset(&token_id, &test_admin, &1i128);
    assert_eq!(token_client.balance(&contract_id), user_init_balance);

    let onramp_amount = 1000i128;
    token_client.approve(&test_user, &contract_id, &onramp_amount, &(env.ledger().sequence() + 100));
    client.onramp_deposit(&token_id, &onramp_amount, &test_user, &OnrampMedium::Primary, &Region::KEN, &Bytes::from_slice(&env, b"test"));

    assert_eq!(token_client.balance(&contract_id), user_init_balance + onramp_amount);
    assert_eq!(token_client.balance(&test_user), user_init_balance - onramp_amount);
    let recipient = Address::generate(&env);
    client.off_ramp_withdraw(&token_id, &recipient, &onramp_amount);

    let topics = (Symbol::new(&env, "OFFRAMP"), token_id.clone(), recipient.clone()).into_val(&env);
    let data = OffRampWithdrawEvent {
        amount: onramp_amount,
    }.into_val(&env);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                token_id.clone(),
                (Symbol::new(&env, "transfer"), contract_id.clone(), recipient.clone()).into_val(&env),
                onramp_amount.into_val(&env),
            ),
            (   
                contract_id.clone(),
                topics,
                data,
            ),
        ]
    );
}
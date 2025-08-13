use starknet::ContractAddress;
use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, spy_events, EventSpyAssertionsTrait};
use snforge_std::{start_cheat_caller_address, stop_cheat_caller_address};
use openzeppelin::access::ownable::interface::{IOwnableDispatcher, IOwnableDispatcherTrait};
use openzeppelin::security::interface::{IPausableDispatcher, IPausableDispatcherTrait};
use ramp_stark::interfaces::ramp_interface::{IRampStackDispatcher, IRampStackDispatcherTrait};
use ramp_stark::RampStark;
use openzeppelin::token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};
//use openzeppelin::upgrades::interface::{IUpgradeableDispatcher, IUpgradeableDispatcherTrait};


fn OWNER() -> ContractAddress {
    'OWNER'.try_into().unwrap()
}

fn ALLOWED_TOKEN() -> ContractAddress {
    'ALLOWED_TOKEN'.try_into().unwrap()
}

fn NEW_OWNER() -> ContractAddress {
    'NEW_OWNER'.try_into().unwrap()
}

fn VAULT() -> ContractAddress {
    'VAULT'.try_into().unwrap()
}


fn NEW_VAULT() -> ContractAddress {
    'NEW_VAULT'.try_into().unwrap()
}


fn deploy_contract() -> ContractAddress {
    let contract = declare("RampStark").unwrap().contract_class();

    let owner = OWNER();

    let mut calldata: Array::<felt252> = ArrayTrait::new();

    calldata.append(owner.into());

    calldata.append(VAULT().into());

    let (contract_address, _) = contract.deploy(@calldata).unwrap();

    contract_address
}

fn deploy_token() -> ContractAddress {
    let contract = declare("RampToken").unwrap().contract_class();

    let owner = NEW_OWNER();

    let mut calldata: Array::<felt252> = ArrayTrait::new();

    calldata.append(owner.into());

    calldata.append(owner.into());

    let (contract_address, _) = contract.deploy(@calldata).unwrap();

    contract_address
}


#[test]
fn test_check_owner() {
    let contract_address = deploy_contract();

    let dispatcher = IOwnableDispatcher { contract_address };

    let owner_before = dispatcher.owner();

    assert(owner_before == OWNER(), 'Invalid Owner');
}

#[test]
fn test_change_owner() {
    let contract_address = deploy_contract();

    let dispatcher = IOwnableDispatcher { contract_address };

    start_cheat_caller_address(contract_address, dispatcher.owner());

    dispatcher.transfer_ownership(NEW_OWNER());

    let owner_after = dispatcher.owner();

    assert(owner_after == NEW_OWNER(), 'Invalid Owner');
    stop_cheat_caller_address(contract_address);

}


#[test]
fn test_pause() {
    let contract_address = deploy_contract();

    let dispatcher = IPausableDispatcher { contract_address };

    start_cheat_caller_address(contract_address, OWNER());

    assert(!dispatcher.is_paused(), 'Contract Paused');
    stop_cheat_caller_address(contract_address);

}


// Test addition of an asset
#[test]
fn test_add_asset() {
    let contract_address = deploy_contract();

    let token_address = deploy_token();

    let dispatcher = IRampStackDispatcher { contract_address };

    let token_dispatcher = IERC20Dispatcher {
        contract_address: token_address
    };

    start_cheat_caller_address(token_address, NEW_OWNER());
    let amount = 100000000;

    token_dispatcher.approve(contract_address, amount);

    let allowance = token_dispatcher.allowance(NEW_OWNER(), contract_address);

    assert(amount == allowance, 'Allawance Mismatch');
    stop_cheat_caller_address(token_address);

    start_cheat_caller_address(contract_address, OWNER());

    let contract_balance_before = token_dispatcher.balance_of(contract_address);

    let mut spy = spy_events();

    assert(contract_balance_before == 0, 'Wrong balance before');
    dispatcher.add_allowed_asset(token_address, NEW_OWNER(), 2);

    spy.assert_emitted(
        @array![
            (
                contract_address,
                RampStark::Event::AssetAllowedAdded(
                    RampStark::AssetAllowedAdded {
                        asset: token_address,
                        funder: NEW_OWNER(),
                        initial_fee_percentage: 2,
                        initial_blalnce: amount
                    }
                ),
            ),
        ],
    );
    let balance_after = token_dispatcher.balance_of(contract_address);

    assert(balance_after == amount, 'Wrong balance after');
    assert(balance_after > contract_balance_before, 'Balance Mismatch');
    stop_cheat_caller_address(contract_address);
}

// Test removal of an asset
#[test]
fn test_remove_allowed_asset() {
    let contract_address = deploy_contract();

    let token_address = deploy_token();

    let dispatcher = IRampStackDispatcher { contract_address };

    let token_dispatcher = IERC20Dispatcher {
        contract_address: token_address
    };

    //add an asset to the contract 
    start_cheat_caller_address(token_address, NEW_OWNER());
    let amount = 100000000;

    token_dispatcher.approve(contract_address, amount);

    stop_cheat_caller_address(token_address);

    start_cheat_caller_address(contract_address, OWNER());

    dispatcher.add_allowed_asset(token_address, NEW_OWNER(), 2);

    //remove the asset
    let mut spy = spy_events();

    let vault_balance = token_dispatcher.balance_of(VAULT());
    let contract_balance = token_dispatcher.balance_of(contract_address);
    dispatcher.remove_allowed_asset(token_address, VAULT());
    let vault_balance_after = token_dispatcher.balance_of(VAULT());

    spy.assert_emitted(
        @array![
            (
                contract_address,
                RampStark::Event::AssetAllowedRemoved(
                    RampStark::AssetAllowedRemoved{
                        asset: token_address,
                        balance_recipient: VAULT(),
                        balance: contract_balance
                    },
                ),
            ),
        ],
    );
    let contract_balance_after = token_dispatcher.balance_of(contract_address);

    assert(vault_balance == contract_balance_after, 'Contract emptied');
    assert(vault_balance_after == contract_balance, 'Vault filled');
    stop_cheat_caller_address(contract_address);
}

#[test]
#[should_panic(expected: 'Ramp: Invalid Asset')]
fn test_remove_allowed_asset_wrong_token() {
    let contract_address = deploy_contract();

    let dispatcher = IRampStackDispatcher { contract_address };

    start_cheat_caller_address(contract_address, OWNER());
    dispatcher.remove_allowed_asset(NEW_OWNER(), VAULT());
    stop_cheat_caller_address(contract_address);
}

#[test]
#[should_panic(expected: 'Caller is not the owner')]
fn test_remove_allowed_asset_wrong_owner() {
    let contract_address = deploy_contract();

    let token_address = deploy_token();
    let dispatcher = IRampStackDispatcher { contract_address };
    
    start_cheat_caller_address(contract_address, OWNER());
    dispatcher.add_allowed_asset(token_address, OWNER(), 1);
    stop_cheat_caller_address(contract_address);

    start_cheat_caller_address(contract_address, NEW_OWNER());
    dispatcher.remove_allowed_asset(token_address, VAULT());
    stop_cheat_caller_address(contract_address);
}

#[test]
#[should_panic(expected: 'Caller is not the owner')]
fn test_add_allowed_asset_invalid_owner() {
    let contract_address = deploy_contract();
    let token_address = deploy_token();
    let dispatcher = IRampStackDispatcher { contract_address };
    let token_dispatcher = IERC20Dispatcher {
        contract_address: token_address
    };
    start_cheat_caller_address(token_address, OWNER());
    let amount = 1000;

    token_dispatcher.approve(contract_address, amount);

    stop_cheat_caller_address(token_address);
    
    start_cheat_caller_address(contract_address, NEW_OWNER());
    dispatcher.add_allowed_asset(token_address, OWNER(), 1);
    stop_cheat_caller_address(contract_address);
}

#[test]
#[should_panic(expected: 'Ramp: Invalid Fee Percentage')]
fn test_add_allowed_asset_invalid_fee() {
    let contract_address = deploy_contract();
    let token_address = deploy_token();
    let dispatcher = IRampStackDispatcher { contract_address };
    let token_dispatcher = IERC20Dispatcher {
        contract_address: token_address
    };
    start_cheat_caller_address(token_address, OWNER());
    let amount = 1000;

    token_dispatcher.approve(contract_address, amount);

    stop_cheat_caller_address(token_address);
    
    start_cheat_caller_address(contract_address, OWNER());
    dispatcher.add_allowed_asset(token_address, OWNER(), 101);
    stop_cheat_caller_address(contract_address);
}

#[test]
#[should_panic(expected: 'Ramp: Asset Already Added')]
fn test_add_allowed_asset_existing_asset() {
    let contract_address = deploy_contract();
    let token_address = deploy_token();
    let dispatcher = IRampStackDispatcher { contract_address };
    let token_dispatcher = IERC20Dispatcher {
        contract_address: token_address
    };
    start_cheat_caller_address(token_address, NEW_OWNER());
    let amount = 1000;

    token_dispatcher.approve(contract_address, amount);

    stop_cheat_caller_address(token_address);
    
    start_cheat_caller_address(contract_address, OWNER());
    dispatcher.add_allowed_asset(token_address, NEW_OWNER(), 1);
    stop_cheat_caller_address(contract_address);
    
    start_cheat_caller_address(contract_address, OWNER());
    dispatcher.add_allowed_asset(token_address, OWNER(), 1);
    stop_cheat_caller_address(contract_address);
}

#[test]
fn test_set_new_vault() {
    let contract_address = deploy_contract();

    let dispatcher = IRampStackDispatcher { contract_address };

    start_cheat_caller_address(contract_address, OWNER());
    let mut spy = spy_events();
    
    dispatcher.set_new_vault(NEW_VAULT());

    spy.assert_emitted(
        @array![
            (
                contract_address,
                RampStark::Event::VaultChanged(
                    RampStark::VaultChanged {
                        old_vault: VAULT(),
                        new_vault: NEW_VAULT(),
                    },
                ),
            ),
        ],
    );

    stop_cheat_caller_address(contract_address);
}

#[test]
fn test_set_fee() {
    let contract_address = deploy_contract();

    let token_address = deploy_token();

    let dispatcher = IRampStackDispatcher { contract_address };

    let token_dispatcher = IERC20Dispatcher {
        contract_address: token_address
    };

    start_cheat_caller_address(token_address, NEW_OWNER());
    let amount = 100000000;

    token_dispatcher.approve(contract_address, amount);

    stop_cheat_caller_address(token_address);

    start_cheat_caller_address(contract_address, OWNER());

    dispatcher.add_allowed_asset(token_address, NEW_OWNER(), 2);

    let fee = dispatcher.get_asset_fee_percentage(token_address);

    assert(fee == 2, 'wrong initial fee');

    let mut spy = spy_events();

    dispatcher.set_fee(token_address, 3);

    let current_fee = dispatcher.get_asset_fee_percentage(token_address);

    assert(current_fee == 3, 'wrong changed fee');
    spy.assert_emitted(
        @array![
            (
                contract_address,
                RampStark::Event::AssetFeeChanged(
                    RampStark::AssetFeeChanged {
                        asset: token_address,
                        old_fee: 2,
                        new_fee: 3
                    }
                )
            )
        ]
    )
}
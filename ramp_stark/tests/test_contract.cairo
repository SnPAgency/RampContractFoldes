use starknet::ContractAddress;
use snforge_std::{declare, ContractClassTrait, DeclareResultTrait};
use snforge_std::start_cheat_caller_address;
use openzeppelin::access::ownable::interface::{IOwnableDispatcher, IOwnableDispatcherTrait};
use openzeppelin::security::interface::{IPausableDispatcher, IPausableDispatcherTrait};
//use openzeppelin::upgrades::interface::{IUpgradeableDispatcher, IUpgradeableDispatcherTrait};
//use ramp_stark::interfaces::ramp_interface::{IRampStackDispatcher, IRampStackDispatcherTrait};

fn OWNNER() -> ContractAddress {
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


fn deploy_contract(name: ByteArray) -> ContractAddress {
    let contract = declare(name).unwrap().contract_class();

    let owner = OWNNER();

    let mut calldata: Array::<felt252> = ArrayTrait::new();

    calldata.append(owner.into());

    calldata.append(VAULT().into());

    let (contract_address, _) = contract.deploy(@calldata).unwrap();

    contract_address
}

#[test]
fn test_check_owner() {
    let contract_address = deploy_contract("RampStark");

    let dispatcher = IOwnableDispatcher { contract_address };

    let owner_before = dispatcher.owner();

    assert(owner_before == OWNNER(), 'Invalid Owner');
}

#[test]
fn test_change_owner() {
    let contract_address = deploy_contract("RampStark");

    let dispatcher = IOwnableDispatcher { contract_address };

    start_cheat_caller_address(contract_address, dispatcher.owner());

    dispatcher.transfer_ownership(NEW_OWNER());

    let owner_after = dispatcher.owner();

    assert(owner_after == NEW_OWNER(), 'Invalid Owner');
}


#[test]
fn test_pause() {
    let contract_address = deploy_contract("RampStark");

    let dispatcher = IPausableDispatcher { contract_address };

    start_cheat_caller_address(contract_address, OWNNER());

    assert(!dispatcher.is_paused(), 'Contract Paused');
}

//#[test]
//fn test_unpause() {
//    let mut contract = RampStark::
//}
//
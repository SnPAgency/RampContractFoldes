use ramp_stark::interfaces::ramp_interface::{OnrampMedium, Region};
use starknet::ContractAddress;
use snforge_std_deprecated::{declare, ContractClassTrait, DeclareResultTrait, spy_events, EventSpyAssertionsTrait};
use snforge_std_deprecated::{start_cheat_caller_address, stop_cheat_caller_address};
use ramp_stark::interfaces::ramp_interface::{IRampStackDispatcher, IRampStackDispatcherTrait};
use ramp_stark::RampStark;
use openzeppelin::token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};

fn OWNER() -> ContractAddress {
    'OWNER'.try_into().unwrap()
}

fn USER_1() -> ContractAddress {
    'USER_1'.try_into().unwrap()
}

fn USER_2() -> ContractAddress {
    'USER_2'.try_into().unwrap()
}

fn VAULT() -> ContractAddress {
    'VAULT'.try_into().unwrap()
}

fn deploy_contract() -> (ContractAddress, ContractAddress) {
    let contract = declare("RampStark").unwrap().contract_class();
    let token_contract = declare("RampToken").unwrap().contract_class();

    let owner = OWNER();

    let mut calldata: Array::<felt252> = ArrayTrait::new();
    let mut token_calldata: Array::<felt252> = ArrayTrait::new();

    calldata.append(owner.into());

    calldata.append(VAULT().into());

    token_calldata.append(USER_1().into());

    token_calldata.append(owner.into());

    let (contract_address, _) = contract.deploy(@calldata).unwrap();

    let (token_contract_address, _) = token_contract.deploy(@token_calldata).unwrap();


    (contract_address, token_contract_address)
}

fn set_up_contract(contract: ContractAddress, token: ContractAddress) -> (IRampStackDispatcher, IERC20Dispatcher) {
    let mut ramp_contract = IRampStackDispatcher { contract_address: contract };
    let mut ramp_token = IERC20Dispatcher {contract_address: token};

    assert(ramp_token.balance_of(USER_1()) == 100000000000000000000000, 'User 1 balance Err');

    // user one send some tokens to the owner
    start_cheat_caller_address(token, USER_1());
    ramp_token.transfer(OWNER(), 10000);

    assert(ramp_token.balance_of(OWNER()) == 10000, 'Failed Transfer to Owner');

    stop_cheat_caller_address(token);
    // add token as allowed

    start_cheat_caller_address(token, OWNER());
    ramp_token.approve(contract, 10000);
    assert(ramp_token.allowance(OWNER(), contract) == 10000, 'Allawance mismatch');
    stop_cheat_caller_address(token);

    start_cheat_caller_address(contract, OWNER());
    ramp_contract.add_allowed_asset(token, OWNER(), 1);
    assert(ramp_contract.is_asset_allowed(token), 'Asset not Added');
    assert(ramp_token.balance_of(contract) == 10000, 'balance Mismatch');
    stop_cheat_caller_address(contract);
    
    (ramp_contract, ramp_token)
}

#[test]
fn test_onramp() {
    let (contract, token) = deploy_contract();

    let (ramp_contract, ramp_token) = set_up_contract(contract, token);

    start_cheat_caller_address(token, USER_1());
    // approve the contract to spend initial liquidity
    ramp_token.approve(contract, 1000);
    stop_cheat_caller_address(token);
    start_cheat_caller_address(contract, USER_1());

    let contract_balance_before = ramp_token.balance_of(contract);
    let mut spy = spy_events();
    ramp_contract.on_ramp_deposit(token, 1000, USER_1(), OnrampMedium::Primary, Region::NGA, "");    

    let fee_percentage: u256 = ramp_contract.get_asset_fee_percentage(token).into();

    let fee = (fee_percentage * 1000) / 100;
    spy.assert_emitted(
        @array![
            (
                contract,
                RampStark::Event::RampDeposit(
                    RampStark::RampDeposit {
                        asset: token,
                        amount: (1000 - fee),
                        sender: USER_1(),
                        medium: OnrampMedium::Primary,
                        region: Region::NGA,
                        data: ""
                    }
                ),
            ),
        ],
    );
    stop_cheat_caller_address(contract);
    let contract_balance_after = ramp_token.balance_of(contract);
    assert(contract_balance_after == contract_balance_before + 1000, 'Wrong balance after deposit');
}

#[test]
#[should_panic(expected: 'Ramp: Invalid Asset')]
fn test_on_ramp_wrong_token() {
    let (contract, token) = deploy_contract();

    let (ramp_contract, _) = set_up_contract(contract, token);

    start_cheat_caller_address(contract, USER_1());
    ramp_contract.on_ramp_deposit(USER_2(), 1000, USER_1(), OnrampMedium::Primary, Region::NGA, "");
    stop_cheat_caller_address(contract);
}

#[test]
fn test_offramp_withdraw() {
    let (contract, token) = deploy_contract();

    let (ramp_contract, ramp_token) = set_up_contract(contract, token);
    let asset_is_added = ramp_contract.is_asset_allowed(token);
    assert(asset_is_added, 'Asset_not_added');


    start_cheat_caller_address(contract, OWNER());
    let mut spy = spy_events();
    let contract_balance_before = ramp_token.balance_of(contract);

    assert(contract_balance_before > 0, 'Balance Error before');

    ramp_contract.off_ramp_withdraw(token, 100, USER_2());
    spy.assert_emitted(
        @array![
            (
                contract,
                RampStark::Event::RampWithdraw(
                    RampStark::RampWithdraw {
                        asset: token,
                        amount: 100,
                        recipient: USER_2()
                    }
                ),
            ),
        ],
    );
    stop_cheat_caller_address(contract);
    let contract_balance_after = ramp_token.balance_of(contract);
    assert(contract_balance_after == contract_balance_before - 100, 'Wrong balance after withdraw');
}

#[test]
#[should_panic(expected: 'Ramp: Invalid Asset')]
fn test_off_ramp_withdraw_wrong_token() {
    let (contract, token) = deploy_contract();

    let (ramp_contract, _) = set_up_contract(contract, token);

    start_cheat_caller_address(contract, OWNER());
    ramp_contract.off_ramp_withdraw(USER_2(), 1000, USER_1());
    stop_cheat_caller_address(contract);
}

#[test]
#[should_panic(expected: 'Caller is not the owner')]
fn test_off_ramp_withdraw_wrong_owner() {
    let (contract, token) = deploy_contract();

    let (ramp_contract, _) = set_up_contract(contract, token);

    start_cheat_caller_address(contract, USER_2());
    ramp_contract.off_ramp_withdraw(token, 1000, USER_1());
    stop_cheat_caller_address(contract);
}

use starknet::ContractAddress;
#[derive(Serde, PartialEq, Drop)]
pub enum OnrampMedium {
    Primary,
    Secondary,
    Tertiary
}


#[derive(Serde, PartialEq, Drop)]
pub enum Region {
    KEN,
    RWN,
    NGA,
    SA,
    EGY,
    GHN
}

#[starknet::interface]
pub trait IRampStack<TContractState> {
    fn add_allowed_asset(ref self: TContractState, asset: ContractAddress, funder: ContractAddress, fee_percentage: u8);

    fn remove_allowed_asset(ref self: TContractState, asset: ContractAddress, token_receiver: ContractAddress);

    fn on_ramp_deposit(
        ref self: TContractState,
        asset: ContractAddress,
        amount: u256,
        sender: ContractAddress,
        medium: OnrampMedium,
        region: Region,
        data: felt252
    );

    fn off_ramp_withdraw(
        ref self: TContractState,
        asset: ContractAddress,
        amount: u256,
        recipient: ContractAddress
    );

    fn is_asset_allowed(self: @TContractState, asset: ContractAddress) -> bool;

    fn get_allowed_assets(self: @TContractState) -> Array<ContractAddress>;

    fn set_new_vault(ref self: TContractState, vault: ContractAddress);

    fn set_fee(ref self: TContractState, asset: ContractAddress, fee: u256);

    fn withdraw_asset_revenue(ref self: TContractState, asset: ContractAddress);

    fn get_asset_revenue(self: @TContractState, asset: ContractAddress) -> u256;

    fn get_asset_fee_percentage(self: @TContractState, asset: ContractAddress) -> u8;

    fn vault(self: @TContractState) -> ContractAddress;


}

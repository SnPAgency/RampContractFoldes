
use starknet::ContractAddress;

#[starknet::interface]
pub trait IRampStack<TContractState> {
    fn add_allowed_asset(ref self: TContractState, asset: ContractAddress);

    fn remove_allowed_asset(ref self: TContractState, asset: ContractAddress);

    fn on_ramp_deposit(
        ref self: TContractState,
        asset: ContractAddress,
        amount: u256,
        sender: ContractAddress
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

}

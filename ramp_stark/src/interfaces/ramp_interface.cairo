


#[starknet::interface]
pub trait IRampStack<TContractState> {

    fn add_asset(self: @TContractState);

    fn remove_asset(self: @TContractState);

}

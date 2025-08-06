use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RampContractError {
    //Indicates insufficient funds from the account trying to make the asset transfer
    InsufficientFunds = 0,

    //Indicates the asset is not allowed to be used in the contract
    AssetNotAllowed = 1,

    //Indicates the asset already exists in the contract
    AssetAlreadyExists = 2,

    //Indicates the asset not found in the contract
    AssetNotFound = 3,

    //Indicates the fee percentage is invalid
    InvalidFeePercentage = 4,

    //Indicates the vault address is invalid
    InvalidVaultAddress = 5,

}

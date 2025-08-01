use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RampError {
    #[error("Invalid instruction")]
    InvalidInstruction,

    #[error("Account data is not initialized")]
    UninitializedAccount,

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Invalid account owner")]
    InvalidAccountOwner,

    #[error("Invalid account state")]
    InvalidAccountState,

    #[error("Account is already initialized")]
    AccountAlreadyInitialized,

    #[error("Rent Error")]
    RentError,

    #[error("Invalid Payer")]
    InvalidSigner,

    #[error("Unauthorized: Only owner can perform this action")]
    Unauthorized,

    #[error("Asset already exists")]
    AssetAlreadyExists,

    #[error("Asset not found")]
    AssetNotFound,

    #[error("Invalid fee percentage")]
    InvalidFeePercentage,

    #[error("Program is not active")]
    ProgramNotActive,
}

impl From<RampError> for ProgramError {
    fn from(error: RampError) -> Self {
        ProgramError::Custom(error as u32)
    }
}
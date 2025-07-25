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
}

impl From<RampError> for ProgramError {
    fn from(error: RampError) -> Self {
        ProgramError::Custom(error as u32)
    }
}
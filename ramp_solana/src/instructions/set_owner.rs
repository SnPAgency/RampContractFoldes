use crate::{errors::RampError, state::RampState};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, 
    msg, 
    pubkey::Pubkey
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SetOwnerInstruction {
    pub new_owner: Pubkey,
}

pub fn set_owner(_program_id: &Pubkey, accounts: &[AccountInfo], args: SetOwnerInstruction) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;
    let current_owner_account = next_account_info(account_info_iter)?;

    msg!("Setting new owner...");

    // Deserialize the RampState
    let mut ramp_data = ramp_account.try_borrow_mut_data()?;
    let mut ramp_state: RampState = RampState::try_from_slice(&ramp_data)?;

    // Check if the ramp account is initialized
    if !ramp_state.is_initialized {
        msg!("Ramp account is not initialized");
        return Err(RampError::UninitializedAccount.into());
    }

    // Check current owner authorization
    if !current_owner_account.is_signer {
        msg!("Current owner account must be signer");
        return Err(RampError::InvalidSigner.into());
    }

    if ramp_state.owner != *current_owner_account.key {
        msg!("Only current owner can change ownership");
        return Err(RampError::Unauthorized.into());
    }

    // Set the new owner
    ramp_state.owner = args.new_owner;

    // Clear the account data first
    ramp_data.fill(0);
    
    // Serialize the updated state back to the account data
    let serialized_data = borsh::to_vec(&ramp_state).map_err(|e| {
        msg!("Serialization failed: {:?}", e);
        RampError::InvalidAccountState
    })?;
    
    if serialized_data.len() > ramp_data.len() {
        msg!("Insufficient account space: need {}, have {}", serialized_data.len(), ramp_data.len());
        return Err(RampError::InvalidAccountState.into());
    }
    
    ramp_data[..serialized_data.len()].copy_from_slice(&serialized_data);
    msg!("State serialized successfully");

    msg!("Ramp account owner set to {}", args.new_owner);
    Ok(())
}
use crate::{errors::RampError, state::RampState};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, 
    msg, 
    pubkey::Pubkey
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SetActiveInstruction {
    pub is_active: bool,
}


pub fn set_active(_program_id: &Pubkey, accounts: &[AccountInfo], args: SetActiveInstruction) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;
    let owner_account = next_account_info(account_info_iter)?;

    msg!("Setting active state to {}...", args.is_active);

    let mut ramp_state: RampState  = {
        let ramp_data = ramp_account.try_borrow_data()?;
        borsh::from_slice(&ramp_data)?
    };

    if !owner_account.is_signer {
        msg!("Owner account must be signer");
        return Err(RampError::InvalidSigner.into());
    }

    ramp_state.set_active(args.is_active);
    let mut ramp_data = ramp_account.try_borrow_mut_data()?;
    ramp_data.fill(0);
    let serialized_data = borsh::to_vec(&ramp_state).map_err(|e| {
        msg!("Serialization failed: {:?}", e);
        RampError::InvalidAccountState
    })?;
    
    if serialized_data.len() > ramp_data.len() {
        msg!("Insufficient account space: need {}, have {}", serialized_data.len(), ramp_data.len());
        return Err(RampError::InvalidAccountState.into());
    }
    
    ramp_data[..serialized_data.len()].copy_from_slice(&serialized_data);
    
    msg!("Ramp account active state set to {}", args.is_active);
    Ok(())
}
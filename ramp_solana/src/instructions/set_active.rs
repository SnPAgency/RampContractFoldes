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
    let mut ramp_state: RampState  = {
        let ramp_data = ramp_account.try_borrow_data()?;
        borsh::from_slice(&ramp_data)?
    };
    if owner_account.key != &ramp_state.owner {
        return Err(RampError::Unauthorized.into());
    }

    let (owner, signer) = (owner_account.key == &ramp_state.owner, owner_account.is_signer);

    match (owner, signer) {
        (true, true) => {
            ramp_state.set_active(args.is_active);
            let mut ramp_data = ramp_account.try_borrow_mut_data()?;
            ramp_data.fill(0);
            let serialized_data = borsh::to_vec(&ramp_state).expect("Failed to serialize ramp state");
            if serialized_data.len() > ramp_data.len() {
                return Err(RampError::InvalidAccountState.into());
            }
            ramp_data[..serialized_data.len()].copy_from_slice(&serialized_data);
            msg!("Ramp account active state set to {}", args.is_active);
            Ok(())
        },
        (true, false) => {
            return Err(RampError::InvalidSigner.into());
        },
        _ => {
            return Err(RampError::Unauthorized.into());
        }
    }
}
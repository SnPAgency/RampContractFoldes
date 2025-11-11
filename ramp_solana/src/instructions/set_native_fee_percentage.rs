use crate::{errors::RampError, state::RampState};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, 
    msg, 
    pubkey::Pubkey,
};


#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SetNativeFeePercentageInstruction {
    pub fee_percentage: u128,
}

pub fn set_native_fee_percentage(_program_id: &Pubkey, accounts: &[AccountInfo], args: SetNativeFeePercentageInstruction) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;
    let owner_account = next_account_info(account_info_iter)?;    
    let mut ramp_state: RampState = {
        let ramp_data = ramp_account.try_borrow_data()?;
        borsh::from_slice(&ramp_data)?
    };
    let (owner, signer) = (owner_account.key == &ramp_state.owner, owner_account.is_signer);
    match (owner, signer) {
        (true, true) => {
            ramp_state.set_native_fee_percentage(args.fee_percentage);
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
            msg!("Native fee percentage set to {}", args.fee_percentage);
            Ok(())
        },
        (false, _) => {
            return Err(RampError::Unauthorized.into());
        }
        _ => {
            return Err(RampError::InvalidSigner.into());
        }
    }
}
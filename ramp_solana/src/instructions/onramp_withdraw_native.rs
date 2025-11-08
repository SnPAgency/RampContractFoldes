use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, 
    pubkey::Pubkey,
};

use crate::{errors::RampError, state::RampState};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct OnRampWithdrawNativeInstruction {
    pub amount: u64,
}

pub fn onramp_withdraw_native(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: OnRampWithdrawNativeInstruction
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;
    let recipient_account = next_account_info(account_info_iter)?;
    let owner_account = next_account_info(account_info_iter)?;

    let ramp_state: RampState = {
        let ramp_data = ramp_account.try_borrow_data()?;
        RampState::try_from_slice(&ramp_data)?
    };

    if !ramp_state.is_active {
        return Err(RampError::ProgramNotActive.into());
    }
    if owner_account.key != &ramp_state.owner && !owner_account.is_signer {
        return Err(RampError::Unauthorized.into());
    }
    
    **ramp_account.try_borrow_mut_lamports()? -= args.amount;
    **recipient_account.try_borrow_mut_lamports()? += args.amount;
    
    Ok(())
}

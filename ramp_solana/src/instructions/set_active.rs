use {
    crate::{errors::RampError, state::RampState}, borsh::{BorshDeserialize, BorshSerialize}, solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        //program::invoke,
        pubkey::Pubkey,
    }
};


pub fn set_active(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;

    // Set the active state
    let mut ramp_data = ramp_account.try_borrow_mut_data()?;
    let mut ramp_state: RampState = RampState::try_from_slice(&ramp_data)?;

    // Check if the ramp account is initialized
    if !ramp_state.is_initialized {
        msg!("Ramp account is not initialized");
        return Err(RampError::UninitializedAccount.into());
    }

    //update the active state
    ramp_state.is_active = true;

    // Serialize the updated state back to the account data
    ramp_state.serialize(&mut &mut ramp_data[..])?;
    msg!("Ramp account active state set to {}", true);
    Ok(())
}
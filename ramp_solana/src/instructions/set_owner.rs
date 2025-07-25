use {
    crate::{errors::RampError, state::RampState}, borsh::{BorshDeserialize, BorshSerialize}, solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        //program::invoke,
        pubkey::Pubkey,
    }
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SetOwnerInstruction {
    pub new_owner: Pubkey,
}

pub fn set_owner(_program_id: &Pubkey, accounts: &[AccountInfo], args: SetOwnerInstruction) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;


    // Deserialize the RampState
    let mut ramp_data = ramp_account.try_borrow_mut_data()?;
    let mut ramp_state: RampState = RampState::try_from_slice(&ramp_data)?;

    // Check if the ramp account is initialized
    if !ramp_state.is_initialized {
        msg!("Ramp account is not initialized");
        return Err(RampError::UninitializedAccount.into());
    }
    // Set the new owner
    ramp_state.owner = args.new_owner;

    // Serialize the updated state back to the account data
    ramp_state.serialize(&mut &mut ramp_data[..])?;

    msg!("Ramp account owner set to {}", args.new_owner);
    Ok(())
}
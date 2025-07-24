use {
    crate::{errors::RampError, state::RampState}, borsh::{BorshDeserialize, BorshSerialize}, solana_program::{
        account_info::{next_account_info, AccountInfo}, entrypoint::ProgramResult, msg, pubkey::Pubkey
    }
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct RemoveAssetsInstruction {
    pub assets: Vec<Pubkey>,
}

pub fn remove_assets(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: RemoveAssetsInstruction
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;

    if ramp_account.data_is_empty() {
        msg!("Ramp account is not initialized");
        return Err(RampError::UninitializedAccount.into());
    }

    let mut ramp_data = ramp_account.try_borrow_mut_data()?;
    let mut ramp_state: RampState = RampState::try_from_slice(&ramp_data)?;

    // Remove the specified assets
    for asset in args.assets {
        ramp_state.assets.retain(|&x| x != asset);
    }

    // Serialize the updated state back to the account data
    ramp_state.serialize(&mut &mut ramp_data[..])?;
    msg!("Assets removed successfully");
    msg!("Removing assets...");
    Ok(())
}
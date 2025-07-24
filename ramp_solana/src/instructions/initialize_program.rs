use {
    crate::{errors::RampError, state::RampState},
    borsh::BorshSerialize,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::invoke,
        rent::Rent,
        system_instruction::create_account,
        sysvar::Sysvar,
        pubkey::Pubkey,
    },
};
use std::mem::size_of
;
pub fn initialize_program(
    program_id: &Pubkey,
    accounts: &[AccountInfo]
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;
    let payer_account = next_account_info(account_info_iter)?;
    let system_program_account = next_account_info(account_info_iter)?;

    let account_space = size_of::<RampState>();

    let rent_required = Rent::get()
        .map_err(|_| RampError::RentError)?
        .minimum_balance(account_space);

    if !payer_account.is_signer {
        msg!("Payer account must be a signer");
        return Err(RampError::InvalidPayer.into());
    }
    invoke(
        &create_account(
            payer_account.key,
            ramp_account.key,
            rent_required,
            account_space as u64,
            program_id,
        ),
        &[
            ramp_account.clone(),
            payer_account.clone(),
            system_program_account.clone(),
        ],
    )?;

    if !ramp_account.data_is_empty() {
        msg!("Ramp account is already initialized");
        return Err(RampError::AccountAlreadyInitialized.into());
    }

    let mut ramp_data = ramp_account.try_borrow_mut_data()?;
    let mut ramp_state = RampState::default();
    ramp_state.is_initialized = true;
    ramp_state.serialize(&mut &mut ramp_data[..])?;

    msg!("Ramp account initialized");
    Ok(())
}
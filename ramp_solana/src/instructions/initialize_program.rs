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

pub fn initialize_program(
    program_id: &Pubkey,
    accounts: &[AccountInfo]
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;
    let payer_account = next_account_info(account_info_iter)?;
    let system_program_account = next_account_info(account_info_iter)?;

    let account_space = 8 + RampState::get_space_with_assets(10);

    let rent_required = Rent::get()
        .map_err(|_| RampError::RentError)?
        .minimum_balance(account_space);

    if !payer_account.is_signer {
        msg!("Payer account should not be the signer");
        return Err(RampError::InvalidSigner.into());
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

    let mut ramp_data = ramp_account.try_borrow_mut_data()?;
    let mut ramp_state = RampState::default();
    ramp_state.is_initialized = true;
    ramp_state.owner = *payer_account.key;
    ramp_state.serialize(&mut &mut ramp_data[..])?;

    msg!("Ramp account initialized");
    Ok(())
}
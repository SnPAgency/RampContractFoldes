use crate::{errors::RampError, state::RampState};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, 
    msg, 
    program::invoke,
    rent::Rent, 
    sysvar::Sysvar,
};
use solana_system_interface::instruction::create_account;
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct InitializeProgramInstruction {
    pub vault_address: Pubkey,
    pub native_fee_percentage: u128,
}

pub fn initialize_program(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: InitializeProgramInstruction
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;
    let payer_account = next_account_info(account_info_iter)?;
    let system_program_account = next_account_info(account_info_iter)?;

    let account_space = borsh::to_vec(&RampState::default()).unwrap().len();

    let rent_required = Rent::get()
        .map_err(|_| RampError::RentError)?
        .minimum_balance(account_space);
    invoke(
        &create_account(
            payer_account.key,
            ramp_account.key,
            rent_required,
            account_space as u64,
            program_id,
        ),
        &[
            payer_account.clone(),
            ramp_account.clone(),
            system_program_account.clone(),
        ],
    )?;

    let mut ramp_data = ramp_account.try_borrow_mut_data()?;
    let mut ramp_state = RampState::default();
    ramp_state.is_active = true;
    ramp_state.owner = *payer_account.key;
    ramp_state.vault_address = args.vault_address;
    ramp_state.native_fee_percentage = args.native_fee_percentage;
    
    ramp_data.fill(0);
    let serialized_data = borsh::to_vec(&ramp_state).expect("Failed to serialize ramp state");
    
    if serialized_data.len() > ramp_data.len() {
        return Err(RampError::InvalidAccountState.into());
    }
    ramp_data[..serialized_data.len()].copy_from_slice(&serialized_data);

    msg!("Ramp account initialized");
    Ok(())
}
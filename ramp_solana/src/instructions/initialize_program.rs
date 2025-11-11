use crate::{errors::RampError, state::RampState};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{
        AccountInfo,
        next_account_info
    },
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    rent::Rent,
    sysvar::Sysvar
};
use solana_system_interface::instruction::create_account;
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct InitializeProgramInstruction {
    pub bump: u8,
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
    invoke_signed(
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
        &[&[b"ramp", payer_account.key.as_ref(), &[args.bump]]],
    )?;
    let mut ramp_data = ramp_account.try_borrow_mut_data()?;
    let mut ramp_state = RampState::default();
    ramp_state.is_active = true;
    ramp_state.owner = *payer_account.key;
    ramp_state.vault_address = args.vault_address;
    ramp_state.native_fee_percentage = args.native_fee_percentage;
    ramp_state.serialize(&mut ramp_data.as_mut())?;
    msg!("account initialized");
    Ok(())
}
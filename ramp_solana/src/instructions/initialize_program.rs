use crate::{errors::RampError, state::RampState};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, 
    msg, 
    program::invoke,
    pubkey::Pubkey, 
    rent::Rent, 
    sysvar::Sysvar,
    system_instruction::create_account
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct InitializeProgramInstruction {
    pub vault_address: Pubkey,
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
    ramp_state.owner = *payer_account.key;
    ramp_state.vault_address = args.vault_address;
    
    // Clear the account data first
    ramp_data.fill(0);
    
    // Serialize the initial state to the account data
    let serialized_data = borsh::to_vec(&ramp_state).map_err(|e| {
        msg!("Serialization failed: {:?}", e);
        RampError::InvalidAccountState
    })?;
    
    if serialized_data.len() > ramp_data.len() {
        msg!("Insufficient account space: need {}, have {}", serialized_data.len(), ramp_data.len());
        return Err(RampError::InvalidAccountState.into());
    }
    
    ramp_data[..serialized_data.len()].copy_from_slice(&serialized_data);
    msg!("State serialized successfully");

    msg!("Ramp account initialized");
    Ok(())
}
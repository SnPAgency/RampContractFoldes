use crate::{errors::RampError, models::{Medium, Region}, state::RampState};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, 
    pubkey::Pubkey,
    program::invoke,
    msg
};
use solana_system_interface::instruction::transfer;
use crate::models::RampDeposit;
use base64::{engine::general_purpose, Engine as _};


#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct OffRampDepositNativeInstruction {
    pub amount: u64,
    pub region: Region,
    pub medium: Medium,
    pub data: Vec<u8>,
}

pub fn off_ramp_deposit_native(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: OffRampDepositNativeInstruction
) -> ProgramResult {

    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;
    let depositor_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    let mut ramp_state: RampState = {
        let ramp_data = ramp_account.try_borrow_data()?;
        RampState::try_from_slice(&ramp_data)?
    };

    if !ramp_state.is_active {
        return Err(RampError::ProgramNotActive.into());
    }

    invoke(
        &transfer(
            depositor_account.key,
            ramp_account.key,
            args.amount
        ),
        &[
            depositor_account.clone(),
            ramp_account.clone(),
            system_program.clone(),
        ],
    )?;

    let fee = ramp_state.native_fee_percentage * (args.amount / 100) as u128;
    ramp_state.update_native_revenue(fee);
    let mut ramp_data = ramp_account.try_borrow_mut_data()?;
    ramp_data.fill(0);
    let serialized_data = borsh::to_vec(&ramp_state).expect("Failed to serialize ramp state");
    if serialized_data.len() > ramp_data.len() {
        return Err(RampError::InvalidAccountState.into());
    }
    ramp_data[..serialized_data.len()].copy_from_slice(&serialized_data);
    msg!("RampDeposit:{}", general_purpose::STANDARD.encode(
        borsh::to_vec(&RampDeposit {
            asset: Pubkey::default(),
            asset_name: "SOL".to_string(),
            amount: args.amount,
            sender: *depositor_account.key,
            region: args.region,
            medium: args.medium,
            data: args.data,
        }).unwrap()
    ));
    
    Ok(())
}

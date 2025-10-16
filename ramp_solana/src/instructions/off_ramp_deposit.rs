use crate::{errors::RampError, models::{Medium, Region}, state::RampState};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, 
    msg, 
    pubkey::Pubkey,
    program::invoke,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction as token_instruction;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct OffRampDepositInstruction {
    pub amount: u64,
    pub region: Region,
    pub medium: Medium,
}

pub fn off_ramp_deposit(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: OffRampDepositInstruction
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;
    let asset_mint_account = next_account_info(account_info_iter)?;
    let asset_owner_account = next_account_info(account_info_iter)?;
    let asset_owner_token_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    
    msg!("Off-ramping deposit...");

    let mut ramp_data = ramp_account.try_borrow_mut_data()?;
    let mut ramp_state: RampState = borsh::from_slice(&ramp_data)?;

    if !ramp_state.is_active {
        msg!("Ramp account is not active or owner is not signer");
        return Err(RampError::UninitializedAccount.into());
    }

    match ramp_state.get_asset_info(asset_mint_account.key) {
        Some(asset) => {
            let revenue = (args.amount as u128) * (asset.get_fee_percentage() / 100);
            asset.add_revenue(revenue);
        },
        None => {
            msg!("Asset not found");
            return Err(RampError::AssetNotFound.into());
        }
    }

    let ramp_associated_token_account = get_associated_token_address(
        ramp_account.key,
        asset_mint_account.key,
    );
    let transfer_instructions = token_instruction::transfer(
        token_program.key,
        &asset_owner_token_account.key,
        &ramp_associated_token_account,
        asset_owner_account.key,
        &[asset_owner_account.key],
        args.amount,
    )?;

    let transfer_result = invoke(
        &transfer_instructions,
        &[
            asset_owner_account.clone(),
            ramp_account.clone(),
            asset_owner_account.clone(),
            token_program.clone(),
        ],
    );

    if transfer_result.is_err() {
        msg!("Transfer failed");
        return Err(RampError::TransferFailed.into());
    }

    // Clear the account data first
    ramp_data.fill(0);
    
    // Serialize the updated state back to the account data
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
    Ok(())
}
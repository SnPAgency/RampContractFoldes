use crate::{errors::RampError, state::RampState};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, 
    msg, 
    pubkey::Pubkey,
    program_pack::Pack,
    program::invoke,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct RemoveAssetsInstruction {
    //pub asset: Pubkey,
}

pub fn remove_assets(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _args: RemoveAssetsInstruction
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;

    let ramp_associated_token_account = next_account_info(account_info_iter)?;

    let asset_mint_account = next_account_info(account_info_iter)?;

    let owner_account = next_account_info(account_info_iter)?;

    let owner_token_account = next_account_info(account_info_iter)?;

    let token_program = next_account_info(account_info_iter)?;
    
    msg!("Removing assets...");

    // Check owner authorization
    if !owner_account.is_signer {
        msg!("Owner account must be signer");
        return Err(RampError::InvalidSigner.into());
    }
    
    // Check ramp account is signer (needed for token transfer)
    if !ramp_account.is_signer {
        msg!("Ramp account must be signer for token transfer");
        return Err(RampError::InvalidSigner.into());
    }

    // Read the ramp state without holding the borrow
    let mut ramp_state: RampState = {
        let ramp_data = ramp_account.try_borrow_data()?;
        borsh::from_slice(&ramp_data)?
    };

    let ramp_associated_token_account_data = spl_token::state::Account::unpack(&ramp_associated_token_account.try_borrow_data()?)?;

    if ramp_associated_token_account_data.amount > 0 {
        let transfer_instructions = spl_token::instruction::transfer(
            token_program.key,
            ramp_associated_token_account.key,
            owner_token_account.key,
            ramp_account.key, // Ramp account is the authority
            &[ramp_account.key],
            ramp_associated_token_account_data.amount,
        )?;
        
        invoke(
            &transfer_instructions,
            &[
                ramp_associated_token_account.clone(),
                owner_token_account.clone(),
                ramp_account.clone(),
                token_program.clone(),
            ]
        )?;
    }

    // Remove the specified assets
    match ramp_state.remove_asset(asset_mint_account.key) {
        Ok(()) => msg!("Removed asset: {}", asset_mint_account.key),
        Err(_) => {
            msg!("Asset not found: {}", asset_mint_account.key);
            return Err(RampError::AssetNotFound.into());
        }
    }

    // Now borrow mutably to update the state
    let mut ramp_data = ramp_account.try_borrow_mut_data()?;
    
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
    
    msg!("Assets removed successfully");
    Ok(())
}
use crate::{errors::RampError, state::RampState};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, 
    msg, 
    pubkey::Pubkey
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AddAssetsInstruction {
    pub asset: Pubkey,
    pub fee_percentage: u128,
}

pub fn add_assets(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: AddAssetsInstruction
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;
    let owner_account = next_account_info(account_info_iter)?;
    
    msg!("Adding assets...");

    let mut ramp_data = ramp_account.try_borrow_mut_data()?;
    let mut ramp_state: RampState = borsh::from_slice(&ramp_data)?;

    if  !ramp_state.is_active && !owner_account.is_signer {
        msg!("Ramp account is not active or owner is not signer");
        return Err(RampError::UninitializedAccount.into());
    }

    // Add the new assets with their fee percentages
    let fee_percentage = args.fee_percentage;
    let asset = args.asset;
        
    // Validate fee percentage (e.g., max 10000 basis points = 100%)
    if fee_percentage > 10000 {
        msg!("Fee percentage too high: {}", fee_percentage);
        return Err(RampError::InvalidFeePercentage.into());
    }

    match ramp_state.add_asset(asset, fee_percentage) {
        Ok(()) => msg!("Added asset: {}", asset),
        Err(_) => {
            msg!("Asset already exists: {}", asset);
            return Err(RampError::AssetAlreadyExists.into());
        }
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
    
    msg!("Assets added successfully");
    Ok(())
}
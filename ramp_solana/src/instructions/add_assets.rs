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
    pub assets: Vec<Pubkey>,
    pub fee_percentages: Vec<u128>,
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

    // Validate input
    if args.assets.len() != args.fee_percentages.len() {
        msg!("Assets and fee percentages length mismatch");
        return Err(RampError::InvalidInstruction.into());
    }

    if args.assets.is_empty() {
        msg!("No assets provided");
        return Err(RampError::InvalidInstruction.into());
    }

    let mut ramp_data = ramp_account.try_borrow_mut_data()?;
    let mut ramp_state: RampState = RampState::try_from_slice(&ramp_data)?;

    if !ramp_state.is_initialized {
        msg!("Ramp account is not initialized");
        return Err(RampError::UninitializedAccount.into());
    }

    // Check owner authorization
    if !owner_account.is_signer {
        msg!("Owner account must be signer");
        return Err(RampError::InvalidSigner.into());
    }

    if ramp_state.owner != *owner_account.key {
        msg!("Only owner can add assets");
        return Err(RampError::Unauthorized.into());
    }

    // Add the new assets with their fee percentages
    for (i, asset) in args.assets.iter().enumerate() {
        let fee_percentage = args.fee_percentages[i];
        
        // Validate fee percentage (e.g., max 10000 basis points = 100%)
        if fee_percentage > 10000 {
            msg!("Fee percentage too high: {}", fee_percentage);
            return Err(RampError::InvalidFeePercentage.into());
        }

        match ramp_state.add_asset(*asset, fee_percentage) {
            Ok(()) => msg!("Added asset: {}", asset),
            Err(_) => {
                msg!("Asset already exists: {}", asset);
                return Err(RampError::AssetAlreadyExists.into());
            }
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
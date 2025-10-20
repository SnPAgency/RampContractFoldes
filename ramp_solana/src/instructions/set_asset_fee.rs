use crate::{errors::RampError, state::RampState};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, 
    msg, 
    pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SetAssetFeeInstruction {
    pub asset_mint: Pubkey,
    pub fee_percentage: u128,
}

pub fn set_asset_fee(_program_id: &Pubkey, accounts: &[AccountInfo], args: SetAssetFeeInstruction) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;
    let owner_account = next_account_info(account_info_iter)?;    
    let mut ramp_state: RampState = {
        let ramp_data = ramp_account.try_borrow_data()?;
        borsh::from_slice(&ramp_data)?
    };
    if !owner_account.is_signer && &ramp_state.owner != owner_account.key && args.fee_percentage < 100 {
        return Err(RampError::InvalidSigner.into());
    }
    match ramp_state.get_asset_info(&args.asset_mint) {
        Some(asset) => {
            asset.asset_fee_percentage = args.fee_percentage;
        },
        None => {
            return Err(RampError::AssetNotFound.into());
        }
    }
    let mut ramp_data = ramp_account.try_borrow_mut_data()?;
    ramp_data.fill(0);
    let serialized_data = borsh::to_vec(&ramp_state).expect("Failed to serialize ramp state");
    
    if serialized_data.len() > ramp_data.len() {
        return Err(RampError::InvalidAccountState.into());
    }
    ramp_data[..serialized_data.len()].copy_from_slice(&serialized_data);    
    msg!("Asset fee set to {} for {}", args.fee_percentage, args.asset_mint);
    Ok(())
}
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

pub fn set_asset_fee(
    _program_id: &Pubkey, 
    accounts: &[AccountInfo], 
    args: SetAssetFeeInstruction
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;
    let owner_account = next_account_info(account_info_iter)?;    
    let mut ramp_state: RampState = {
        let ramp_data = ramp_account.try_borrow_data()?;
        borsh::from_slice(&ramp_data)?
    };
    let (owner, signer, fee) = (
        owner_account.key == &ramp_state.owner,
        owner_account.is_signer,
        args.fee_percentage < 100
    );
    match (owner, signer, fee) {
        (true, true, true) => {
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
        },
        (true, false, _) => {
            return Err(RampError::InvalidSigner.into());
        },
        (false, _, _) => {
            return Err(RampError::Unauthorized.into());
        }
        (true, true, false) => {
            return Err(RampError::InvalidFeePercentage.into());
        }
    }

}
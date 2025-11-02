use crate::{errors::RampError, state::RampState};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, 
    msg, 
    pubkey::Pubkey,
    program::invoke,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token_interface::instruction as token_instruction;


#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct OnRampWithdrawInstruction {
    pub amount: u64,
}

pub fn onramp_withdraw(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: OnRampWithdrawInstruction
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let ramp_account = next_account_info(account_info_iter)?;
    let asset_mint_account = next_account_info(account_info_iter)?;
    let ramp_owner = next_account_info(account_info_iter)?;
    let asset_receiver_token_account = next_account_info(account_info_iter)?;
    let ramp_token_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    let ramp_state: RampState = {
        let ramp_data = ramp_account.try_borrow_data()?;
        RampState::try_from_slice(&ramp_data)?
    };

    if !ramp_state.is_active && ramp_owner.key != &ramp_state.owner {
        return Err(RampError::UninitializedAccount.into());
    }

    match ramp_state.get_asset_info_ref(asset_mint_account.key) {
        Some(_asset) => {
            let ramp_associated_token_account = get_associated_token_address(
                ramp_account.key,
                asset_mint_account.key,
            );
        
            let transfer_instructions = token_instruction::transfer(
                token_program.key,
                &ramp_associated_token_account,
                &asset_receiver_token_account.key,
                ramp_account.key,
                &[ramp_account.key],
                args.amount,
            )?;
        
            let transfer_result = invoke(
                &transfer_instructions,
                &[
                    ramp_token_account.clone(),
                    asset_receiver_token_account.clone(),
                    ramp_account.clone(),
                    token_program.clone(),
                ],
            );
        
            if transfer_result.is_err() {
                return Err(RampError::TransferFailed.into());
            }
        },
        None => {
            return Err(RampError::AssetNotFound.into());
        }
    }
    msg!("On-ramp withdraw completed successfully");    
    Ok(())
}


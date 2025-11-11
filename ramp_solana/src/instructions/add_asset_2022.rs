use crate::{errors::RampError, instructions::AddAssetsInstruction, state::RampState};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, 
    msg, 
    pubkey::Pubkey,
    program::invoke,
};
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token_2022_interface::instruction::transfer_checked;

pub fn add_assets_2022(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: AddAssetsInstruction
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    //recepient account
    let ramp_account = next_account_info(account_info_iter)?;
    //asset account
    let asset_mint_account = next_account_info(account_info_iter)?;
    //sender
    let owner_account = next_account_info(account_info_iter)?;
    //token program
    let token_program = next_account_info(account_info_iter)?;
    // system program (needed for creating associated token account)
    let system_program = next_account_info(account_info_iter)?;
    // associated token account program
    let associated_token_program = next_account_info(account_info_iter)?;
    // owner's associated token account
    let owner_token_account = next_account_info(account_info_iter)?;
    // ramp's associated token account (will be created if doesn't exist)
    let ramp_token_account = next_account_info(account_info_iter)?;

    if args.fee_percentage > 100 {
        return Err(RampError::InvalidFeePercentage.into());
    }
    let mut ramp_state: RampState = {
        let ramp_data = ramp_account.try_borrow_data()?;
        borsh::from_slice(&ramp_data)?
    };

    if  !ramp_state.is_active {
        return Err(RampError::UninitializedAccount.into());
    }
    if owner_account.key != &ramp_state.owner {
        return Err(RampError::Unauthorized.into());
    }
    if ramp_token_account.lamports() == 0 {
        let account_instructions = create_associated_token_account(
            owner_account.key,
            ramp_account.key,
            asset_mint_account.key,
            token_program.key,
        );
        invoke(
            &account_instructions,
            &[
                owner_account.clone(),
                ramp_token_account.clone(),
                ramp_account.clone(),
                asset_mint_account.clone(),
                system_program.clone(),
                token_program.clone(),
                associated_token_program.clone(),
            ],
        )?;
    }
    let transfer_instructions = transfer_checked(
        token_program.key,
        owner_token_account.key,
        ramp_token_account.key,
        owner_account.key,
        &owner_account.key,
        &[owner_account.key],
        args.initial_amount,
        9,
    )?;
    let transfer_result = invoke(
        &transfer_instructions,
        &[
            owner_token_account.clone(),
            ramp_token_account.clone(),
            owner_account.clone(),
            token_program.clone(),
        ],
    );
    if transfer_result.is_err() {
        return Err(RampError::TransferFailed.into());
    }
    match ramp_state.add_asset(*asset_mint_account.key, args.fee_percentage) {
        Ok(()) => {
            let mut ramp_data = ramp_account.try_borrow_mut_data()?;
            ramp_data.fill(0);
            let serialized_data = borsh::to_vec(&ramp_state).expect("Failed to serialize ramp state");
            if serialized_data.len() > ramp_data.len() {
                return Err(RampError::InvalidAccountState.into());
            }
            ramp_data[..serialized_data.len()].copy_from_slice(&serialized_data);    
            msg!("Assets added successfully");
        },
        Err(_) => {
            return Err(RampError::AssetAlreadyExists.into());
        }
    }
    Ok(())
}
use crate::{errors::RampError, state::RampState};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, 
    msg, 
    pubkey::Pubkey,
    program::invoke,
};
use spl_associated_token_account::{
    get_associated_token_address,
    get_associated_token_address_with_program_id,
    instruction::create_associated_token_account
}; 
use spl_token_interface::instruction::transfer;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AddAssetsInstruction {
    pub initial_amount: u64,
    pub fee_percentage: u128,
}

pub fn add_assets(
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

    // First, read and verify the ramp state
    let mut ramp_state: RampState = {
        let ramp_data = ramp_account.try_borrow_data()?;
        borsh::from_slice(&ramp_data)?
    };

    if  !ramp_state.is_active && !owner_account.is_signer {
        return Err(RampError::UninitializedAccount.into());
    }
    
    //ramp asset account
    let ramp_associated_token_address = get_associated_token_address_with_program_id(
        ramp_account.key,
        asset_mint_account.key,
        &spl_token_interface::ID,
    );
    //sender asset account
    let owner_associated_token_address = get_associated_token_address_with_program_id(
        owner_account.key,
        asset_mint_account.key,
        &spl_token_interface::ID,
    );
    if args.fee_percentage > 10000 {
        return Err(RampError::InvalidFeePercentage.into());
    }
    
    //if owner_token_account.key != &owner_associated_token_address {
    //    return Err(RampError::InvalidAccountState.into());
    //}
    //
    //if ramp_token_account.key != &ramp_associated_token_address {
    //    return Err(RampError::InvalidAccountState.into());
    //}
    
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

    let transfer_instructions = transfer(
        token_program.key,
        owner_token_account.key,
        ramp_token_account.key,
        owner_account.key,
        &[owner_account.key],
        args.initial_amount,
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
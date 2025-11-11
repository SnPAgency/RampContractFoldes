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
//use spl_token_2022_interface::{
//    extension::{
//        //BaseStateWithExtensions,
//        //StateWithExtensions,
//        //metadata_pointer::MetadataPointer
//    },
//    state::{
//        //Account,
//       // Mint
//    }
//};
use spl_token_interface::instruction as token_instruction;
use crate::models::RampDeposit;
use base64::{engine::general_purpose, Engine as _};
//use spl_token_metadata_interface::state::TokenMetadata;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct OffRampDepositInstruction {
    pub amount: u64,
    pub region: Region,
    pub medium: Medium,
    pub data: Vec<u8>,
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
    let ramp_token_account = next_account_info(account_info_iter)?;
    //let metadata_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    let mut ramp_state: RampState = {
        let ramp_data = ramp_account.try_borrow_data()?;
        borsh::from_slice(&ramp_data)?
    };
    if !ramp_state.is_active {
        return Err(RampError::ProgramNotActive.into());
    }

    let ramp_associated_token_account = get_associated_token_address(
        ramp_account.key,
        asset_mint_account.clone().key,
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
            asset_owner_token_account.clone(),
            ramp_token_account.clone(),
            asset_owner_account.clone(),
            token_program.clone(),
        ],
    );

    if transfer_result.is_err() {
        return Err(RampError::TransferFailed.into());
    }
    match ramp_state.get_asset_info(asset_mint_account.clone().key) {
        Some(asset) => {
            let revenue = (args.amount as u128) * (asset.get_fee_percentage() / 100);
            asset.add_revenue(revenue);
        },
        None => {
            return Err(RampError::AssetNotFound.into());
        }
    }

    let mut ramp_data = ramp_account.try_borrow_mut_data()?;

    ramp_state.serialize(&mut ramp_data.as_mut())?;

    //let metadata_account_data = metadata_account.try_borrow_data()?;

    //let metadata: TokenMetadata = borsh::from_slice(&metadata_account_data)?;
    
    msg!("RampDeposit:{}", general_purpose::STANDARD.encode(
        borsh::to_vec(&RampDeposit {
            asset: *asset_mint_account.key,
            asset_name: "".to_string(),
            amount: args.amount,
            sender: *asset_owner_account.key,
            region: args.region,
            medium: args.medium,
            data: args.data,
        }).unwrap()
    ));    
    Ok(())
}
use {
    crate::instructions::{
            add_assets,
            remove_assets,
            set_active,
            set_owner,
            initialize_program,
            AddAssetsInstruction,
            RemoveAssetsInstruction,
            SetOwnerInstruction
        },
        borsh::{BorshDeserialize, BorshSerialize},
        solana_program::{
            account_info::AccountInfo,
            entrypoint::ProgramResult,
            pubkey::Pubkey
        },
    };
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct RampInstruction {
    pub instruction: Instruction,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum Instruction {
    SetActive,
    RemoveAssets(RemoveAssetsInstruction),
    SetOwner(SetOwnerInstruction),
    AddAssets(AddAssetsInstruction),
    InitializeProgram,
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = RampInstruction::try_from_slice(instruction_data)?;

    match instruction.instruction {
        Instruction::SetActive => set_active::set_active(program_id, accounts),
        Instruction::RemoveAssets( args) => remove_assets::remove_assets(program_id, accounts, args),
        Instruction::SetOwner( args) => set_owner::set_owner(program_id, accounts, args),
        Instruction::AddAssets( args) => add_assets::add_assets(program_id, accounts, args),
        Instruction::InitializeProgram => initialize_program::initialize_program(program_id, accounts),
    }
}
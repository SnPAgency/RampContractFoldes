use {
    crate::instructions::{
            add_assets,
            remove_assets,
            set_active,
            set_owner,
            initialize_program,
            token_transfers,
            AddAssetsInstruction,
            RemoveAssetsInstruction,
            SetOwnerInstruction,
            SetActiveInstruction,
            InitializeProgramInstruction,
            TransferTokensArgs
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
    SetActive(SetActiveInstruction),
    RemoveAssets(RemoveAssetsInstruction),
    SetOwner(SetOwnerInstruction),
    AddAssets(AddAssetsInstruction),
    InitializeProgram(InitializeProgramInstruction),
    TransferTokens(TransferTokensArgs),
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = RampInstruction::try_from_slice(instruction_data)?;

    match instruction.instruction {
        Instruction::SetActive(args) => set_active::set_active(program_id, accounts, args),
        Instruction::RemoveAssets(args) => remove_assets::remove_assets(program_id, accounts, args),
        Instruction::SetOwner(args) => set_owner::set_owner(program_id, accounts, args),
        Instruction::AddAssets(args) => add_assets::add_assets(program_id, accounts, args),
        Instruction::InitializeProgram(args) => initialize_program::initialize_program(program_id, accounts, args),
        Instruction::TransferTokens(args) => token_transfers::transfer_tokens(accounts, args),
    }
}
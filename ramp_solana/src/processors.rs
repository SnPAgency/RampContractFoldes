use {
    crate::instructions::{
            add_assets,
            initialize_program,
            off_ramp_deposit,
            off_ramp_deposit_native,
            onramp_withdraw,
            onramp_withdraw_native,
            remove_assets,
            set_active,
            set_asset_fee,
            set_native_fee_percentage,
            set_owner,
            off_ramp_deposit_token22,
            AddAssetsInstruction,
            InitializeProgramInstruction,
            OffRampDepositInstruction,
            OffRampDepositNativeInstruction,
            OnRampWithdrawInstruction,
            OnRampWithdrawNativeInstruction,
            RemoveAssetsInstruction,
            SetActiveInstruction,
            SetAssetFeeInstruction,
            SetNativeFeePercentageInstruction,
            SetOwnerInstruction,
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
    OffRampDepositToken22(OffRampDepositInstruction),
    OffRampDeposit(OffRampDepositInstruction),
    OnRampWithdraw(OnRampWithdrawInstruction),
    OffRampDepositNative(OffRampDepositNativeInstruction),
    OnRampWithdrawNative(OnRampWithdrawNativeInstruction),
    SetNativeFeePercentage(SetNativeFeePercentageInstruction),
    SetAssetFee(SetAssetFeeInstruction)
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = RampInstruction::try_from_slice(instruction_data)?;

    match instruction.instruction {
        Instruction::SetActive(args) => {
            set_active::set_active(
                program_id,
                accounts,
                args
            )
        },
        Instruction::RemoveAssets(args) => {
            remove_assets::remove_assets(
                program_id,
                accounts,
                args
            )
        },
        Instruction::SetOwner(args) => {
            set_owner::set_owner(
                program_id,
                accounts,
                args
            )
        },
        Instruction::AddAssets(args) => {
            add_assets::add_assets(
                program_id,
                accounts,
                args
            )
        },
        Instruction::InitializeProgram(args) => {
            initialize_program::initialize_program(
                program_id,
                accounts,
                args
            )
        },
        Instruction::OffRampDepositToken22(args) => {
            off_ramp_deposit_token22::off_ramp_deposit_token_22(
                program_id,
                accounts,
                args
            )
        },
        Instruction::OffRampDeposit(args) => {
            off_ramp_deposit::off_ramp_deposit(
                program_id,
                accounts,
                args
            )
        },
        Instruction::OnRampWithdraw(args) => {
            onramp_withdraw::onramp_withdraw(
                program_id,
                accounts,
                args
            )
        },
        Instruction::OffRampDepositNative(args) => {
            off_ramp_deposit_native::off_ramp_deposit_native(
                program_id,
                accounts,
                args
            )
        },
        Instruction::OnRampWithdrawNative(args) => {
            onramp_withdraw_native::onramp_withdraw_native(
                program_id,
                accounts,
                args
            )
        },
        Instruction::SetNativeFeePercentage(args) => {
            set_native_fee_percentage::set_native_fee_percentage(
                program_id,
                accounts,
                args
            )
        },
        Instruction::SetAssetFee(args) => {
            set_asset_fee::set_asset_fee(
                program_id,
                accounts,
                args
            )
        }
    }
}
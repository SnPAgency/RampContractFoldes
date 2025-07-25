mod state;
mod errors;
mod instructions;
mod processors;

use solana_program::{
    pubkey::Pubkey,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    entrypoint,
};

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    processors::process_instruction(program_id, accounts, instruction_data)
}

#[cfg(test)]
mod tests {
    use crate::state::RampState;

    use super::*;
    use solana_program_test::*;
    use solana_sdk::{
        instruction::{
            AccountMeta, Instruction
        },
        signature::{
            Keypair,
            Signer
        },
        system_program,
        transaction::Transaction,
    };
    use borsh::BorshDeserialize;
    
    #[tokio::test]
    async fn test_program_initialization() {
        let program_id = Pubkey::new_unique();

        let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
            "ramp_solana",
            program_id,
            processor!(process_instruction),
        ).start().await;

        let ramp_keypair = Keypair::new();

        println!("Testing program initialization...");

        // Instruction::new_with_bytes(program_id, &vec![4], accounts);
        let init_instructions = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::InitializeProgram,
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), true),
                AccountMeta::new(payer.pubkey(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );
//
        let mut transaction = Transaction::new_with_payer(&[init_instructions], Some(&payer.pubkey()));

        transaction.sign(&[&payer, &ramp_keypair], recent_blockhash);

        let result = banks_client.process_transaction(transaction).await;

        if result.is_err() {
            println!("Error during program initialization: {:?}", result.err());
            panic!("Program initialization failed!");
        }
        println!("Program initialized successfully!");

        let ramp_account = banks_client.get_account(ramp_keypair.pubkey()).await.expect("Ramp account not found!");
        assert!(ramp_account.is_some(), "Ramp account should be initialized");

        // Check the account data
        if let Some(account) = ramp_account {
            let data_size = account.data.len();

            println!("Ramp account data size: {}", data_size);
            let mut data_slice: &[u8] = &account.data;
            let ramp_state: RampState = RampState::deserialize(&mut data_slice)
                .expect("Failed to deserialize RampState");

            assert!(ramp_state.is_initialized, "RampState should be initialized");
            assert_eq!(ramp_state.owner, payer.pubkey(), "RampState owner should match payer");
        } else {
            panic!("Ramp account data is empty!");
        }

    }
}
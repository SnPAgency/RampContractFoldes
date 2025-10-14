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
    use crate::{
        state::RampState,
        processors,
        instructions::*,
    };

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
    
    async fn setup_program() -> (BanksClient, Keypair, solana_sdk::hash::Hash, Pubkey, Keypair) {
        let program_id = Pubkey::new_unique();
        let (banks_client, payer, recent_blockhash) = ProgramTest::new(
            "ramp_solana",
            program_id,
            processor!(process_instruction),
        ).start().await;
        
        let ramp_keypair = Keypair::new();
        (banks_client, payer, recent_blockhash, program_id, ramp_keypair)
    }
    
    async fn initialize_ramp_program(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: solana_sdk::hash::Hash,
        program_id: Pubkey,
        ramp_keypair: &Keypair,
        max_assets: usize,
        vault_address: Pubkey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let init_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::InitializeProgram(
                    InitializeProgramInstruction {
                        max_assets,
                        vault_address,
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), true),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
        transaction.sign(&[payer, ramp_keypair], recent_blockhash);
        
        banks_client.process_transaction(transaction).await?;
        Ok(())
    }
    
    async fn get_ramp_state(
        banks_client: &mut BanksClient,
        ramp_pubkey: Pubkey,
    ) -> Result<RampState, Box<dyn std::error::Error>> {
        let account = banks_client.get_account(ramp_pubkey).await?
            .ok_or("Ramp account not found")?;
        
        let mut data_slice: &[u8] = &account.data;
        let ramp_state = RampState::deserialize(&mut data_slice)?;
        Ok(ramp_state)
    }

    #[tokio::test]
    async fn test_program_initialization() {
        let (mut banks_client, payer, recent_blockhash, program_id, ramp_keypair) = setup_program().await;
        let vault_address = Pubkey::new_unique();

        println!("Testing program initialization...");

        // Test successful initialization
        let result = initialize_ramp_program(
            &mut banks_client,
            &payer,
            recent_blockhash,
            program_id,
            &ramp_keypair,
            10,
            vault_address,
        ).await;

        assert!(result.is_ok(), "Program initialization should succeed");
        println!("Program initialized successfully!");

        // Verify the state
        let ramp_state = get_ramp_state(&mut banks_client, ramp_keypair.pubkey()).await
            .expect("Failed to get ramp state");

        assert_eq!(ramp_state.owner, payer.pubkey(), "RampState owner should match payer");
        assert_eq!(ramp_state.vault_address, vault_address, "Vault address should match");
        assert!(!ramp_state.is_active, "RampState should not be active initially");
    }

    #[tokio::test]
    async fn test_initialization_invalid_max_assets() {
        let (mut banks_client, payer, recent_blockhash, program_id, ramp_keypair) = setup_program().await;
        let vault_address = Pubkey::new_unique();

        println!("Testing initialization with invalid max_assets...");

        // Test with 0 max_assets (should fail)
        let result = initialize_ramp_program(
            &mut banks_client,
            &payer,
            recent_blockhash,
            program_id,
            &ramp_keypair,
            0,
            vault_address,
        ).await;

        assert!(result.is_err(), "Initialization with 0 max_assets should fail");
        println!("Correctly rejected invalid max_assets");
    }

    #[tokio::test]
    async fn test_add_assets() {
        let (mut banks_client, payer, recent_blockhash, program_id, ramp_keypair) = setup_program().await;
        let vault_address = Pubkey::new_unique();

        // Initialize the program first
        initialize_ramp_program(
            &mut banks_client,
            &payer,
            recent_blockhash,
            program_id,
            &ramp_keypair,
            10,
            vault_address,
        ).await.expect("Initialization should succeed");

        println!("Testing add assets...");

        let asset = Pubkey::new_unique();
        let fee_percentage = 100;

        let add_assets_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::AddAssets(
                    AddAssetsInstruction {
                        asset,
                        fee_percentage,
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true), // Owner must sign
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[add_assets_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        
        let result = banks_client.process_transaction(transaction).await;
        assert!(result.is_ok(), "Add assets should succeed: {:?}", result.err());

        // Verify the state
        let ramp_state = get_ramp_state(&mut banks_client, ramp_keypair.pubkey()).await
            .expect("Failed to get ramp state");

        let assets = ramp_state.get_assets();
        assert!(assets.contains(&asset), "Should contain asset");
        
        // Check asset info
        let asset_info = ramp_state.get_asset_info(&asset).expect("Asset info should exist");
        assert_eq!(asset_info.asset_fee_percentage, 100, "Asset fee should be 100");
        
        println!("Add assets test passed!");
    }

    #[tokio::test]
    async fn test_add_assets_unauthorized() {
        let (mut banks_client, payer, recent_blockhash, program_id, ramp_keypair) = setup_program().await;
        let vault_address = Pubkey::new_unique();
        let unauthorized_user = Keypair::new();

        // Initialize the program first
        initialize_ramp_program(
            &mut banks_client,
            &payer,
            recent_blockhash,
            program_id,
            &ramp_keypair,
            10,
            vault_address,
        ).await.expect("Initialization should succeed");

        println!("Testing unauthorized add assets...");

        let asset = Pubkey::new_unique();
        let fee_percentage = 100;

        let add_assets_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::AddAssets(
                    AddAssetsInstruction {
                        asset,
                        fee_percentage,
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), false),
                AccountMeta::new(unauthorized_user.pubkey(), true), // Wrong owner
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[add_assets_instruction], Some(&unauthorized_user.pubkey()));
        transaction.sign(&[&unauthorized_user], recent_blockhash);
        
        let result = banks_client.process_transaction(transaction).await;
        assert!(result.is_err(), "Unauthorized add assets should fail");
        println!("Correctly rejected unauthorized add assets");
    }

    #[tokio::test]
    async fn test_remove_assets() {
        let (mut banks_client, payer, recent_blockhash, program_id, ramp_keypair) = setup_program().await;
        let vault_address = Pubkey::new_unique();

        // Initialize and add assets first
        initialize_ramp_program(
            &mut banks_client,
            &payer,
            recent_blockhash,
            program_id,
            &ramp_keypair,
            10,
            vault_address,
        ).await.expect("Initialization should succeed");

        let asset = Pubkey::new_unique();
        let fee_percentage = 100;

        // Add assets first
        let add_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::AddAssets(
                    AddAssetsInstruction {
                        asset,
                        fee_percentage,
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true),
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[add_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.expect("Add assets should succeed");

        println!("Testing remove assets...");

        // Now remove one asset
        let remove_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::RemoveAssets(
                    RemoveAssetsInstruction {
                        asset,
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true),
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[remove_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        
        let result = banks_client.process_transaction(transaction).await;
        assert!(result.is_ok(), "Remove assets should succeed: {:?}", result.err());

        // Verify the state
        let ramp_state = get_ramp_state(&mut banks_client, ramp_keypair.pubkey()).await
            .expect("Failed to get ramp state");

        let assets = ramp_state.get_assets();
        assert!(!assets.contains(&asset), "Should not contain removed asset1");
        assert!(ramp_state.get_asset_info(&asset).is_none(), "Asset1 info should be removed");
        
        println!("Remove assets test passed!");
    }

    #[tokio::test]
    async fn test_set_active() {
        let (mut banks_client, payer, recent_blockhash, program_id, ramp_keypair) = setup_program().await;
        let vault_address = Pubkey::new_unique();

        // Initialize the program first
        initialize_ramp_program(
            &mut banks_client,
            &payer,
            recent_blockhash,
            program_id,
            &ramp_keypair,
            10,
            vault_address,
        ).await.expect("Initialization should succeed");

        println!("Testing set active...");

        // Set active to true
        let set_active_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::SetActive(
                    SetActiveInstruction {
                        is_active: true,
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true),
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[set_active_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        
        let result = banks_client.process_transaction(transaction).await;
        assert!(result.is_ok(), "Set active should succeed: {:?}", result.err());

        // Verify the state
        let ramp_state = get_ramp_state(&mut banks_client, ramp_keypair.pubkey()).await
            .expect("Failed to get ramp state");

        assert!(ramp_state.is_active, "RampState should be active");
        println!("Set active test passed!");
    }

    #[tokio::test]
    async fn test_set_owner() {
        let (mut banks_client, payer, recent_blockhash, program_id, ramp_keypair) = setup_program().await;
        let vault_address = Pubkey::new_unique();
        let new_owner = Keypair::new();

        // Initialize the program first
        initialize_ramp_program(
            &mut banks_client,
            &payer,
            recent_blockhash,
            program_id,
            &ramp_keypair,
            10,
            vault_address,
        ).await.expect("Initialization should succeed");

        println!("Testing set owner...");

        let set_owner_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::SetOwner(
                    SetOwnerInstruction {
                        new_owner: new_owner.pubkey(),
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true), // Current owner must sign
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[set_owner_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        
        let result = banks_client.process_transaction(transaction).await;

        assert!(result.is_ok(), "Set owner should succeed: {:?}", result.err());

        // Verify the state
        let ramp_state = get_ramp_state(&mut banks_client, ramp_keypair.pubkey()).await
            .expect("Failed to get ramp state");

        assert_eq!(ramp_state.owner, new_owner.pubkey(), "Owner should be updated");
        println!("Set owner test passed!");
    }

    #[tokio::test]
    async fn test_add_assets_invalid_fee() {
        let (mut banks_client, payer, recent_blockhash, program_id, ramp_keypair) = setup_program().await;
        let vault_address = Pubkey::new_unique();

        // Initialize the program first
        initialize_ramp_program(
            &mut banks_client,
            &payer,
            recent_blockhash,
            program_id,
            &ramp_keypair,
            10,
            vault_address,
        ).await.expect("Initialization should succeed");

        println!("Testing add assets with invalid fee...");

        let asset = Pubkey::new_unique();
        let fee_percentage = 15000; // > 10000 basis points (100%)

        let add_assets_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::AddAssets(
                    AddAssetsInstruction {
                        asset,
                        fee_percentage,
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true),
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[add_assets_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        
        let result = banks_client.process_transaction(transaction).await;
        assert!(result.is_err(), "Add assets with invalid fee should fail");
        println!("Correctly rejected invalid fee percentage");
    }

    #[tokio::test]
    async fn test_comprehensive_workflow() {
        let (mut banks_client, payer, recent_blockhash, program_id, ramp_keypair) = setup_program().await;
        let vault_address = Pubkey::new_unique();

        println!("Testing comprehensive workflow...");

        // 1. Initialize
        initialize_ramp_program(
            &mut banks_client,
            &payer,
            recent_blockhash,
            program_id,
            &ramp_keypair,
            5,
            vault_address,
        ).await.expect("Initialization should succeed");

        // 2. Activate the program
        let activate_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::SetActive(
                    SetActiveInstruction {
                        is_active: true,
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true),
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[activate_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.expect("Activate should succeed");

        // 3. Add assets
        let asset = Pubkey::new_unique();
        let add_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::AddAssets(
                    AddAssetsInstruction {
                        asset,
                        fee_percentage: 100,
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true),
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[add_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.expect("Add assets should succeed");

        // 4. Verify final state
        let ramp_state = get_ramp_state(&mut banks_client, ramp_keypair.pubkey()).await
            .expect("Failed to get ramp state");

        assert!(ramp_state.is_active, "Should be active");
        assert!(ramp_state.get_assets().contains(&asset), "Should have assets");
        assert_eq!(ramp_state.owner, payer.pubkey(), "Owner should be payer");
        assert_eq!(ramp_state.vault_address, vault_address, "Vault address should match");

        println!("Comprehensive workflow test passed!");
    }
}
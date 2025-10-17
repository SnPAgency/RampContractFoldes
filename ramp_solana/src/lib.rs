mod state;
mod errors;
mod instructions;
mod processors;
mod models;

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
        instructions::*, processors, state::RampState
    };

    use super::*;
    use solana_program_test::*;
    use solana_sdk::{
        instruction::{
            AccountMeta, Instruction
        },
        program_pack::Pack, signature::{
            Keypair,
            Signer
        }, system_program, transaction::Transaction
    };
    use spl_associated_token_account::get_associated_token_address;
    use spl_token;
    use borsh::BorshDeserialize;
    

    /**
     * Setup the program for testing
     * 
     * @returns (program_test_context, program_id, asset_mint)
     */
    async fn setup_program() -> (ProgramTestContext, Pubkey, Keypair, Keypair) {
        let program_id = Pubkey::new_unique();
        let program_test = ProgramTest::new(
            "ramp_solana",
            program_id,
            processor!(process_instruction),
        );

        let mut program_ctx = program_test.start_with_context().await;

        let asset_mint = Keypair::new();
        let rent = program_ctx.banks_client.get_rent().await.unwrap();
        let mint_rent = rent.minimum_balance(spl_token::state::Mint::LEN);

        let mint_account_instructions = solana_sdk::system_instruction::create_account(
            &program_ctx.payer.pubkey(),
            &asset_mint.pubkey(),
            mint_rent,
            spl_token::state::Mint::LEN as u64,
            &spl_token::id(),
        );

        let mint_init_tx = spl_token::instruction::initialize_mint(
            &spl_token::ID,
            &asset_mint.pubkey(),
            &program_ctx.payer.pubkey(),
            Some(&program_ctx.payer.pubkey()),
            9,
        ).unwrap();

        let asset_mint_tx = Transaction::new_signed_with_payer(
            &[mint_account_instructions, mint_init_tx],
            Some(&program_ctx.payer.pubkey()),
            &[&program_ctx.payer, &asset_mint],
            program_ctx.banks_client.get_latest_blockhash().await.unwrap(),
        );

        program_ctx.banks_client.process_transaction(asset_mint_tx).await.unwrap();

        let ramp_keypair = Keypair::new();

        // Create associated token account for the payer (owner) to hold tokens
        let create_associated_token_account_instructions = spl_associated_token_account::instruction::create_associated_token_account(
            &program_ctx.payer.pubkey(),
            &program_ctx.payer.pubkey(),
            &asset_mint.pubkey(),
            &spl_token::id(),
        );

        let mut create_associated_token_account_transaction = Transaction::new_with_payer(
            &[create_associated_token_account_instructions],
            Some(&program_ctx.payer.pubkey())
        );

        create_associated_token_account_transaction.sign(&[&program_ctx.payer], program_ctx.banks_client.get_latest_blockhash().await.unwrap());

        let create_associated_token_account_result = program_ctx.banks_client.process_transaction(create_associated_token_account_transaction).await;
        assert!(create_associated_token_account_result.is_ok(), "{:?}", create_associated_token_account_result.err().unwrap());

        // Mint tokens to the payer's token account
        let mint_instructions = spl_token::instruction::mint_to(
            &spl_token::ID,
            &asset_mint.pubkey(),
            &get_associated_token_address(&program_ctx.payer.pubkey(), &asset_mint.pubkey()),
            &program_ctx.payer.pubkey(),
            &[&program_ctx.payer.pubkey()],
            1000000
        ).unwrap();

        let mut mint_transaction = Transaction::new_with_payer(&[mint_instructions], Some(&program_ctx.payer.pubkey()));
        mint_transaction.sign(&[&program_ctx.payer], program_ctx.banks_client.get_latest_blockhash().await.unwrap());

        let mint_result = program_ctx.banks_client.process_transaction(mint_transaction).await;
        assert!(mint_result.is_ok(), "{:?}", mint_result.err().unwrap());

        (program_ctx, program_id, asset_mint, ramp_keypair)
    }
    
    /**
     * Initialize the ramp program
     * 
     * @param banks_client - The banks client
     * @param payer - The payer
     * @param recent_blockhash - The recent blockhash
     * @param program_id - The program id
     * @param ramp_keypair - The ramp keypair
     * @param max_assets - The max assets
     * @param vault_address - The vault address
     * @returns Result<(), Box<dyn std::error::Error>>
     */
    async fn initialize_ramp_program(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: solana_sdk::hash::Hash,
        program_id: Pubkey,
        ramp_keypair: &Keypair,
        vault_address: Pubkey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let init_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::InitializeProgram(
                    InitializeProgramInstruction {
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
    
    /**
     * Get the ramp state
     * 
     * @param banks_client - The banks client
     * @param ramp_pubkey - The ramp public key
     * @returns Result<RampState, Box<dyn std::error::Error>>
     */
    async fn get_ramp_state(
        banks_client: &mut BanksClient,
        ramp_pubkey: Pubkey,
    ) -> Result<RampState, Box<dyn std::error::Error>> {
        let account = banks_client.get_account(ramp_pubkey).await?
            .ok_or("Ramp account not found")?;
        
        let mut data_slice: &[u8] = &account.data;
        let ramp_state = RampState::try_from_slice(&mut data_slice)?;
        Ok(ramp_state)
    }


    /**
     * Tests
     */



    /**
     * Test program initialization
     */
    #[tokio::test]
    async fn test_program_initialization() {
        let (mut ctx, program_id, _asset_mint, ramp_keypair) = setup_program().await;
        //let owner = Keypair::new();
        let vault_address = Pubkey::new_unique();

        println!("Testing program initialization...");

        // Test successful initialization
        let result = initialize_ramp_program(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            program_id,
            &ramp_keypair,
            vault_address,
        ).await;

        println!("Program initialized successfully!");
        assert!(result.is_ok(), "{:?}", result.err().unwrap());

        // Verify the state
        let ramp_state = get_ramp_state(&mut ctx.banks_client, ramp_keypair.pubkey()).await
            .expect("Failed to get ramp state");

        assert_eq!(ramp_state.owner, ctx.payer.pubkey(), "RampState owner should match payer");
        assert_eq!(ramp_state.vault_address, vault_address, "Vault address should match");
        assert!(!ramp_state.is_active, "RampState should not be active initially");
    }


    /**
     * Test add assets
     */
    #[tokio::test]
    async fn test_add_assets() {
        let (mut ctx, program_id, asset_mint, ramp_keypair) = setup_program().await;
        let vault_address = Pubkey::new_unique();

        // Initialize the program first
        initialize_ramp_program(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            program_id,
            &ramp_keypair,
            vault_address,
        ).await.expect("Initialization should succeed");

        println!("Testing add assets...");
        
        let fee_percentage = 100;
        let initial_amount = 100;
        
        // Get associated token addresses
        let owner_token_account = get_associated_token_address(&ctx.payer.pubkey(), &asset_mint.pubkey());
        let ramp_token_account = get_associated_token_address(&ramp_keypair.pubkey(), &asset_mint.pubkey());

        let add_assets_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::AddAssets(
                    AddAssetsInstruction {
                        fee_percentage,
                        initial_amount,
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), false),
                AccountMeta::new(asset_mint.pubkey(), false),
                AccountMeta::new(ctx.payer.pubkey(), true), // Owner must sign
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(system_program::id(), false), // System program
                AccountMeta::new_readonly(spl_associated_token_account::id(), false), // Associated token program
                AccountMeta::new(owner_token_account, false), // Owner's token account
                AccountMeta::new(ramp_token_account, false), // Ramp's token account
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[add_assets_instruction], Some(&ctx.payer.pubkey()));
        transaction.sign(&[&ctx.payer], ctx.last_blockhash);
        
        let result = ctx.banks_client.process_transaction(transaction).await;
        assert!(result.is_ok(), "Add assets should succeed: {:?}", result.err());

        // Verify the state
        let ramp_state = get_ramp_state(&mut ctx.banks_client, ramp_keypair.pubkey()).await
            .expect("Failed to get ramp state");

        let assets = ramp_state.get_assets();
        assert!(assets.contains(&asset_mint.pubkey()), "Should contain asset");
        
        // Check asset info
        let asset_info = ramp_state.get_asset_info_ref(&asset_mint.pubkey()).expect("Asset info should exist");
        assert_eq!(asset_info.asset_fee_percentage, 100, "Asset fee should be 100");
        
        println!("Add assets test passed!");
    }

    /**
     * Test add assets unauthorized
     */
    #[tokio::test]
    async fn test_add_assets_unauthorized() {
        let (mut ctx, program_id, asset_mint, ramp_keypair) = setup_program().await;
        let vault_address = Pubkey::new_unique();
        let unauthorized_user = Keypair::new();

        // Initialize the program first
        initialize_ramp_program(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            program_id,
            &ramp_keypair,
            vault_address,
        ).await.expect("Initialization should succeed");

        println!("Testing unauthorized add assets...");

        let fee_percentage = 100;
        let initial_amount = 100;
        
        // Get associated token addresses
        let unauthorized_token_account = get_associated_token_address(&unauthorized_user.pubkey(), &asset_mint.pubkey());
        let ramp_token_account = get_associated_token_address(&ramp_keypair.pubkey(), &asset_mint.pubkey());

        let add_assets_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::AddAssets(
                    AddAssetsInstruction {
                        fee_percentage,
                        initial_amount,
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), false),
                AccountMeta::new(asset_mint.pubkey(), false),
                AccountMeta::new(unauthorized_user.pubkey(), true), // Wrong owner
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(system_program::id(), false), // System program
                AccountMeta::new_readonly(spl_associated_token_account::id(), false), // Associated token program
                AccountMeta::new(unauthorized_token_account, false), // Unauthorized user's token account
                AccountMeta::new(ramp_token_account, false), // Ramp's token account
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[add_assets_instruction], Some(&unauthorized_user.pubkey()));
        transaction.sign(&[&unauthorized_user], ctx.last_blockhash);
        
        let result = ctx.banks_client.process_transaction(transaction).await;
        assert!(result.is_err(), "Unauthorized add assets should fail");
        println!("Correctly rejected unauthorized add assets");
    }

    /**
     * Test remove assets
     */
    #[tokio::test]
    async fn test_remove_assets() {
        let (mut ctx, program_id, asset_mint, ramp_keypair) = setup_program().await;
        let vault_address = Pubkey::new_unique();

        // Initialize and add assets first
        initialize_ramp_program(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            program_id,
            &ramp_keypair,
            vault_address,
        ).await.expect("Initialization should succeed");

        println!("Testing add assets...");
        
        let fee_percentage = 100;
        let initial_amount = 100;
        
        // Get associated token addresses
        let owner_token_account = get_associated_token_address(&ctx.payer.pubkey(), &asset_mint.pubkey());
        let ramp_token_account = get_associated_token_address(&ramp_keypair.pubkey(), &asset_mint.pubkey());

        let add_assets_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::AddAssets(
                    AddAssetsInstruction {
                        fee_percentage,
                        initial_amount,
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), false),
                AccountMeta::new(asset_mint.pubkey(), false),
                AccountMeta::new(ctx.payer.pubkey(), true), // Owner must sign
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(system_program::id(), false), // System program
                AccountMeta::new_readonly(spl_associated_token_account::id(), false), // Associated token program
                AccountMeta::new(owner_token_account, false), // Owner's token account
                AccountMeta::new(ramp_token_account, false), // Ramp's token account
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[add_assets_instruction], Some(&ctx.payer.pubkey()));
        transaction.sign(&[&ctx.payer], ctx.last_blockhash);
        
        let result = ctx.banks_client.process_transaction(transaction).await;
        assert!(result.is_ok(), "Add assets should succeed: {:?}", result.err());

        println!("Testing remove assets...");

        // remove one asset
        let remove_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::RemoveAssets(
                    RemoveAssetsInstruction {
                        //asset: asset_mint.pubkey(),
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), true), // Ramp account needs to sign for token transfer
                AccountMeta::new(
                    get_associated_token_address(&ramp_keypair.pubkey(), &asset_mint.pubkey()),
                    false,
                ),
                AccountMeta::new_readonly(asset_mint.pubkey(), false),
                AccountMeta::new(ctx.payer.pubkey(), true), // Owner also signs
                AccountMeta::new(
                    get_associated_token_address(&ctx.payer.pubkey(), &asset_mint.pubkey()),
                    false,
                ),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
        );
        let mut transaction = Transaction::new_with_payer(
            &[remove_instruction],
            Some(&ctx.payer.pubkey())
        );
        transaction.sign(&[&ctx.payer, &ramp_keypair], ctx.last_blockhash);
        
        let result = ctx.banks_client.process_transaction(transaction).await;
        assert!(result.is_ok(), "Remove assets should succeed: {:?}", result.err());

        // Verify the state
        let ramp_state = get_ramp_state(&mut ctx.banks_client, ramp_keypair.pubkey()).await
            .expect("Failed to get ramp state");

        let assets = ramp_state.get_assets();
        assert!(!assets.contains(&asset_mint.pubkey()), "Should not contain removed asset1");
        assert!(ramp_state.get_asset_info_ref(&asset_mint.pubkey()).is_none(), "Asset1 info should be removed");
        println!("Remove assets test passed!");
    }

    /**
     * Test set active
     */
    #[tokio::test]
    async fn test_set_active() {
        let (mut ctx, program_id, _asset_mint, ramp_keypair) = setup_program().await;
        let vault_address = Pubkey::new_unique();

        // Initialize the program first
        initialize_ramp_program(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            program_id,
            &ramp_keypair,
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
                AccountMeta::new(ramp_keypair.pubkey(), true),
                AccountMeta::new(ctx.payer.pubkey(), true),
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[set_active_instruction], Some(&ctx.payer.pubkey()));
        transaction.sign(&[&ctx.payer, &ramp_keypair], ctx.last_blockhash);
        
        let result = ctx.banks_client.process_transaction(transaction).await;
        assert!(result.is_ok(), "Set active should succeed: {:?}", result.err());

        // Verify the state
        let ramp_state = get_ramp_state(&mut ctx.banks_client, ramp_keypair.pubkey()).await
            .expect("Failed to get ramp state");

        assert!(ramp_state.is_active, "RampState should be active");
        println!("Set active test passed!");
    }

    /**
     * Test set owner
     */
    #[tokio::test]
    async fn test_set_owner() {
        let (
            mut ctx,
            program_id,
            _asset_mint,
            ramp_keypair,
        ) = setup_program().await;

        let vault_address = Pubkey::new_unique();
        let new_owner = Keypair::new();

        // Initialize the program first
        initialize_ramp_program(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            program_id,
            &ramp_keypair,
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
                AccountMeta::new(ramp_keypair.pubkey(), false), // Ramp account doesn't need to sign
                AccountMeta::new(ctx.payer.pubkey(), true), // Current owner must sign
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[set_owner_instruction], Some(&ctx.payer.pubkey()));
        transaction.sign(&[&ctx.payer], ctx.last_blockhash);
        
        let result = ctx.banks_client.process_transaction(transaction).await;

        assert!(result.is_ok(), "Set owner should succeed: {:?}", result.err());

        // Verify the state
        let ramp_state = get_ramp_state(&mut ctx.banks_client, ramp_keypair.pubkey()).await
            .expect("Failed to get ramp state");

        assert_eq!(ramp_state.owner, new_owner.pubkey(), "Owner should be updated");
        println!("Set owner test passed!");
    }

    /**
     * Test add assets with invalid fee
     */
    #[tokio::test]
    async fn test_add_assets_invalid_fee() {
        let (mut ctx, program_id, asset_mint, ramp_keypair) = setup_program().await;
        let vault_address = Pubkey::new_unique();

        // Initialize the program first
        initialize_ramp_program(
            &mut ctx.banks_client,
            &ctx.payer,
            ctx.last_blockhash,
            program_id,
            &ramp_keypair,
            vault_address,
        ).await.expect("Initialization should succeed");

        println!("Testing add assets with invalid fee...");
        let fee_percentage = 15000; // > 10000 basis points (100%)
        let initial_amount = 100;

        // Get associated token addresses
        let owner_token_account = get_associated_token_address(&ctx.payer.pubkey(), &asset_mint.pubkey());
        let ramp_token_account = get_associated_token_address(&ramp_keypair.pubkey(), &asset_mint.pubkey());

        let add_assets_instruction = Instruction::new_with_borsh(
            program_id,
            &processors::RampInstruction {
                instruction: processors::Instruction::AddAssets(
                    AddAssetsInstruction {
                        fee_percentage,
                        initial_amount,
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_keypair.pubkey(), false),
                AccountMeta::new(asset_mint.pubkey(), false),
                AccountMeta::new(ctx.payer.pubkey(), true), // Owner must sign
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(system_program::id(), false), // System program
                AccountMeta::new_readonly(spl_associated_token_account::id(), false), // Associated token program
                AccountMeta::new(owner_token_account, false), // Owner's token account
                AccountMeta::new(ramp_token_account, false), // Ramp's token account
            ],
        );

        let mut transaction = Transaction::new_with_payer(&[add_assets_instruction], Some(&ctx.payer.pubkey()));
        transaction.sign(&[&ctx.payer], ctx.last_blockhash);
        
        let result = ctx.banks_client.process_transaction(transaction).await;
        assert!(result.is_err(), "Add assets with invalid fee should fail");
        println!("Correctly rejected invalid fee percentage");
    }
}
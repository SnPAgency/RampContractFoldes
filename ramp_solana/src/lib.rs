pub mod state;
pub mod errors;
pub mod instructions;
pub mod processors;
pub mod models;


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
mod test {
    use borsh::BorshDeserialize as _;
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_commitment_config::CommitmentConfig;
    use solana_program::example_mocks::solana_sdk::system_program;
    use solana_sdk::{
        message::{
            AccountMeta,
            Instruction
        }, pubkey::Pubkey, signature::{
            Keypair,
            Signer
        }, signer::EncodableKey, transaction::Transaction
    };
    use solana_system_interface::instruction::create_account;
    use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
    use spl_token_2022_interface::{
        extension::{
            metadata_pointer::{
                instruction::initialize as initialize_metadata_pointer, MetadataPointer,
            },
            BaseStateWithExtensions, ExtensionType, StateWithExtensions,
        },
        instruction::initialize_mint,
        state::Mint,
        ID as TOKEN_2022_PROGRAM_ID,
    };
    use spl_token_metadata_interface::{
        instruction::{initialize as initialize_token_metadata, update_field},
        state::{Field, TokenMetadata},

    };
    use crate::{instructions::{AddAssetsInstruction, InitializeProgramInstruction}, processors, state::RampState};
    use dotenv::dotenv;
    

    fn get_key(path: &str) -> Keypair {
        Keypair::read_from_file(path).unwrap_or_else(
            |_| {
                let key = Keypair::new();
                key.write_to_file(path).unwrap();
                key
            }
        )
    }

    async fn get_set_up() -> (RpcClient, Keypair, Keypair, Keypair, Keypair) {
        dotenv().ok();        
        let client = RpcClient::new_with_commitment(
            String::from("http://localhost:8899"),
            CommitmentConfig::confirmed(),
        );

        let fee_payer = get_key("keys/dev_keypair.json");

        let airdrop_signature = client
            .request_airdrop(&fee_payer.pubkey(), 50_000_000_000)
            .await.unwrap();

        loop {
            let confirmed = client.confirm_transaction(&airdrop_signature).await.unwrap();
            if confirmed {
                break;
            }
        }

        let ramp_program_id = get_key("target/deploy/ramp_solana-keypair.json");

        let mint = get_key("keys/token_keypair.json");

        let ramp_data_account = get_key("keys/ramp_data_account.json");

        let is_initialized: bool = std::env::var("IS_INITIALIZED").unwrap().parse().unwrap();

        println!("is_initialized: {}", is_initialized);
        if !is_initialized {
            
            init_ramp(&client, &fee_payer, &ramp_program_id, &ramp_data_account).await;
        }
        let is_token_setup: bool = std::env::var("IS_TOKEN_SETUP").unwrap().parse().unwrap();
        println!("is_token_setup: {}", is_token_setup);
        if !is_token_setup {
            setup_token(&client, &fee_payer, &mint).await;
        }

        (client, fee_payer, ramp_program_id, mint, ramp_data_account)
    }


    async fn setup_token(
        client: &RpcClient,
        fee_payer: &Keypair,
        //ramp_program_id: Keypair,
        mint: &Keypair
    ){

        let latest_blockhash = client.get_latest_blockhash().await.unwrap();

        // Define Token metadata
        let token_metadata  = TokenMetadata {
            update_authority: Some(fee_payer.pubkey()).try_into().unwrap(),
            mint: mint.pubkey(),
            name: "USDT COin".to_string(),
            symbol : "USDT".to_string(),
            uri : "https://example.com/image.json".to_string(),
            additional_metadata: vec![("some".to_string(),"desc".to_string())]
        };

        // Calculate space for mint with metadata pointer and token metadata extensions
        let mint_space =
            ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::MetadataPointer]).unwrap();

        let metadata_len = token_metadata.tlv_size_of().unwrap();

        let mint_rent = client.get_minimum_balance_for_rent_exemption(mint_space + metadata_len).await.unwrap();

        // Instruction to create new account for mint (token22)
        let create_mint_account_instruction = create_account(
            &fee_payer.pubkey(),    // payer
            &mint.pubkey(),         // new account (mint)
            mint_rent,              // lamports
            mint_space as u64,      // space
            &TOKEN_2022_PROGRAM_ID, // program id
        );

        // Instruction to initialize metadata pointer (pointing to itself for self-managed metadata)
        let initialize_metadata_pointer_instruction = initialize_metadata_pointer(
            &TOKEN_2022_PROGRAM_ID,
            &mint.pubkey(),
            Some(fee_payer.pubkey()), // authority
            Some(mint.pubkey()),      // metadata address (pointing to self)
        ).unwrap();

        // Instruction to initialize mint account data
        let initialize_mint_instruction = initialize_mint(
            &TOKEN_2022_PROGRAM_ID,    // program id
            &mint.pubkey(),            // mint
            &fee_payer.pubkey(),       // mint authority
            Some(&fee_payer.pubkey()), // freeze authority
            9,                         // decimals
        ).unwrap();

        // Instruction to initialize token metadata
        let initialize_metadata_instruction = initialize_token_metadata(
            &TOKEN_2022_PROGRAM_ID,            // program id
            &mint.pubkey(),                    //metadata
            &fee_payer.pubkey(),               // update authority
            &mint.pubkey(),                    // mint
            &fee_payer.pubkey(),               // mint authority
            token_metadata.name.to_string(),   // name
            token_metadata.symbol.to_string(), // symbol
            token_metadata.uri.to_string(),    // uri
        );

        // Create update field instructions from token_metadata.additional_metadata
        // Additional metadata must be initialized separately using the update_field instruction
        // If the field already exists, it will be updated instead of creating a new field
        let update_field_instructions: Vec<_> = token_metadata
            .additional_metadata
            .iter()
            .map(|(key, value)| {
                update_field(
                    &TOKEN_2022_PROGRAM_ID,
                    &mint.pubkey(),
                    &fee_payer.pubkey(),
                    Field::Key(key.clone()),
                    value.clone(),
                )
            })
            .collect();

        // Construct transaction with all instructions
        let mut instructions = vec![
            create_mint_account_instruction,
            initialize_metadata_pointer_instruction,
            initialize_mint_instruction,
            initialize_metadata_instruction,
        ];
        instructions.extend(update_field_instructions);

        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&fee_payer.pubkey()),
            &[&fee_payer, &mint],
            latest_blockhash,
        );

        let transaction_signature = client.send_and_confirm_transaction(&transaction).await;

        assert!(transaction_signature.is_ok(), "{:?}", transaction_signature.err().unwrap());

        println!("Mint Address: {}", mint.pubkey());
        //println!("Transaction Signature: {}", transaction_signature);
        // Fetch mint account
        let mint_account = client.get_account(&mint.pubkey()).await.unwrap();
        // Deserialize the mint account with extensions
        let mint_state = StateWithExtensions::<Mint>::unpack(&mint_account.data).unwrap();
        // Get all extension types enabled on this mint
        let extension_types = mint_state.get_extension_types().unwrap();
        println!("\nExtensions enabled: {:?}", extension_types);
        // Deserialize the MetadataPointer extension data
        let metadata_pointer = mint_state.get_extension::<MetadataPointer>().unwrap();
        println!("\n{:#?}", metadata_pointer);
        // Deserialize the TokenMetadata extension data (variable-length)
        let token_metadata = mint_state.get_variable_len_extension::<TokenMetadata>().unwrap();
        println!("\n{:#?}", token_metadata);

        // Create associated token account for the payer (owner) to hold tokens

        let associated_token_address = get_associated_token_address_with_program_id(
            &fee_payer.pubkey(),
            &mint.pubkey(),
            &TOKEN_2022_PROGRAM_ID,
        );

        let create_associated_token_account_instructions = spl_associated_token_account::instruction::create_associated_token_account(
            &fee_payer.pubkey(),
            &fee_payer.pubkey(),
            &mint.pubkey(),
            &TOKEN_2022_PROGRAM_ID,
        );

        let create_associated_token_account_transaction = Transaction::new_signed_with_payer(
            &[create_associated_token_account_instructions],
            Some(&fee_payer.pubkey()),
            &[&fee_payer],
            latest_blockhash,
        );
        let create_associated_token_account_result = client.send_and_confirm_transaction(&create_associated_token_account_transaction).await;

        assert!(create_associated_token_account_result.is_ok(), "{:?}", create_associated_token_account_result.err().unwrap());

        // Mint tokens to the payer's token account
        let mint_instructions = spl_token_2022_interface::instruction::mint_to(
            &TOKEN_2022_PROGRAM_ID,
            &mint.pubkey(),
            &associated_token_address,
            &fee_payer.pubkey(),
            &[&fee_payer.pubkey()],
            1000000
        ).unwrap();
        let mint_tokens_tx = Transaction::new_signed_with_payer(
            &[mint_instructions],
            Some(&fee_payer.pubkey()),
            &[&fee_payer],
            latest_blockhash,
        );
        let mint_result = client.send_and_confirm_transaction(&mint_tokens_tx).await;

        assert!(mint_result.is_ok(), "{:?}", mint_result.err().unwrap());
    }

    async fn init_ramp(
        client: &RpcClient,
        fee_payer: &Keypair,
        ramp_program_id: &Keypair,
        ramp_data_account: &Keypair,
    ) {
        let latest_blockhash = client.get_latest_blockhash().await.unwrap();

        let init_instruction = Instruction::new_with_borsh(
            ramp_program_id.pubkey(),
             &processors::RampInstruction {
                 instruction: processors::Instruction::InitializeProgram(
                     InitializeProgramInstruction {
                        vault_address: fee_payer.pubkey(),
                        native_fee_percentage: 10,
                     }
                 ),
             },
             vec![
                 AccountMeta::new(ramp_data_account.pubkey(), true),
                 AccountMeta::new_readonly(fee_payer.pubkey(), true),
                 AccountMeta::new_readonly(system_program::id(), false),
             ],
         );

         let init_tx = Transaction::new_signed_with_payer(
            &[init_instruction],
            Some(&fee_payer.pubkey()),
            &[&ramp_data_account, &fee_payer],
            latest_blockhash,
        );
        let init_result = client.send_and_confirm_transaction(&init_tx).await;

        assert!(init_result.is_ok(), "{:?}", init_result.err().unwrap());
    }

     /**
      * Get the ramp state
      * 
      * @param banks_client - The banks client
      * @param ramp_pubkey - The ramp public key
      * @returns Result<RampState, Box<dyn std::error::Error>>
      */
     async fn get_ramp_state(
         client: &RpcClient,
         ramp_data_account: &Pubkey,
     ) -> Result<RampState, Box<dyn std::error::Error>> {
         let account = client.get_account(ramp_data_account).await.unwrap();    
         let mut data_slice: &[u8] = &account.data;
         let ramp_state = RampState::try_from_slice(&mut data_slice)?;
         Ok(ramp_state)
     }

    #[tokio::test]
    async fn test_add_assets() {
        let (
            client,
            fee_payer,
            ramp_program_id,
            mint,
            ramp_data_account
        ) = get_set_up().await;
        let fee_percentage = 100;
        let initial_amount = 100;
        
        // Get associated token addresses
        let owner_token_account = get_associated_token_address_with_program_id(
            &fee_payer.pubkey(),
            &mint.pubkey(),
            &spl_token_interface::ID
        );
        let ramp_token_account = get_associated_token_address_with_program_id(
            &ramp_data_account.pubkey(),
            &mint.pubkey(),
        &spl_token_interface::ID
    );

        let add_assets_instruction = Instruction::new_with_borsh(
            ramp_program_id.pubkey(),
            &processors::RampInstruction {
                instruction: processors::Instruction::AddAssets(
                    AddAssetsInstruction {
                        fee_percentage,
                        initial_amount,
                    }
                ),
            },
            vec![
                AccountMeta::new(ramp_data_account.pubkey(), false),
                AccountMeta::new(mint.pubkey(), false),
                AccountMeta::new(fee_payer.pubkey(), true), // Owner must sign
                AccountMeta::new_readonly(spl_token_interface::ID, false),
                AccountMeta::new_readonly(system_program::id(), false), // System program
                AccountMeta::new_readonly(spl_associated_token_account::id(), false), // Associated token program
                AccountMeta::new(owner_token_account, false), // Owner's token account
                AccountMeta::new(ramp_token_account, false), // Ramp's token account
            ],
        );
        
        let add_assets_tx = Transaction::new_signed_with_payer(
            &[add_assets_instruction],
            Some(&fee_payer.pubkey()),
            &[&fee_payer],
            client.get_latest_blockhash().await.unwrap(),
        );
        let add_assets_result = client.send_and_confirm_transaction(&add_assets_tx).await;
        assert!(add_assets_result.is_ok(), "{:?}", add_assets_result.err().unwrap());

        // Verify the state
        let ramp_state = get_ramp_state(&client, &ramp_data_account.pubkey()).await
            .expect("Failed to get ramp state");
        
        let assets = ramp_state.get_assets();
        assert!(assets.contains(&mint.pubkey()), "Should contain asset: {:?}", assets);
        
        // Check asset info
        let asset_info = ramp_state.get_asset_info_ref(&mint.pubkey()).expect("Asset info should exist");
        assert_eq!(asset_info.asset_fee_percentage, 100, "Asset fee should be 100");
        
        println!("Add assets test passed!");
    }
}




//     /**
//      * Tests
//      */



//     ///**
//     // * Test program initialization
//     // */
//     //#[tokio::test]
//     //async fn test_program_initialization() {
//     //    let (mut ctx, program_id, _asset_mint, ramp_keypair) = setup_program().await;
//     //    //let owner = Keypair::new();
//     //    let vault_address = Pubkey::new_unique();
// //
//     //    println!("Testing program initialization...");
// //
//     //    // Test successful initialization
//     //    let result = initialize_ramp_program(
//     //        &mut ctx.banks_client,
//     //        &ctx.payer,
//     //        ctx.last_blockhash,
//     //        program_id,
//     //        &ramp_keypair,
//     //        vault_address,
//     //    ).await;
// //
//     //    println!("Program initialized successfully!");
//     //    assert!(result.is_ok(), "{:?}", result.err().unwrap());
// //
//     //    // Verify the state
//     //    let ramp_state = get_ramp_state(&mut ctx.banks_client, ramp_keypair.pubkey()).await
//     //        .expect("Failed to get ramp state");
// //
//     //    assert_eq!(ramp_state.owner, ctx.payer.pubkey(), "RampState owner should match payer");
//     //    assert_eq!(ramp_state.vault_address, vault_address, "Vault address should match");
//     //    assert!(!ramp_state.is_active, "RampState should not be active initially");
//     //}
// //
// //
//     ///**
//     // * Test add assets
//     // */
//     //#[tokio::test]
//     //async fn test_add_assets() {
//     //    let (mut ctx, program_id, asset_mint, ramp_keypair) = setup_program().await;
//     //    let vault_address = Pubkey::new_unique();
// //
//     //    // Initialize the program first
//     //    initialize_ramp_program(
//     //        &mut ctx.banks_client,
//     //        &ctx.payer,
//     //        ctx.last_blockhash,
//     //        program_id,
//     //        &ramp_keypair,
//     //        vault_address,
//     //    ).await.expect("Initialization should succeed");
// //
//     //    println!("Testing add assets...");
//     //    
//     //    let fee_percentage = 100;
//     //    let initial_amount = 100;
//     //    
//     //    // Get associated token addresses
//     //    let owner_token_account = get_associated_token_address(&ctx.payer.pubkey(), &asset_mint.pubkey());
//     //    let ramp_token_account = get_associated_token_address(&ramp_keypair.pubkey(), &asset_mint.pubkey());
// //
//     //    let add_assets_instruction = Instruction::new_with_borsh(
//     //        program_id,
//     //        &processors::RampInstruction {
//     //            instruction: processors::Instruction::AddAssets(
//     //                AddAssetsInstruction {
//     //                    fee_percentage,
//     //                    initial_amount,
//     //                }
//     //            ),
//     //        },
//     //        vec![
//     //            AccountMeta::new(ramp_keypair.pubkey(), false),
//     //            AccountMeta::new(asset_mint.pubkey(), false),
//     //            AccountMeta::new(ctx.payer.pubkey(), true), // Owner must sign
//     //            AccountMeta::new_readonly(spl_token::id(), false),
//     //            AccountMeta::new_readonly(system_program::id(), false), // System program
//     //            AccountMeta::new_readonly(spl_associated_token_account::id(), false), // Associated token program
//     //            AccountMeta::new(owner_token_account, false), // Owner's token account
//     //            AccountMeta::new(ramp_token_account, false), // Ramp's token account
//     //        ],
//     //    );
// //
//     //    let mut transaction = Transaction::new_with_payer(&[add_assets_instruction], Some(&ctx.payer.pubkey()));
//     //    transaction.sign(&[&ctx.payer], ctx.last_blockhash);
//     //    
//     //    let result = ctx.banks_client.process_transaction(transaction).await;
//     //    assert!(result.is_ok(), "Add assets should succeed: {:?}", result.err());
// //
//     //    // Verify the state
//     //    let ramp_state = get_ramp_state(&mut ctx.banks_client, ramp_keypair.pubkey()).await
//     //        .expect("Failed to get ramp state");
// //
//     //    let assets = ramp_state.get_assets();
//     //    assert!(assets.contains(&asset_mint.pubkey()), "Should contain asset");
//     //    
//     //    // Check asset info
//     //    let asset_info = ramp_state.get_asset_info_ref(&asset_mint.pubkey()).expect("Asset info should exist");
//     //    assert_eq!(asset_info.asset_fee_percentage, 100, "Asset fee should be 100");
//     //    
//     //    println!("Add assets test passed!");
//     //}
// //
//     ///**
//     // * Test add assets unauthorized
//     // */
//     //#[tokio::test]
//     //async fn test_add_assets_unauthorized() {
//     //    let (mut ctx, program_id, asset_mint, ramp_keypair) = setup_program().await;
//     //    let vault_address = Pubkey::new_unique();
//     //    let unauthorized_user = Keypair::new();
// //
//     //    // Initialize the program first
//     //    initialize_ramp_program(
//     //        &mut ctx.banks_client,
//     //        &ctx.payer,
//     //        ctx.last_blockhash,
//     //        program_id,
//     //        &ramp_keypair,
//     //        vault_address,
//     //    ).await.expect("Initialization should succeed");
// //
//     //    println!("Testing unauthorized add assets...");
// //
//     //    let fee_percentage = 100;
//     //    let initial_amount = 100;
//     //    
//     //    // Get associated token addresses
//     //    let unauthorized_token_account = get_associated_token_address(&unauthorized_user.pubkey(), &asset_mint.pubkey());
//     //    let ramp_token_account = get_associated_token_address(&ramp_keypair.pubkey(), &asset_mint.pubkey());
// //
//     //    let add_assets_instruction = Instruction::new_with_borsh(
//     //        program_id,
//     //        &processors::RampInstruction {
//     //            instruction: processors::Instruction::AddAssets(
//     //                AddAssetsInstruction {
//     //                    fee_percentage,
//     //                    initial_amount,
//     //                }
//     //            ),
//     //        },
//     //        vec![
//     //            AccountMeta::new(ramp_keypair.pubkey(), false),
//     //            AccountMeta::new(asset_mint.pubkey(), false),
//     //            AccountMeta::new(unauthorized_user.pubkey(), true), // Wrong owner
//     //            AccountMeta::new_readonly(spl_token::id(), false),
//     //            AccountMeta::new_readonly(system_program::id(), false), // System program
//     //            AccountMeta::new_readonly(spl_associated_token_account::id(), false), // Associated token program
//     //            AccountMeta::new(unauthorized_token_account, false), // Unauthorized user's token account
//     //            AccountMeta::new(ramp_token_account, false), // Ramp's token account
//     //        ],
//     //    );
// //
//     //    let mut transaction = Transaction::new_with_payer(&[add_assets_instruction], Some(&unauthorized_user.pubkey()));
//     //    transaction.sign(&[&unauthorized_user], ctx.last_blockhash);
//     //    
//     //    let result = ctx.banks_client.process_transaction(transaction).await;
//     //    assert!(result.is_err(), "Unauthorized add assets should fail");
//     //    println!("Correctly rejected unauthorized add assets");
//     //}
// //
//     ///**
//     // * Test remove assets
//     // */
//     //#[tokio::test]
//     //async fn test_remove_assets() {
//     //    let (mut ctx, program_id, asset_mint, ramp_keypair) = setup_program().await;
//     //    let vault_address = Pubkey::new_unique();
// //
//     //    // Initialize and add assets first
//     //    initialize_ramp_program(
//     //        &mut ctx.banks_client,
//     //        &ctx.payer,
//     //        ctx.last_blockhash,
//     //        program_id,
//     //        &ramp_keypair,
//     //        vault_address,
//     //    ).await.expect("Initialization should succeed");
// //
//     //    println!("Testing add assets...");
//     //    
//     //    let fee_percentage = 100;
//     //    let initial_amount = 100;
//     //    
//     //    // Get associated token addresses
//     //    let owner_token_account = get_associated_token_address(&ctx.payer.pubkey(), &asset_mint.pubkey());
//     //    let ramp_token_account = get_associated_token_address(&ramp_keypair.pubkey(), &asset_mint.pubkey());
// //
//     //    let add_assets_instruction = Instruction::new_with_borsh(
//     //        program_id,
//     //        &processors::RampInstruction {
//     //            instruction: processors::Instruction::AddAssets(
//     //                AddAssetsInstruction {
//     //                    fee_percentage,
//     //                    initial_amount,
//     //                }
//     //            ),
//     //        },
//     //        vec![
//     //            AccountMeta::new(ramp_keypair.pubkey(), false),
//     //            AccountMeta::new(asset_mint.pubkey(), false),
//     //            AccountMeta::new(ctx.payer.pubkey(), true), // Owner must sign
//     //            AccountMeta::new_readonly(spl_token::id(), false),
//     //            AccountMeta::new_readonly(system_program::id(), false), // System program
//     //            AccountMeta::new_readonly(spl_associated_token_account::id(), false), // Associated token program
//     //            AccountMeta::new(owner_token_account, false), // Owner's token account
//     //            AccountMeta::new(ramp_token_account, false), // Ramp's token account
//     //        ],
//     //    );
// //
//     //    let mut transaction = Transaction::new_with_payer(&[add_assets_instruction], Some(&ctx.payer.pubkey()));
//     //    transaction.sign(&[&ctx.payer], ctx.last_blockhash);
//     //    
//     //    let result = ctx.banks_client.process_transaction(transaction).await;
//     //    assert!(result.is_ok(), "Add assets should succeed: {:?}", result.err());
// //
//     //    println!("Testing remove assets...");
// //
//     //    // remove one asset
//     //    let remove_instruction = Instruction::new_with_borsh(
//     //        program_id,
//     //        &processors::RampInstruction {
//     //            instruction: processors::Instruction::RemoveAssets(
//     //                RemoveAssetsInstruction {
//     //                    //asset: asset_mint.pubkey(),
//     //                }
//     //            ),
//     //        },
//     //        vec![
//     //            AccountMeta::new(ramp_keypair.pubkey(), true), // Ramp account needs to sign for token transfer
//     //            AccountMeta::new(
//     //                get_associated_token_address(&ramp_keypair.pubkey(), &asset_mint.pubkey()),
//     //                false,
//     //            ),
//     //            AccountMeta::new_readonly(asset_mint.pubkey(), false),
//     //            AccountMeta::new(ctx.payer.pubkey(), true), // Owner also signs
//     //            AccountMeta::new(
//     //                get_associated_token_address(&ctx.payer.pubkey(), &asset_mint.pubkey()),
//     //                false,
//     //            ),
//     //            AccountMeta::new_readonly(spl_token::id(), false),
//     //        ],
//     //    );
//     //    let mut transaction = Transaction::new_with_payer(
//     //        &[remove_instruction],
//     //        Some(&ctx.payer.pubkey())
//     //    );
//     //    transaction.sign(&[&ctx.payer, &ramp_keypair], ctx.last_blockhash);
//     //    
//     //    let result = ctx.banks_client.process_transaction(transaction).await;
//     //    assert!(result.is_ok(), "Remove assets should succeed: {:?}", result.err());
// //
//     //    // Verify the state
//     //    let ramp_state = get_ramp_state(&mut ctx.banks_client, ramp_keypair.pubkey()).await
//     //        .expect("Failed to get ramp state");
// //
//     //    let assets = ramp_state.get_assets();
//     //    assert!(!assets.contains(&asset_mint.pubkey()), "Should not contain removed asset1");
//     //    assert!(ramp_state.get_asset_info_ref(&asset_mint.pubkey()).is_none(), "Asset1 info should be removed");
//     //    println!("Remove assets test passed!");
//     //}
// //
//     ///**
//     // * Test set active
//     // */
//     //#[tokio::test]
//     //async fn test_set_active() {
//     //    let (mut ctx, program_id, _asset_mint, ramp_keypair) = setup_program().await;
//     //    let vault_address = Pubkey::new_unique();
// //
//     //    // Initialize the program first
//     //    initialize_ramp_program(
//     //        &mut ctx.banks_client,
//     //        &ctx.payer,
//     //        ctx.last_blockhash,
//     //        program_id,
//     //        &ramp_keypair,
//     //        vault_address,
//     //    ).await.expect("Initialization should succeed");
// //
//     //    println!("Testing set active...");
//     //    // Set active to true
//     //    let set_active_instruction = Instruction::new_with_borsh(
//     //        program_id,
//     //        &processors::RampInstruction {
//     //            instruction: processors::Instruction::SetActive(
//     //                SetActiveInstruction {
//     //                    is_active: true,
//     //                }
//     //            ),
//     //        },
//     //        vec![
//     //            AccountMeta::new(ramp_keypair.pubkey(), true),
//     //            AccountMeta::new(ctx.payer.pubkey(), true),
//     //        ],
//     //    );
// //
//     //    let mut transaction = Transaction::new_with_payer(&[set_active_instruction], Some(&ctx.payer.pubkey()));
//     //    transaction.sign(&[&ctx.payer, &ramp_keypair], ctx.last_blockhash);
//     //    
//     //    let result = ctx.banks_client.process_transaction(transaction).await;
//     //    assert!(result.is_ok(), "Set active should succeed: {:?}", result.err());
// //
//     //    // Verify the state
//     //    let ramp_state = get_ramp_state(&mut ctx.banks_client, ramp_keypair.pubkey()).await
//     //        .expect("Failed to get ramp state");
// //
//     //    assert!(ramp_state.is_active, "RampState should be active");
//     //    println!("Set active test passed!");
//     //}
// //
//     ///**
//     // * Test Set native fee
//     // */
// //
//     //#[tokio::test]
//     //async fn test_set_native_fee_percentage() {
//     //    let (
//     //        mut ctx,
//     //        program_id,
//     //        _asset_mint,
//     //        ramp_keypair,
//     //    ) = setup_program().await;
// //
//     //    let vault_address = Pubkey::new_unique();
// //
//     //    // Initialize the program first
//     //    initialize_ramp_program(
//     //        &mut ctx.banks_client,
//     //        &ctx.payer,
//     //        ctx.last_blockhash,
//     //        program_id,
//     //        &ramp_keypair,
//     //        vault_address,
//     //    ).await.expect("Initialization should succeed");
// //
//     //    println!("Testing set native fee percentage...");
// //
//     //    let set_native_fee_percentage_instruction = Instruction::new_with_borsh(
//     //        program_id,
//     //        &processors::RampInstruction {
//     //            instruction: processors::Instruction::SetNativeFeePercentage(
//     //                SetNativeFeePercentageInstruction {
//     //                    fee_percentage: 100,
//     //                }
//     //            ),
//     //        },
//     //        vec![
//     //            AccountMeta::new(ramp_keypair.pubkey(), true),
//     //            AccountMeta::new(ctx.payer.pubkey(), true),
//     //        ],
//     //    );
// //
//     //    let mut transaction = Transaction::new_with_payer(&[set_native_fee_percentage_instruction], Some(&ctx.payer.pubkey()));
//     //    transaction.sign(&[&ctx.payer, &ramp_keypair], ctx.last_blockhash);
//     //    
//     //    let result = ctx.banks_client.process_transaction(transaction).await;
//     //    assert!(result.is_ok(), "Set native fee percentage should succeed: {:?}", result.err());
// //
//     //    // Verify the state
//     //    let ramp_state = get_ramp_state(&mut ctx.banks_client, ramp_keypair.pubkey()).await
//     //        .expect("Failed to get ramp state");
// //
//     //    assert_eq!(ramp_state.native_fee_percentage, 100, "Native fee percentage should be updated");
//     //    println!("Set native fee percentage test passed!");
//     //}
// //
//     ///**
//     // * Set Asset Fee
//     // */
// //
//     //#[tokio::test]
//     //async fn test_set_asset_fee() {
//     //    let (
//     //        mut ctx,
//     //        program_id,
//     //        asset_mint,
//     //        ramp_keypair,
//     //    ) = setup_program().await;
// //
//     //    let vault_address = Pubkey::new_unique();
// //
//     //    // Initialize the program first
//     //    initialize_ramp_program(
//     //        &mut ctx.banks_client,
//     //        &ctx.payer,
//     //        ctx.last_blockhash,
//     //        program_id,
//     //        &ramp_keypair,
//     //        vault_address,
//     //    ).await.expect("Initialization should succeed");
// //
//     //    // Set active first
//     //    ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
//     //    let set_active_instruction = Instruction::new_with_borsh(
//     //        program_id,
//     //        &processors::RampInstruction {
//     //            instruction: processors::Instruction::SetActive(
//     //                SetActiveInstruction {
//     //                    is_active: true,
//     //                }
//     //            ),
//     //        },
//     //        vec![
//     //            AccountMeta::new(ramp_keypair.pubkey(), true),
//     //            AccountMeta::new(ctx.payer.pubkey(), true),
//     //        ],
//     //    );
// //
//     //    let mut transaction = Transaction::new_with_payer(&[set_active_instruction], Some(&ctx.payer.pubkey()));
//     //    transaction.sign(&[&ctx.payer, &ramp_keypair], ctx.last_blockhash);
//     //    
//     //    let result = ctx.banks_client.process_transaction(transaction).await;
//     //    assert!(result.is_ok(), "Set active should succeed");
// //
//     //    // Add the asset first
//     //    ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
//     //    let owner_token_account = get_associated_token_address(&ctx.payer.pubkey(), &asset_mint.pubkey());
//     //    let ramp_token_account = get_associated_token_address(&ramp_keypair.pubkey(), &asset_mint.pubkey());
// //
//     //    let add_assets_instruction = Instruction::new_with_borsh(
//     //        program_id,
//     //        &processors::RampInstruction {
//     //            instruction: processors::Instruction::AddAssets(
//     //                AddAssetsInstruction {
//     //                    initial_amount: 1000,
//     //                    fee_percentage: 50, // Initial fee
//     //                }
//     //            ),
//     //        },
//     //        vec![
//     //            AccountMeta::new(ramp_keypair.pubkey(), false),
//     //            AccountMeta::new(asset_mint.pubkey(), false),
//     //            AccountMeta::new(ctx.payer.pubkey(), true),
//     //            AccountMeta::new_readonly(spl_token::id(), false),
//     //            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
//     //            AccountMeta::new_readonly(system_program::id(), false),
//     //            AccountMeta::new(owner_token_account, false),
//     //            AccountMeta::new(ramp_token_account, false),
//     //        ],
//     //    );
// //
//     //    let mut transaction = Transaction::new_with_payer(&[add_assets_instruction], Some(&ctx.payer.pubkey()));
//     //    transaction.sign(&[&ctx.payer], ctx.last_blockhash);
//     //    
//     //    let result = ctx.banks_client.process_transaction(transaction).await;
//     //    assert!(result.is_ok(), "Add assets should succeed");
// //
//     //    println!("Testing set asset fee...");
// //
//     //    // Now set the asset fee
//     //    ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
//     //    let set_asset_fee_instruction = Instruction::new_with_borsh(
//     //        program_id,
//     //        &processors::RampInstruction {
//     //            instruction: processors::Instruction::SetAssetFee(
//     //                SetAssetFeeInstruction {
//     //                    asset_mint: asset_mint.pubkey(),
//     //                    fee_percentage: 100,
//     //                }
//     //            ),
//     //        },
//     //        vec![
//     //            AccountMeta::new(ramp_keypair.pubkey(), true),
//     //            AccountMeta::new(ctx.payer.pubkey(), true),
//     //        ],
//     //    );
// //
//     //    let mut transaction = Transaction::new_with_payer(&[set_asset_fee_instruction], Some(&ctx.payer.pubkey()));
//     //    transaction.sign(&[&ctx.payer, &ramp_keypair], ctx.last_blockhash);
//     //    
//     //    let result = ctx.banks_client.process_transaction(transaction).await;
//     //    assert!(result.is_ok(), "Set asset fee should succeed: {:?}", result.err());
// //
//     //    let ramp_state = get_ramp_state(&mut ctx.banks_client, ramp_keypair.pubkey()).await
//     //        .expect("Failed to get ramp state");
// //
//     //    let asset_info = ramp_state.get_asset_info_ref(&asset_mint.pubkey())
//     //        .expect("Asset should exist");
//     //    assert_eq!(asset_info.get_fee_percentage(), 100, "Asset fee percentage should be updated to 100");
//     //    println!("Set asset fee test passed!");
//     //}
// //
//     ///**
//     // * Test set owner
//     // */
//     //#[tokio::test]
//     //async fn test_set_owner() {
//     //    let (
//     //        mut ctx,
//     //        program_id,
//     //        _asset_mint,
//     //        ramp_keypair,
//     //    ) = setup_program().await;
// //
//     //    let vault_address = Pubkey::new_unique();
//     //    let new_owner = Keypair::new();
// //
//     //    // Initialize the program first
//     //    initialize_ramp_program(
//     //        &mut ctx.banks_client,
//     //        &ctx.payer,
//     //        ctx.last_blockhash,
//     //        program_id,
//     //        &ramp_keypair,
//     //        vault_address,
//     //    ).await.expect("Initialization should succeed");
// //
//     //    println!("Testing set owner...");
// //
//     //    let set_owner_instruction = Instruction::new_with_borsh(
//     //        program_id,
//     //        &processors::RampInstruction {
//     //            instruction: processors::Instruction::SetOwner(
//     //                SetOwnerInstruction {
//     //                    new_owner: new_owner.pubkey(),
//     //                }
//     //            ),
//     //        },
//     //        vec![
//     //            AccountMeta::new(ramp_keypair.pubkey(), false),
//     //            AccountMeta::new(ctx.payer.pubkey(), true),
//     //        ],
//     //    );
// //
//     //    let mut transaction = Transaction::new_with_payer(&[set_owner_instruction], Some(&ctx.payer.pubkey()));
//     //    transaction.sign(&[&ctx.payer], ctx.last_blockhash);
//     //    
//     //    let result = ctx.banks_client.process_transaction(transaction).await;
// //
//     //    assert!(result.is_ok(), "Set owner should succeed: {:?}", result.err());
// //
//     //    let ramp_state = get_ramp_state(&mut ctx.banks_client, ramp_keypair.pubkey()).await
//     //        .expect("Failed to get ramp state");
// //
//     //    assert_eq!(ramp_state.owner, new_owner.pubkey(), "Owner should be updated");
//     //    println!("Set owner test passed!");
//     //}
// //
//     ///**
//     // * Test add assets with invalid fee
//     // */
//     //#[tokio::test]
//     //async fn test_add_assets_invalid_fee() {
//     //    let (mut ctx, program_id, asset_mint, ramp_keypair) = setup_program().await;
//     //    let vault_address = Pubkey::new_unique();
// //
//     //    // Initialize the program first
//     //    initialize_ramp_program(
//     //        &mut ctx.banks_client,
//     //        &ctx.payer,
//     //        ctx.last_blockhash,
//     //        program_id,
//     //        &ramp_keypair,
//     //        vault_address,
//     //    ).await.expect("Initialization should succeed");
// //
//     //    println!("Testing add assets with invalid fee...");
//     //    let fee_percentage = 15000; // > 10000 basis points (100%)
//     //    let initial_amount = 100;
// //
//     //    // Get associated token addresses
//     //    let owner_token_account = get_associated_token_address(&ctx.payer.pubkey(), &asset_mint.pubkey());
//     //    let ramp_token_account = get_associated_token_address(&ramp_keypair.pubkey(), &asset_mint.pubkey());
// //
//     //    let add_assets_instruction = Instruction::new_with_borsh(
//     //        program_id,
//     //        &processors::RampInstruction {
//     //            instruction: processors::Instruction::AddAssets(
//     //                AddAssetsInstruction {
//     //                    fee_percentage,
//     //                    initial_amount,
//     //                }
//     //            ),
//     //        },
//     //        vec![
//     //            AccountMeta::new(ramp_keypair.pubkey(), false),
//     //            AccountMeta::new(asset_mint.pubkey(), false),
//     //            AccountMeta::new(ctx.payer.pubkey(), true),
//     //            AccountMeta::new_readonly(spl_token::id(), false),
//     //            AccountMeta::new_readonly(system_program::id(), false),
//     //            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
//     //            AccountMeta::new(owner_token_account, false),
//     //            AccountMeta::new(ramp_token_account, false),
//     //        ],
//     //    );
// //
//     //    let mut transaction = Transaction::new_with_payer(&[add_assets_instruction], Some(&ctx.payer.pubkey()));
//     //    transaction.sign(&[&ctx.payer], ctx.last_blockhash);
//     //    
//     //    let result = ctx.banks_client.process_transaction(transaction).await;
//     //    assert!(result.is_err(), "Add assets with invalid fee should fail");
//     //    println!("Correctly rejected invalid fee percentage");
//     //}

//     /**
//      * Test off_ramp_deposit
//      * 
//      * 
//      */

//     #[tokio::test]
//     async fn test_off_ramp_deposit() {
//         // initialize program
//         let (mut ctx, program_id, asset_mint, ramp_keypair) = setup_program().await;
//         let vault_address = Pubkey::new_unique();

//         // Initialize the program first
//         initialize_ramp_program(
//             &mut ctx.banks_client,
//             &ctx.payer,
//             ctx.last_blockhash,
//             program_id,
//             &ramp_keypair,
//             vault_address,
//         ).await.expect("Initialization should succeed");

//         // set active
//         let set_active_instruction = Instruction::new_with_borsh(
//             program_id,
//             &processors::RampInstruction {
//                 instruction: processors::Instruction::SetActive(
//                     SetActiveInstruction {
//                         is_active: true,
//                     }
//                 ),
//             },
//             vec![
//                 AccountMeta::new(ramp_keypair.pubkey(), true),
//                 AccountMeta::new(ctx.payer.pubkey(), true),
//             ],
//         );

//         let mut transaction = Transaction::new_with_payer(&[set_active_instruction], Some(&ctx.payer.pubkey()));
//         transaction.sign(&[&ctx.payer, &ramp_keypair], ctx.last_blockhash);
        
//         let result = ctx.banks_client.process_transaction(transaction).await;
//         assert!(result.is_ok(), "Set active should succeed");
//         println!("Set active test passed!");

//         let fee_percentage = 100;
//         let initial_amount = 100;
        
//         // Get associated token addresses
//         let owner_token_account = get_associated_token_address(&ctx.payer.pubkey(), &asset_mint.pubkey());
//         let ramp_token_account = get_associated_token_address(&ramp_keypair.pubkey(), &asset_mint.pubkey());
//         // add assets
//         let add_assets_instruction = Instruction::new_with_borsh(
//             program_id,
//             &processors::RampInstruction {
//                 instruction: processors::Instruction::AddAssets(
//                     AddAssetsInstruction {
//                         fee_percentage,
//                         initial_amount,
//                     }
//                 ),
//             },
//             vec![
//                 AccountMeta::new(ramp_keypair.pubkey(), false),
//                 AccountMeta::new(asset_mint.pubkey(), false),
//                 AccountMeta::new(ctx.payer.pubkey(), true),
//                 AccountMeta::new_readonly(spl_token::id(), false),
//                 AccountMeta::new_readonly(system_program::id(), false),
//                 AccountMeta::new_readonly(spl_associated_token_account::id(), false),
//                 AccountMeta::new(owner_token_account, false),
//                 AccountMeta::new(ramp_token_account, false),
//             ],
//         );

//         let mut transaction = Transaction::new_with_payer(
//             &[add_assets_instruction],
//             Some(&ctx.payer.pubkey())
//         );
//         transaction.sign(&[&ctx.payer], ctx.last_blockhash);
        
//         let result = ctx.banks_client.process_transaction(transaction).await;
//         assert!(result.is_ok(), "Add assets should succeed");
//         println!("Add assets test passed!");

//         // off ramp deposit (ramp ATA is created by add_assets)
//         ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();

//         let off_ramp_withdraw_instruction = Instruction::new_with_borsh(
//             program_id,
//             &processors::RampInstruction {
//                 instruction: processors::Instruction::OffRampDeposit(
//                     OffRampDepositInstruction {
//                         amount: 100,
//                         region: models::Region::KEN,
//                         medium: models::Medium::Primary,
//                         data: vec![],
//                     }
//                 ),
//             },
//             vec![
//                 AccountMeta::new(ramp_keypair.pubkey(), false),
//                 AccountMeta::new(asset_mint.pubkey(), false),
//                 AccountMeta::new(ctx.payer.pubkey(), true),
//                 AccountMeta::new(owner_token_account, false),
//                 AccountMeta::new(ramp_token_account, false),
//                 AccountMeta::new_readonly(spl_token::id(), false),
//             ],
//         );

//         let mut transaction = Transaction::new_with_payer(
//             &[off_ramp_withdraw_instruction],
//             Some(&ctx.payer.pubkey()),
//         );
//         transaction.sign(&[&ctx.payer], ctx.last_blockhash);
        
//         let result = ctx.banks_client.process_transaction(transaction).await;
//         assert!(result.is_ok(), "Off ramp withdraw should succeed: {:?}", result.err());
//         println!("Off ramp withdraw test passed!");
        
//     }

//     //#[tokio::test]
//     //async fn test_on_ramp_withdraw() {
//     //    // initialize program
//     //    let (mut ctx, program_id, asset_mint, ramp_keypair) = setup_program().await;
//     //    let vault_address = Pubkey::new_unique();
// //
//     //    // Initialize the program first
//     //    initialize_ramp_program(
//     //        &mut ctx.banks_client,
//     //        &ctx.payer,
//     //        ctx.last_blockhash,
//     //        program_id,
//     //        &ramp_keypair,
//     //        vault_address,
//     //    ).await.expect("Initialization should succeed");
// //
//     //    // set active
//     //    let set_active_instruction = Instruction::new_with_borsh(
//     //        program_id,
//     //        &processors::RampInstruction {
//     //            instruction: processors::Instruction::SetActive(
//     //                SetActiveInstruction {
//     //                    is_active: true,
//     //                }
//     //            ),
//     //        },
//     //        vec![
//     //            AccountMeta::new(ramp_keypair.pubkey(), true),
//     //            AccountMeta::new(ctx.payer.pubkey(), true),
//     //        ],
//     //    );
// //
//     //    let mut transaction = Transaction::new_with_payer(&[set_active_instruction], Some(&ctx.payer.pubkey()));
//     //    transaction.sign(&[&ctx.payer, &ramp_keypair], ctx.last_blockhash);
//     //    
//     //    let result = ctx.banks_client.process_transaction(transaction).await;
//     //    assert!(result.is_ok(), "Set active should succeed");
//     //    println!("Set active test passed!");
// //
//     //    // add assets
//     //    let owner_token_account = get_associated_token_address(&ctx.payer.pubkey(), &asset_mint.pubkey());
//     //    let ramp_token_account = get_associated_token_address(&ramp_keypair.pubkey(), &asset_mint.pubkey());
// //
//     //    let add_assets_instruction = Instruction::new_with_borsh(
//     //        program_id,
//     //        &processors::RampInstruction {
//     //            instruction: processors::Instruction::AddAssets(
//     //                AddAssetsInstruction {
//     //                    fee_percentage: 100,
//     //                    initial_amount: 100,
//     //                }
//     //            ),
//     //        },
//     //        vec![
//     //            AccountMeta::new(ramp_keypair.pubkey(), false),
//     //            AccountMeta::new(asset_mint.pubkey(), false),
//     //            AccountMeta::new(ctx.payer.pubkey(), true),
//     //            AccountMeta::new_readonly(spl_token::id(), false),
//     //            AccountMeta::new_readonly(system_program::id(), false),
//     //            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
//     //            AccountMeta::new(owner_token_account, false),
//     //            AccountMeta::new(ramp_token_account, false),
//     //        ],
//     //    );
// //
//     //    let mut transaction = Transaction::new_with_payer(&[add_assets_instruction], Some(&ctx.payer.pubkey()));
//     //    transaction.sign(&[&ctx.payer], ctx.last_blockhash);
//     //    
//     //    let result = ctx.banks_client.process_transaction(transaction).await;
//     //    assert!(result.is_ok(), "Add assets should succeed");
//     //    println!("Add assets test passed!");
// //
//     //    // on ramp withdraw
//     //    let on_ramp_withdraw_instruction = Instruction::new_with_borsh(
//     //        program_id,
//     //        &processors::RampInstruction {
//     //            instruction: processors::Instruction::OnRampWithdraw(
//     //                OnRampWithdrawInstruction {
//     //                    amount: 100,
//     //                }
//     //            ),
//     //        },
//     //        vec![
//     //            AccountMeta::new(ramp_keypair.pubkey(), true),
//     //            AccountMeta::new(asset_mint.pubkey(), false),
//     //            AccountMeta::new(ctx.payer.pubkey(), true),
//     //            AccountMeta::new(owner_token_account, false),
//     //            AccountMeta::new(ramp_token_account, false),
//     //            AccountMeta::new_readonly(spl_token::id(), false),
//     //        ],
//     //    );
// //
//     //    let mut transaction = Transaction::new_with_payer(&[on_ramp_withdraw_instruction], Some(&ctx.payer.pubkey()));
//     //    transaction.sign(&[&ctx.payer, &ramp_keypair], ctx.last_blockhash);
//     //    
//     //    let result = ctx.banks_client.process_transaction(transaction).await;
//     //    assert!(result.is_ok(), "{:?}", result.err());
//     //    println!("On ramp withdraw test passed!");
//     //    
//     //}
// //
//     #[tokio::test]
//     async fn test_off_ramp_deposit_native() {
//         //initialize program
//         let (mut ctx, program_id, _asset_mint, ramp_keypair) = setup_program().await;

//         initialize_ramp_program(
//             &mut ctx.banks_client,
//             &ctx.payer,
//             ctx.last_blockhash,
//             program_id,
//             &ramp_keypair,
//             ctx.payer.pubkey()
//         ).await.expect("Initialization should succeed");

//         // set active
//         let set_active_instruction = Instruction::new_with_borsh(
//             program_id,
//             &processors::RampInstruction {
//                 instruction: processors::Instruction::SetActive(
//                     SetActiveInstruction {
//                         is_active: true,
//                     }
//                 ),
//             },
//             vec![
//                 AccountMeta::new(ramp_keypair.pubkey(), true),
//                 AccountMeta::new(ctx.payer.pubkey(), true),
//             ],
//         );

//         let mut transaction = Transaction::new_with_payer(&[set_active_instruction], Some(&ctx.payer.pubkey()));
//         transaction.sign(&[&ctx.payer, &ramp_keypair], ctx.last_blockhash);
        
//         let result = ctx.banks_client.process_transaction(transaction).await;
//         assert!(result.is_ok(), "Set active should succeed");
//         println!("Set active test passed!");

//         // off ramp deposit native

//         let off_ramp_deposit_native_instruction = Instruction::new_with_borsh(
//             program_id,
//             &processors::RampInstruction {
//                 instruction: processors::Instruction::OffRampDepositNative(
//                     OffRampDepositNativeInstruction {
//                         amount: 100,
//                         region: models::Region::KEN,
//                         medium: models::Medium::Primary,
//                         data: vec![],
//                     }
//                 ),
//             },
//             vec![
//                 AccountMeta::new(ramp_keypair.pubkey(), false),
//                 AccountMeta::new(ctx.payer.pubkey(), true),
//                 AccountMeta::new_readonly(system_program::id(), false),
//             ],
//         );

//         let mut transaction = Transaction::new_with_payer(&[off_ramp_deposit_native_instruction], Some(&ctx.payer.pubkey()));
//         transaction.sign(&[&ctx.payer], ctx.last_blockhash);
        
//         let result = ctx.banks_client.process_transaction(transaction).await;
//         assert!(result.is_ok(), "Off ramp deposit native should succeed {:?}", result.err());
//         println!("Off ramp deposit native test passed!");
        
//     }

//     #[tokio::test]
//     async fn test_on_ramp_withdraw_native() {
//         //initialize program
//         let (mut ctx, program_id, _asset_mint, ramp_keypair) = setup_program().await;
        
//         initialize_ramp_program(
//             &mut ctx.banks_client,
//             &ctx.payer,
//             ctx.last_blockhash,
//             program_id,
//             &ramp_keypair,
//             ctx.payer.pubkey()
//         ).await.expect("Initialization should succeed");

//         // set active
//         let set_active_instruction = Instruction::new_with_borsh(
//             program_id,
//             &processors::RampInstruction {
//                 instruction: processors::Instruction::SetActive(
//                     SetActiveInstruction {
//                         is_active: true,
//                     }
//                 ),
//             },
//             vec![
//                 AccountMeta::new(ramp_keypair.pubkey(), true),
//                 AccountMeta::new(ctx.payer.pubkey(), true),
//             ],
//         );

//         let mut transaction = Transaction::new_with_payer(&[set_active_instruction], Some(&ctx.payer.pubkey()));
//         transaction.sign(&[&ctx.payer, &ramp_keypair], ctx.last_blockhash);
        
//         let result = ctx.banks_client.process_transaction(transaction).await;
//         assert!(result.is_ok(), "Set active should succeed");
//         println!("Set active test passed!");

//         // First deposit some SOL to the ramp account so it has funds to withdraw
//         ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
//         let off_ramp_deposit_native_instruction = Instruction::new_with_borsh(
//             program_id,
//             &processors::RampInstruction {
//                 instruction: processors::Instruction::OffRampDepositNative(
//                     OffRampDepositNativeInstruction {
//                         amount: 1000,
//                         region: models::Region::KEN,
//                         medium: models::Medium::Primary,
//                         data: vec![],
//                     }
//                 ),
//             },
//             vec![
//                 AccountMeta::new(ramp_keypair.pubkey(), false),
//                 AccountMeta::new(ctx.payer.pubkey(), true),
//                 AccountMeta::new_readonly(system_program::id(), false),
//             ],
//         );

//         let mut transaction = Transaction::new_with_payer(&[off_ramp_deposit_native_instruction], Some(&ctx.payer.pubkey()));
//         transaction.sign(&[&ctx.payer], ctx.last_blockhash);
        
//         let result = ctx.banks_client.process_transaction(transaction).await;
//         assert!(result.is_ok(), "Off ramp deposit native should succeed");
//         println!("Off ramp deposit native test passed!");

//         // Now withdraw some SOL from the ramp account
//         ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
//         let on_ramp_withdraw_native_instruction = Instruction::new_with_borsh(
//             program_id,
//             &processors::RampInstruction {
//                 instruction: processors::Instruction::OnRampWithdrawNative(
//                     OnRampWithdrawNativeInstruction {
//                         amount: 100,
//                     }
//                 ),
//             },
//             vec![
//                 AccountMeta::new(ramp_keypair.pubkey(), true),
//                 AccountMeta::new(ctx.payer.pubkey(), false),
//             ],
//         );

//         let mut transaction = Transaction::new_with_payer(&[on_ramp_withdraw_native_instruction], Some(&ctx.payer.pubkey()));
//         transaction.sign(&[&ctx.payer, &ramp_keypair], ctx.last_blockhash);
        
//         let result = ctx.banks_client.process_transaction(transaction).await;
//         assert!(result.is_ok(), "On ramp withdraw native should succeed: {:?}", result.err());
//         println!("On ramp withdraw native test passed!");
        
//     }
// }
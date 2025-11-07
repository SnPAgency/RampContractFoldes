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
    use std::collections::HashMap;

    use borsh::BorshDeserialize as _;
    use mpl_token_metadata::accounts::Metadata;
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_commitment_config::CommitmentConfig;
    use solana_program::example_mocks::solana_sdk::system_program;
    use solana_sdk::{
        account::Account, message::{
            AccountMeta,
            Instruction
        }, pubkey::Pubkey, signature::{
            Keypair,
            Signer
        }, signer::EncodableKey, transaction::Transaction
    };
    use solana_system_interface::instruction::create_account;
    use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
    //use spl_token_2022_interface::{
    //    extension::{
    //        metadata_pointer::{
    //            instruction::initialize as initialize_metadata_pointer, MetadataPointer,
    //        },
    //        BaseStateWithExtensions, ExtensionType, StateWithExtensions,
    //    },
    //    instruction::initialize_mint,
    //    state::Mint,
    //    ID as TOKEN_2022_PROGRAM_ID,
    //};
    use spl_token_interface::state::Mint;
    use spl_token_metadata_interface::{
        instruction::{initialize as initialize_token_metadata, update_field},
        state::{Field, TokenMetadata},

    };
    use crate::{instructions::{AddAssetsInstruction, InitializeProgramInstruction, RemoveAssetsInstruction, SetActiveInstruction, SetAssetFeeInstruction, SetNativeFeePercentageInstruction, SetOwnerInstruction}, processors, state::RampState};
    use mollusk_svm::{result::Check, Mollusk, program::keyed_account_for_system_program as mollusk_system_program};
    use mollusk_svm_programs_token::{token, token2022, associated_token};
    use dotenv::dotenv;
    

    #[test]
    fn set_up() {

        let mut client = Mollusk::default();

        token::add_program(&mut client);
        token2022::add_program(&mut client);
        associated_token::add_program(&mut client);

        let payer = (Pubkey::new_unique(), Account::new(
            1000000000000000,
            0,
            &mollusk_system_program().0
        ));
        let mint = (Pubkey::new_unique(), 
            token::create_account_for_mint(
            Mint {
                    mint_authority: Some(payer.0).into(),
                    supply: 100000000000,
                    freeze_authority: Some(payer.0).into(),
                    decimals: 9,
                    is_initialized: true
                }
            )
        );

        let mint_vault = (
            associated_token::create_account_for_associated_token_account(
                spl_token::state::Account {
                    mint: mint.0,
                    owner: payer.0,
                    amount: 100000000000,
                    delegate: None.into(),
                    state: spl_token::state::AccountState::Initialized,
                    is_native: None.into(),
                    delegated_amount: 0,
                    close_authority: None.into()
                }
            )
        );

        let metadata  = TokenMetadata {
            update_authority: Some(payer.0).try_into().unwrap(),
            mint: mint.0,
            name: "USDT COin".to_string(),
            symbol : "USDT".to_string(),
            uri : "https://example.com/image.json".to_string(),
            additional_metadata: vec![("some".to_string(),"desc".to_string())]
        };

        let mut token_metadata = vec![0u8; metadata.tlv_size_of().unwrap()];
        
        
        let metadata_account = (Pubkey::new_unique(),
            Account {
                lamports: 0,
                data: borsh::to_vec(&metadata).unwrap(),
                owner: token::ID,
                executable: false,
                rent_epoch: 0
            }
        );

        let account = [
            mint.clone(),
            mint_vault.clone(),
            payer.clone()
        ];
        let mint_instructions = spl_token_interface::instruction::mint_to(
            &token::ID,
            &mint.0,
            &mint_vault.0,
            &payer.0,
            &vec![],
            100000000000
        ).unwrap();

        let result = client.process_instruction(&mint_instructions, &account);
        assert!(result.program_result.is_ok(), "Failed to mint tokens");
    }

    #[test]
    fn test_initialize_ramp() {
        let ramp_program_id = Pubkey::new_unique();
        let ramp_account=  (
            Pubkey::new_unique(),
            Account::default()
        );
        let payer = Keypair::new();
        let payer_account = Account::new(
            1000000000000000,
            0,
            &mollusk_system_program().0
        );

        let client = Mollusk::new(&ramp_program_id, "target/deploy/ramp_solana");

        let initialize_instruction = InitializeProgramInstruction {
            vault_address: Pubkey::new_unique(),
            native_fee_percentage: 10,
        };

        println!("Initialize instruction: {:?}", system_program::id());
        let instruction = Instruction::new_with_borsh(
            ramp_program_id,
            &processors::Instruction::InitializeProgram(initialize_instruction),
            vec![
                AccountMeta::new(ramp_account.0, true),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );
        let accounts = [
            (payer.pubkey(), payer_account),
            (ramp_account.0, ramp_account.1),
            (mollusk_system_program()),
        ];
        let result = client.process_instruction(&instruction, &accounts);
        //assert!(result.program_result.is_ok(), "Failed to initialize ramp");
        client.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::success()]
        );

    }


    #[test]
    fn test_set_active() {
        let ramp_program_id = Pubkey::new_unique();
        let ramp_account= (Pubkey::new_unique(), Account::default());
        let payer = (Pubkey::new_unique(), Account::new(
            1000000000000000,
            0,
            &mollusk_system_program().0
        ));
        let system_program = mollusk_system_program();

        let mollusk = Mollusk::new(&ramp_program_id, "target/deploy/ramp_solana");

        let mut accounts  = HashMap::new();
        accounts.insert(ramp_account.0, ramp_account.1);
        accounts.insert(payer.0, payer.1);
        accounts.insert(system_program.0, system_program.1);

        let initialize_instruction = InitializeProgramInstruction {
            vault_address: Pubkey::new_unique(),
            native_fee_percentage: 10,
        };
        let client = mollusk.with_context(accounts);


        println!("Initialize instruction: {:?}", system_program::id());
        let instruction = Instruction::new_with_borsh(
            ramp_program_id,
            &processors::Instruction::InitializeProgram(initialize_instruction),
            vec![
                AccountMeta::new(ramp_account.0, true),
                AccountMeta::new(payer.0, true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        client.process_and_validate_instruction(
            &instruction,
            &[Check::success()]
        );

        let set_active_instruction = SetActiveInstruction {
            is_active: true,
        };
        let active_instruction = Instruction::new_with_borsh(
            ramp_program_id,
            &processors::Instruction::SetActive(set_active_instruction),
            vec![
                AccountMeta::new(ramp_account.0, true),
                AccountMeta::new(payer.0, true),
            ],
        );
        client.process_and_validate_instruction(
            &active_instruction,
            &[Check::success()]
        );
    }

    #[test]
    fn test_add_asset() {
        let ramp_program_id = Pubkey::new_unique();
        let ramp_account= (Pubkey::new_unique(), Account::default());
        let payer = (Pubkey::new_unique(), Account::new(
            1000000000000000,
            0,
            &mollusk_system_program().0
        ));
        let mint = (Pubkey::new_unique(), 
            token::create_account_for_mint(
            Mint {
                    mint_authority: Some(payer.0).into(),
                    supply: 100000000000,
                    freeze_authority: Some(payer.0).into(),
                    decimals: 9,
                    is_initialized: true
                }
            )
        );

        let mint_vault = 
            associated_token::create_account_for_associated_token_account(
                spl_token::state::Account {
                    mint: mint.0,
                    owner: payer.0,
                    amount: 100000000000,
                    delegate: None.into(),
                    state: spl_token::state::AccountState::Initialized,
                    is_native: None.into(),
                    delegated_amount: 0,
                    close_authority: None.into()
                }
            );

        let ramp_token_account = 
            associated_token::create_account_for_associated_token_account(
                spl_token::state::Account {
                    mint: mint.0,
                    owner: ramp_account.0,
                    amount: 100000000000,
                    delegate: None.into(),
                    state: spl_token::state::AccountState::Initialized,
                    is_native: None.into(),
                    delegated_amount: 0,
                    close_authority: None.into()
                }
            );

        let token_program = token::keyed_account();

        let system_program = mollusk_system_program();

        let associated_token_program = associated_token::keyed_account();

        let mut mollusk = Mollusk::new(&ramp_program_id, "target/deploy/ramp_solana");

        token::add_program(&mut mollusk);
        associated_token::add_program(&mut mollusk);

        let mut accounts  = HashMap::new();
        //ramp account
        accounts.insert(ramp_account.0, ramp_account.1);

        //mint
        accounts.insert(mint.0, mint.1);

        //payer
        accounts.insert(payer.0, payer.1);

        //token_program
        accounts.insert(token_program.0, token_program.1);

        //system program
        accounts.insert(system_program.0, system_program.1);

        //associated token program
        accounts.insert(associated_token_program.0, associated_token_program.1);

        //owner token account
        accounts.insert(mint_vault.0, mint_vault.1);

        //ramp token account
        accounts.insert(ramp_token_account.0, ramp_token_account.1);

        let initialize_instruction = InitializeProgramInstruction {
            vault_address: Pubkey::new_unique(),
            native_fee_percentage: 10,
        };
        let client = mollusk.with_context(accounts);


        println!("Initialize instruction: {:?}", system_program::id());
        let instruction = Instruction::new_with_borsh(
            ramp_program_id,
            &processors::Instruction::InitializeProgram(initialize_instruction),
            vec![
                AccountMeta::new(ramp_account.0, true),
                AccountMeta::new(payer.0, true),
                AccountMeta::new_readonly(system_program.0, false),
            ],
        );

        client.process_and_validate_instruction(
            &instruction,
            &[Check::success()]
        );

        let add_asset_instruction = AddAssetsInstruction {
            initial_amount: 100000000,
            fee_percentage: 10,
        };

        let instruction = Instruction::new_with_borsh(
            ramp_program_id,
            &processors::Instruction::AddAssets(add_asset_instruction),
            vec![
                AccountMeta::new(ramp_account.0, false),
                AccountMeta::new(mint.0, false),
                AccountMeta::new(payer.0, true),
                AccountMeta::new_readonly(token::ID, false),
                AccountMeta::new_readonly(system_program.0, false),
                AccountMeta::new_readonly(associated_token_program.0, false),
                AccountMeta::new(mint_vault.0, false),
                AccountMeta::new(ramp_token_account.0, false),
            ],
        );

        client.process_and_validate_instruction(
            &instruction,
            &[Check::success()]
        );
    }

        #[test]
    fn test_set_asset_fee() {
        let ramp_program_id = Pubkey::new_unique();
        let ramp_account= (Pubkey::new_unique(), Account::default());
        let payer = (Pubkey::new_unique(), Account::new(
            1000000000000000,
            0,
            &mollusk_system_program().0
        ));
        let mint = (Pubkey::new_unique(), 
            token::create_account_for_mint(
            Mint {
                    mint_authority: Some(payer.0).into(),
                    supply: 100000000000,
                    freeze_authority: Some(payer.0).into(),
                    decimals: 9,
                    is_initialized: true
                }
            )
        );

        let mint_vault = 
            associated_token::create_account_for_associated_token_account(
                spl_token::state::Account {
                    mint: mint.0,
                    owner: payer.0,
                    amount: 100000000000,
                    delegate: None.into(),
                    state: spl_token::state::AccountState::Initialized,
                    is_native: None.into(),
                    delegated_amount: 0,
                    close_authority: None.into()
                }
            );

        let ramp_token_account = 
            associated_token::create_account_for_associated_token_account(
                spl_token::state::Account {
                    mint: mint.0,
                    owner: ramp_account.0,
                    amount: 100000000000,
                    delegate: None.into(),
                    state: spl_token::state::AccountState::Initialized,
                    is_native: None.into(),
                    delegated_amount: 0,
                    close_authority: None.into()
                }
            );

        let token_program = token::keyed_account();

        let system_program = mollusk_system_program();

        let associated_token_program = associated_token::keyed_account();

        let mut mollusk = Mollusk::new(&ramp_program_id, "target/deploy/ramp_solana");

        token::add_program(&mut mollusk);
        associated_token::add_program(&mut mollusk);

        let mut accounts  = HashMap::new();
        //ramp account
        accounts.insert(ramp_account.0, ramp_account.1);

        //mint
        accounts.insert(mint.0, mint.1);

        //payer
        accounts.insert(payer.0, payer.1);

        //token_program
        accounts.insert(token_program.0, token_program.1);

        //system program
        accounts.insert(system_program.0, system_program.1);

        //associated token program
        accounts.insert(associated_token_program.0, associated_token_program.1);

        //owner token account
        accounts.insert(mint_vault.0, mint_vault.1);

        //ramp token account
        accounts.insert(ramp_token_account.0, ramp_token_account.1);

        let initialize_instruction = InitializeProgramInstruction {
            vault_address: Pubkey::new_unique(),
            native_fee_percentage: 10,
        };
        let client = mollusk.with_context(accounts);


        println!("Initialize instruction: {:?}", system_program::id());
        let instruction = Instruction::new_with_borsh(
            ramp_program_id,
            &processors::Instruction::InitializeProgram(initialize_instruction),
            vec![
                AccountMeta::new(ramp_account.0, true),
                AccountMeta::new(payer.0, true),
                AccountMeta::new_readonly(system_program.0, false),
            ],
        );

        client.process_and_validate_instruction(
            &instruction,
            &[Check::success()]
        );

        let add_asset_instruction = AddAssetsInstruction {
            initial_amount: 100000000,
            fee_percentage: 10,
        };

        let instruction = Instruction::new_with_borsh(
            ramp_program_id,
            &processors::Instruction::AddAssets(add_asset_instruction),
            vec![
                AccountMeta::new(ramp_account.0, false),
                AccountMeta::new(mint.0, false),
                AccountMeta::new(payer.0, true),
                AccountMeta::new_readonly(token::ID, false),
                AccountMeta::new_readonly(system_program.0, false),
                AccountMeta::new_readonly(associated_token_program.0, false),
                AccountMeta::new(mint_vault.0, false),
                AccountMeta::new(ramp_token_account.0, false),
            ],
        );

        client.process_and_validate_instruction(
            &instruction,
            &[Check::success()]
        );
        let set_asset_fee_instruction = SetAssetFeeInstruction {
            asset_mint: mint.0,
            fee_percentage: 10,
        };
        let instruction = Instruction::new_with_borsh(
            ramp_program_id,
            &processors::Instruction::SetAssetFee(set_asset_fee_instruction),
            vec![
                AccountMeta::new(ramp_account.0, true),
                AccountMeta::new(payer.0, true),
            ],
        );
        client.process_and_validate_instruction(
            &instruction,
            &[Check::success()]
        );
    }

    #[test]
    fn test_remove_asset() {
        let ramp_program_id = Pubkey::new_unique();
        let ramp_account= (Pubkey::new_unique(), Account::default());
        let payer = (Pubkey::new_unique(), Account::new(
            1000000000000000,
            0,
            &mollusk_system_program().0
        ));
        let mint = (Pubkey::new_unique(), 
            token::create_account_for_mint(
            Mint {
                    mint_authority: Some(payer.0).into(),
                    supply: 100000000000,
                    freeze_authority: Some(payer.0).into(),
                    decimals: 9,
                    is_initialized: true
                }
            )
        );

        let mint_vault = 
            associated_token::create_account_for_associated_token_account(
                spl_token::state::Account {
                    mint: mint.0,
                    owner: payer.0,
                    amount: 100000000000,
                    delegate: None.into(),
                    state: spl_token::state::AccountState::Initialized,
                    is_native: None.into(),
                    delegated_amount: 0,
                    close_authority: None.into()
                }
            );

        let ramp_token_account = 
            associated_token::create_account_for_associated_token_account(
                spl_token::state::Account {
                    mint: mint.0,
                    owner: ramp_account.0,
                    amount: 100000000000,
                    delegate: None.into(),
                    state: spl_token::state::AccountState::Initialized,
                    is_native: None.into(),
                    delegated_amount: 0,
                    close_authority: None.into()
                }
            );

        let token_program = token::keyed_account();

        let system_program = mollusk_system_program();

        let associated_token_program = associated_token::keyed_account();

        let mut mollusk = Mollusk::new(&ramp_program_id, "target/deploy/ramp_solana");

        token::add_program(&mut mollusk);
        associated_token::add_program(&mut mollusk);

        let mut accounts  = HashMap::new();
        //ramp account
        accounts.insert(ramp_account.0, ramp_account.1);

        //mint
        accounts.insert(mint.0, mint.1);

        //payer
        accounts.insert(payer.0, payer.1);

        //token_program
        accounts.insert(token_program.0, token_program.1);

        //system program
        accounts.insert(system_program.0, system_program.1);

        //associated token program
        accounts.insert(associated_token_program.0, associated_token_program.1);

        //owner token account
        accounts.insert(mint_vault.0, mint_vault.1);

        //ramp token account
        accounts.insert(ramp_token_account.0, ramp_token_account.1);

        let initialize_instruction = InitializeProgramInstruction {
            vault_address: Pubkey::new_unique(),
            native_fee_percentage: 10,
        };
        let client = mollusk.with_context(accounts);


        println!("Initialize instruction: {:?}", system_program::id());
        let instruction = Instruction::new_with_borsh(
            ramp_program_id,
            &processors::Instruction::InitializeProgram(initialize_instruction),
            vec![
                AccountMeta::new(ramp_account.0, true),
                AccountMeta::new(payer.0, true),
                AccountMeta::new_readonly(system_program.0, false),
            ],
        );

        client.process_and_validate_instruction(
            &instruction,
            &[Check::success()]
        );

        let add_asset_instruction = AddAssetsInstruction {
            initial_amount: 100000000,
            fee_percentage: 10,
        };

        let instruction = Instruction::new_with_borsh(
            ramp_program_id,
            &processors::Instruction::AddAssets(add_asset_instruction),
            vec![
                AccountMeta::new(ramp_account.0, false),
                AccountMeta::new(mint.0, false),
                AccountMeta::new(payer.0, true),
                AccountMeta::new_readonly(token::ID, false),
                AccountMeta::new_readonly(system_program.0, false),
                AccountMeta::new_readonly(associated_token_program.0, false),
                AccountMeta::new(mint_vault.0, false),
                AccountMeta::new(ramp_token_account.0, false),
            ],
        );

        client.process_and_validate_instruction(
            &instruction,
            &[Check::success()]
        );
        let remove_asset_instruction = RemoveAssetsInstruction {};

        let instruction = Instruction::new_with_borsh(
            ramp_program_id,
            &processors::Instruction::RemoveAssets(remove_asset_instruction),
            vec![
                AccountMeta::new(ramp_account.0, true),
                AccountMeta::new(ramp_token_account.0, true),
                AccountMeta::new(mint.0, false),
                AccountMeta::new(payer.0, true),
                AccountMeta::new(mint_vault.0, false),
                AccountMeta::new_readonly(token_program.0, false),
            ],
        );

        client.process_and_validate_instruction(
            &instruction,
            &[Check::success()]
        );
    }

    #[test]
    fn test_set_owner() {

        let ramp_program_id = Pubkey::new_unique();
        let ramp_account= (Pubkey::new_unique(), Account::default());
        let payer = (Pubkey::new_unique(), Account::new(
            1000000000000000,
            0,
            &mollusk_system_program().0
        ));
        let system_program = mollusk_system_program();

        let mollusk = Mollusk::new(&ramp_program_id, "target/deploy/ramp_solana");

        let mut accounts  = HashMap::new();
        accounts.insert(ramp_account.0, ramp_account.1);
        accounts.insert(payer.0, payer.1);
        accounts.insert(system_program.0, system_program.1);
        let client = mollusk.with_context(accounts);

        let initialize_instruction = InitializeProgramInstruction {
            vault_address: Pubkey::new_unique(),
            native_fee_percentage: 10,
        };
        let instruction = Instruction::new_with_borsh(
            ramp_program_id,
            &processors::Instruction::InitializeProgram(initialize_instruction),
            vec![
                AccountMeta::new(ramp_account.0, true),
                AccountMeta::new(payer.0, true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );
        client.process_and_validate_instruction(
            &instruction,
            &[Check::success()]
        );
        let set_owner_instruction = SetOwnerInstruction {
            new_owner: Pubkey::new_unique(),
        };
        let instruction = Instruction::new_with_borsh(
            ramp_program_id,
            &processors::Instruction::SetOwner(set_owner_instruction),
            vec![
                AccountMeta::new(ramp_account.0, false),
                AccountMeta::new(payer.0, true),
            ],
        );
        client.process_and_validate_instruction(
            &instruction,
            &[Check::success()]
        );
    }

    #[test]
    fn test_set_native_fee() {
        let ramp_program_id = Pubkey::new_unique();
        let ramp_account= (Pubkey::new_unique(), Account::default());
        let payer = (Pubkey::new_unique(), Account::new(
            1000000000000000,
            0,
            &mollusk_system_program().0
        ));
        let system_program = mollusk_system_program();

        let mollusk = Mollusk::new(&ramp_program_id, "target/deploy/ramp_solana");

        let mut accounts  = HashMap::new();
        accounts.insert(ramp_account.0, ramp_account.1);
        accounts.insert(payer.0, payer.1);
        accounts.insert(system_program.0, system_program.1);
        let client = mollusk.with_context(accounts);

        let initialize_instruction = InitializeProgramInstruction {
            vault_address: Pubkey::new_unique(),
            native_fee_percentage: 10,
        };
        let instruction = Instruction::new_with_borsh(
            ramp_program_id,
            &processors::Instruction::InitializeProgram(initialize_instruction),
            vec![
                AccountMeta::new(ramp_account.0, true),
                AccountMeta::new(payer.0, true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );
        client.process_and_validate_instruction(
            &instruction,
            &[Check::success()]
        );
        let set_native_fee_instruction = SetNativeFeePercentageInstruction {
            fee_percentage: 20,
        };
        let instruction = Instruction::new_with_borsh(
            ramp_program_id,
            &processors::Instruction::SetNativeFeePercentage(set_native_fee_instruction),
            vec![
                AccountMeta::new(ramp_account.0, false),
                AccountMeta::new(payer.0, true),
            ],
        );
        client.process_and_validate_instruction(
            &instruction,
            &[Check::success()]
        );
    }
}
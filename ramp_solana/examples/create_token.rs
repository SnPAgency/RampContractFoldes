use solana_sdk::{
    signature::{Keypair, Signer}, signer::EncodableKey, transaction::Transaction
};
use solana_commitment_config::CommitmentConfig;
use solana_system_interface::instruction::create_account;
//use spl_associated_token_account::get_associated_token_address_with_program_id;
//use spl_token::instruction::initialize_account;
//use spl_associated_token_account_interface::instruction::create_associated_token_account;
//use spl_associated_token_account::get_associated_token_address;

use solana_client::nonblocking::rpc_client::RpcClient;
use once_cell::sync::Lazy;
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

static SOLANA_MAINNET_URL: Lazy<String> = Lazy::new(|| "https://api.mainnet-beta.solana.com".to_string());
static SOLANA_TESTNET_URL: Lazy<String> = Lazy::new(|| "https://api.testnet.solana.com".to_string());
static SOLANA_LOCAL_URL: Lazy<String> = Lazy::new(|| "http://localhost:8899".to_string());
static SOLANA_DEVNET_URL: Lazy<String> = Lazy::new(|| "https://api.devnet.solana.com".to_string());

pub fn get_client(network: &str) -> RpcClient {
    match network {
        "mainnet" => RpcClient::new_with_commitment(
            SOLANA_MAINNET_URL.clone(),
            CommitmentConfig::confirmed()
        ),
        "testnet" => RpcClient::new_with_commitment(
            SOLANA_TESTNET_URL.clone(),
            CommitmentConfig::confirmed()
        ),
        "local" => RpcClient::new_with_commitment(
            SOLANA_LOCAL_URL.clone(),
            CommitmentConfig::confirmed()
        ),
        "devnet" => RpcClient::new_with_commitment(
            SOLANA_DEVNET_URL.clone(),
            CommitmentConfig::confirmed()
        ),
        _ => panic!("Invalid network"),
    }
}

#[tokio::main]
async fn main() {
    let client = get_client("local");
    let path = std::env::var_os("DEV_KEY_PAIR").unwrap();
    let mint =  Keypair::read_from_file("../../token_keypair.json").unwrap_or_else(
        |_| {
            let key = Keypair::new();
            key.write_to_file("../../token_keypair.json").unwrap();
            key
        }
    );

    let fee_payer = Keypair::read_from_file(path.clone()).unwrap();
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
            client.get_latest_blockhash().await.unwrap(),
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

        //let associated_token_address = get_associated_token_address_with_program_id(
        //    &fee_payer.pubkey(),
        //    &mint.pubkey(),
        //    &TOKEN_2022_PROGRAM_ID,
        //);

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
            client.get_latest_blockhash().await.unwrap(),
        );
        let create_associated_token_account_result = client.send_and_confirm_transaction(&create_associated_token_account_transaction).await;

        assert!(create_associated_token_account_result.is_ok(), "{:?}", create_associated_token_account_result.err().unwrap());
}
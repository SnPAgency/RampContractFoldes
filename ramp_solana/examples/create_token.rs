use solana_sdk::{
    program_pack::Pack,signature::{Keypair, Signer}, signer::EncodableKey, transaction::Transaction
};
use solana_commitment_config::CommitmentConfig;
use solana_system_interface::instruction::create_account;
use solana_client::nonblocking::rpc_client::RpcClient;
use once_cell::sync::Lazy;
use spl_token::{instruction::{initialize_mint, initialize_account}, state::{Account, Mint}};
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

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
    dotenv::dotenv().ok();
    let client = get_client("local");
    let mint =  Keypair::read_from_file("keys/token_keypair.json").unwrap_or_else(
        |_| {
            let key = Keypair::new();
            key.write_to_file("keys/token_keypair.json").unwrap();
            key
        }
    );
    let fee_payer = Keypair::read_from_file("keys/dev-keypair.json").unwrap();

    client.request_airdrop(&fee_payer.pubkey(), 1000000000).await.unwrap();
    // Calculate space for mint with metadata pointer and token metadata extensions
    let mint_space = Mint::LEN;
    let mint_rent = client.get_minimum_balance_for_rent_exemption(mint_space).await.unwrap();
    // Instruction to create new account for mint (token22)
    let create_mint_account_instruction = create_account(
        &fee_payer.pubkey(),    // payer
        &mint.pubkey(),         // new account (mint)
        mint_rent,              // lamports
        mint_space as u64,      // space
        &TOKEN_PROGRAM_ID, // program id
    );

    // Instruction to initialize mint account data
    let initialize_mint_instruction = initialize_mint(
        &TOKEN_PROGRAM_ID,    // program id
        &mint.pubkey(),            // mint
        &fee_payer.pubkey(),       // mint authority
        Some(&fee_payer.pubkey()), // freeze authority
        9,                         // decimals
    ).unwrap();

    let mint_tx = Transaction::new_signed_with_payer(
        &[create_mint_account_instruction, initialize_mint_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint],
        client.get_latest_blockhash().await.unwrap(),
    );
    let mint_tx_signature = client.send_and_confirm_transaction(&mint_tx).await;
    assert!(mint_tx_signature.is_ok(), "{:?}", mint_tx_signature.err().unwrap());


    let token_account = Keypair::read_from_file("keys/token_account_keypair.json").unwrap_or_else(
        |_| {
            let key = Keypair::new();
            key.write_to_file("keys/token_account_keypair.json").unwrap();
            key
        }
    );
    let token_account_sape = Account::LEN;
    let token_account_rent = client.get_minimum_balance_for_rent_exemption(token_account_sape).await.unwrap();
    let create_token_account_instruction = create_account(
        &fee_payer.pubkey(),    // payer
        &token_account.pubkey(),         // new account (mint)
        token_account_rent,              // lamports
        token_account_sape as u64,      // space
        &TOKEN_PROGRAM_ID, // program id
    );

    let initialize_token_account_instruction = initialize_account(
        &TOKEN_PROGRAM_ID,    // program id
        &token_account.pubkey(),            // token account
        &mint.pubkey(),       // mint
        &mint.pubkey(), // freeze authority
    ).unwrap();

    let token_account_tx = Transaction::new_signed_with_payer(
        &[create_token_account_instruction, initialize_token_account_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &token_account],
        client.get_latest_blockhash().await.unwrap(),
    );
    let token_account_tx_signature = client.send_and_confirm_transaction(&token_account_tx).await;
    assert!(token_account_tx_signature.is_ok(), "{:?}", token_account_tx_signature.err().unwrap());
}
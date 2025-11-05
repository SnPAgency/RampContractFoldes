use once_cell::sync::Lazy;
use solana_commitment_config::CommitmentConfig;
use solana_client::nonblocking::rpc_client::RpcClient;
//use solana_sdk::message::AccountMeta;
use solana_sdk::{
    //instruction::Instruction,
    signer::EncodableKey,
    transaction::Transaction,
};
use solana_sdk::signature::{Keypair, Signer};
use spl_token_interface::id as token_program;
//use spl_associated_token_account_interface::program::id as associated_token_program;
use spl_associated_token_account::get_associated_token_address;
//use solana_program::config::program::ID as system_program_id;

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
        "devnet" => RpcClient::new_with_commitment(SOLANA_DEVNET_URL.clone(), CommitmentConfig::confirmed()),
        _ => panic!("Invalid network"),
    }
}

#[tokio::main]
async fn main() {
    let client = get_client("local");
    let path = std::env::var_os("DEV_KEY_PAIR").unwrap();
    let signer_keypair = Keypair::read_from_file(path).unwrap();
    let asset_mint_account = Keypair::read_from_file("../../token_keypair.json").unwrap();
    let owner_token_account = get_associated_token_address(
        &signer_keypair.pubkey(),
        &asset_mint_account.pubkey(),
    );

    let mint_instruction = spl_token_interface::instruction::mint_to(
        &token_program(),
        &asset_mint_account.pubkey(),
        &owner_token_account,
        &signer_keypair.pubkey(),
        &[&signer_keypair.pubkey()],
        100000000000000000,
    ).unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[mint_instruction],
        Some(&signer_keypair.pubkey()),
        &[&signer_keypair],
        client.get_latest_blockhash().await.unwrap(),
    );

    let result = client.send_transaction(&transaction).await;
    assert!(result.is_ok(), "{:?}", result.err().unwrap());
}
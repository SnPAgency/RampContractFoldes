use solana_sdk::{
    program_pack::Pack, signature::{Keypair, Signer}, signer::EncodableKey, transaction::Transaction
};
use solana_commitment_config::CommitmentConfig;
use solana_system_interface::instruction::create_account;
use spl_token::instruction::initialize_account;
use spl_token_interface::{id as token_program_id, instruction::initialize_mint, state::Mint};

use solana_client::nonblocking::rpc_client::RpcClient;
use once_cell::sync::Lazy;

static SOLANA_MAINNET_URL: Lazy<String> = Lazy::new(|| "https://api.mainnet-beta.solana.com".to_string());
static SOLANA_TESTNET_URL: Lazy<String> = Lazy::new(|| "https://api.testnet.solana.com".to_string());
static SOLANA_LOCAL_URL: Lazy<String> = Lazy::new(|| "http://localhost:8899".to_string());
static SOLANA_DEVNET_URL: Lazy<String> = Lazy::new(|| "https://api.devnet.solana.com".to_string());

pub fn get_client(network: &str) -> RpcClient {
    match network {
        "mainnet" => RpcClient::new_with_commitment(SOLANA_MAINNET_URL.clone(), CommitmentConfig::confirmed()),
        "testnet" => RpcClient::new_with_commitment(SOLANA_TESTNET_URL.clone(), CommitmentConfig::confirmed()),
        "local" => RpcClient::new_with_commitment(SOLANA_LOCAL_URL.clone(), CommitmentConfig::confirmed()),
        "devnet" => RpcClient::new_with_commitment(SOLANA_DEVNET_URL.clone(), CommitmentConfig::confirmed()),
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

    let keypair = Keypair::read_from_file(path.clone()).unwrap();
    let space = Mint::LEN;

    let rent = client.get_minimum_balance_for_rent_exemption(space).await.unwrap();

    let create_account_instruction = create_account(
        &keypair.pubkey().clone(),
        &mint.pubkey(),
        rent,
        space as u64,
        &token_program_id(),
    );

    let initialize_intructions = initialize_mint(
        &token_program_id(),
        &mint.pubkey(),
        &keypair.pubkey().clone(),
        None,
        9,
    ).unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[create_account_instruction,initialize_intructions],
        Some(&keypair.pubkey()),
        &[keypair, mint],
        client.get_latest_blockhash().await.unwrap(),
    );

    let tx = client.send_and_confirm_transaction(&transaction).await.unwrap();
    println!("Transaction: {}", tx);

    let token_account = Keypair::read_from_file("./token_account_key.json").unwrap_or_else(
        |_| {
            let key = Keypair::new();
            key.write_to_file("./token_account_key.json").unwrap();
            key
        }
    );

    let token_account_space = spl_token_interface::state::Account::LEN;

    let rent = client.get_minimum_balance_for_rent_exemption(token_account_space).await.unwrap();

    let signer_keypair = Keypair::read_from_file(path.clone()).unwrap();
    let mint_keypair = Keypair::read_from_file("../../token_keypair.json").unwrap();
    let create_account_instruction = create_account(
        &signer_keypair.pubkey(),
        &token_account.pubkey().clone(),
        rent,
        token_account_space as u64,
        &token_program_id(),
    );

    let initialize_intructions = initialize_account(
        &token_program_id(),
        &mint_keypair.pubkey(),
        &token_account.pubkey().clone(),
        &signer_keypair.pubkey().clone(),
    ).unwrap();


    let transaction = Transaction::new_signed_with_payer(
        &[create_account_instruction,initialize_intructions],
        Some(&signer_keypair.pubkey()),
        &[signer_keypair, mint_keypair],
        client.get_latest_blockhash().await.unwrap(),
    );

    let tx = client.send_and_confirm_transaction(&transaction).await.unwrap();
    println!("Transaction: {}", tx);

}
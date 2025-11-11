use once_cell::sync::Lazy;
use solana_commitment_config::CommitmentConfig;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{
    signer::EncodableKey,
    transaction::Transaction,
};
use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022_interface::ID as TOKEN_PROGRAM_ID;
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token_2022_interface::instruction::mint_to;

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
    let signer_keypair = Keypair::read_from_file("keys/dev-keypair.json").unwrap();
    let asset_mint_account = Keypair::read_from_file("keys/token_keypair_2022.json").unwrap();
    let ramp_program_id = Keypair::read_from_file("target/deploy/ramp_solana-keypair.json").unwrap().pubkey();

    let owner_token_account = get_associated_token_address_with_program_id(
        &signer_keypair.pubkey(),
        &asset_mint_account.pubkey(),
        &TOKEN_PROGRAM_ID,
    );
    let create_owner_associated_token_account_instructions = create_associated_token_account(
        &signer_keypair.pubkey(),
        &signer_keypair.pubkey(),
        &asset_mint_account.pubkey(),
        &TOKEN_PROGRAM_ID,
    );
    let create_owner_associated_token_account_transaction = Transaction::new_signed_with_payer(
        &[create_owner_associated_token_account_instructions],
        Some(&signer_keypair.pubkey()),
        &[&signer_keypair],
        client.get_latest_blockhash().await.unwrap(),
    );
    let create_owner_associated_token_account_result = client.send_and_confirm_transaction(&create_owner_associated_token_account_transaction).await;
    assert!(create_owner_associated_token_account_result.is_ok(), "{:?}", create_owner_associated_token_account_result.err().unwrap());
    let ramp_account = Pubkey::find_program_address(&[
        b"ramp",signer_keypair.pubkey().as_ref()],
        &ramp_program_id
    );
    let create_ramp_associated_token_account_instructions = create_associated_token_account(
        &signer_keypair.pubkey(),
        &ramp_account.0,
        &asset_mint_account.pubkey(),
        &TOKEN_PROGRAM_ID,
    );
    let create_ramp_associated_token_account_transaction = Transaction::new_signed_with_payer(
        &[create_ramp_associated_token_account_instructions],
        Some(&signer_keypair.pubkey()),
        &[&signer_keypair],
        client.get_latest_blockhash().await.unwrap(),
    );
    let create_ramp_associated_token_account_result = client.send_and_confirm_transaction(&create_ramp_associated_token_account_transaction).await;
    assert!(create_ramp_associated_token_account_result.is_ok(), "{:?}", create_ramp_associated_token_account_result.err().unwrap());
    // Mint tokens to the payer's token account
    let mint_instructions = mint_to(
        &TOKEN_PROGRAM_ID,
        &asset_mint_account.pubkey(),
        &owner_token_account,
        &signer_keypair.pubkey(),
        &[&signer_keypair.pubkey()],
        100000000000000
    ).unwrap();
    let mint_tokens_tx = Transaction::new_signed_with_payer(
        &[mint_instructions],
        Some(&signer_keypair.pubkey()),
        &[&signer_keypair],
        client.get_latest_blockhash().await.unwrap(),
    );
    let mint_result = client.send_and_confirm_transaction(&mint_tokens_tx).await;
    assert!(mint_result.is_ok(), "{:?}", mint_result.err().unwrap());
}
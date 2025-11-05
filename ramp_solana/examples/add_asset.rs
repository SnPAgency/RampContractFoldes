use once_cell::sync::Lazy;
use solana_commitment_config::CommitmentConfig;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::config::program::ID as system_program_id;
use solana_sdk::{
    instruction::{
        Instruction,
        AccountMeta
    },
    signer::EncodableKey,
    transaction::Transaction,
};
use solana_sdk::signature::{Keypair, Signer};
use spl_token_interface::id as token_program;
use spl_associated_token_account_interface::program::id as associated_token_program;
use spl_associated_token_account::get_associated_token_address;

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
    let ramp_keypair = Keypair::read_from_file("../target/deploy/ramp_solana-keypair.json").unwrap();
    let asset_mint_account = Keypair::read_from_file("../../token_keypair.json").unwrap();

    let add_assets_data = ramp_solana::instructions::AddAssetsInstruction {
        initial_amount: 1000000000000000,
        fee_percentage: 10,
    };
    let owner_token_account = get_associated_token_address(
        &signer_keypair.pubkey(),
        &asset_mint_account.pubkey(),
    );
    let ramp_token_account = get_associated_token_address(
        &ramp_keypair.pubkey(),
        &asset_mint_account.pubkey(),
    );

    let accounts = vec![
        AccountMeta::new(ramp_keypair.pubkey(), false),
        AccountMeta::new(asset_mint_account.pubkey(), false),
        AccountMeta::new(signer_keypair.pubkey(), true),
        AccountMeta::new_readonly(token_program(), false),
        AccountMeta::new_readonly(system_program_id, false),
        AccountMeta::new_readonly(associated_token_program(), false),
        AccountMeta::new(owner_token_account, false),
        AccountMeta::new(ramp_token_account, false),
    ];

    let add_assets_instruction = Instruction::new_with_borsh(
        ramp_keypair.pubkey(),
        &add_assets_data,
        accounts
    );
    let mut transaction = Transaction::new_with_payer(
        &[add_assets_instruction],
        Some(&signer_keypair.pubkey())
    );
    transaction.sign(&[&signer_keypair], client.get_latest_blockhash().await.unwrap());

    let result = client.send_transaction(&transaction).await;
    assert!(result.is_ok(), "{:?}", result.err().unwrap());
}

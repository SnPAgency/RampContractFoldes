use once_cell::sync::Lazy;
use ramp_solana::processors;
use solana_commitment_config::CommitmentConfig;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{
        AccountMeta, Instruction
    }, pubkey::Pubkey, signer::EncodableKey, transaction::Transaction
};
use solana_sdk::signature::{Keypair, Signer};
use solana_system_interface::program::ID as system_program_id;

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
    let ramp_program_id = Keypair::read_from_file("target/deploy/ramp_solana-keypair.json").unwrap().pubkey();

    let ramp_account = Pubkey::find_program_address(&[
        b"ramp",signer_keypair.pubkey().as_ref()],
        &ramp_program_id
    );

    let off_ramp_native = ramp_solana::instructions::OffRampDepositNativeInstruction {
        amount: 100000,
        region: ramp_solana::models::Region::EGY,
        medium: ramp_solana::models::Medium::Primary,
        data: b"".to_vec(),
    };
    let accounts = vec![
        AccountMeta::new(ramp_account.0, false),
        AccountMeta::new(signer_keypair.pubkey(), true),
        AccountMeta::new_readonly(system_program_id, false),
    ];
    let off_ramp_native_instruction = Instruction::new_with_borsh(
        ramp_program_id,
        &processors::Instruction::OffRampDepositNative(off_ramp_native),
        accounts
    );
    let off_ramp_native_tx = Transaction::new_signed_with_payer(
        &[off_ramp_native_instruction],
        Some(&signer_keypair.pubkey()),
        &[&signer_keypair],
        client.get_latest_blockhash().await.unwrap(),
    );
    let off_ramp_native_result = client.send_and_confirm_transaction(&off_ramp_native_tx).await;
    assert!(off_ramp_native_result.is_ok(), "{:?}", off_ramp_native_result.err().unwrap());
}
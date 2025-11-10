use ramp_solana::{instructions::{InitializeProgramInstruction, SetActiveInstruction}, processors};
use solana_sdk::{
    instruction::{
        AccountMeta, Instruction
    }, pubkey::Pubkey, signer::EncodableKey, transaction::Transaction
};
use solana_sdk::signature::{Keypair, Signer};
use once_cell::sync::Lazy;
use solana_commitment_config::CommitmentConfig;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_system_interface::program::ID as system_program_id;
use dotenv::dotenv;

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
    dotenv().ok();
    let client = get_client("local");
    let signer_keypair = Keypair::read_from_file("keys/dev-keypair.json").unwrap_or_else(
        |_| {
            let key = Keypair::new();
            key.write_to_file("keys/dev-keypair.json").unwrap();
            key
        }
    );
    client.request_airdrop(&signer_keypair.pubkey(), 100000000000).await.unwrap();

    let ramp_program_id = Keypair::read_from_file("target/deploy/ramp_solana-keypair.json").unwrap().pubkey();


    let ramp_account = Pubkey::find_program_address(&[
        b"ramp",signer_keypair.pubkey().as_ref()],
        &ramp_program_id
    );

    let latest_blockhash = client.get_latest_blockhash().await.unwrap();
    let init_instruction = Instruction::new_with_borsh(
        ramp_program_id,
         &processors::RampInstruction {
             instruction: processors::Instruction::InitializeProgram(
                InitializeProgramInstruction {
                    bump: ramp_account.1,
                    vault_address: signer_keypair.pubkey(),
                    native_fee_percentage: 10,
                }
             ),
         },
         vec![
             AccountMeta::new(ramp_account.0, false),
             AccountMeta::new(signer_keypair.pubkey(), true),
             AccountMeta::new_readonly(system_program_id, false),
         ],
     );
     let init_tx = Transaction::new_signed_with_payer(
        &[init_instruction],
        Some(&signer_keypair.pubkey()),
        &[&signer_keypair],
        latest_blockhash,
    );
    let init_result = client.send_and_confirm_transaction(&init_tx).await;
    assert!(init_result.is_ok(), "{:?}", init_result.err().unwrap());
    println!("Ramp initialized successfully");

    //set active
    let set_active_instruction = Instruction::new_with_borsh(
        ramp_program_id,
         &processors::RampInstruction {
             instruction: processors::Instruction::SetActive(
                SetActiveInstruction {
                    is_active: true,
                }
             ),
         },
         vec![
             AccountMeta::new(ramp_account.0, false),
             AccountMeta::new(signer_keypair.pubkey(), true),
             AccountMeta::new_readonly(system_program_id, false),
         ],
     );
     let set_active_tx = Transaction::new_signed_with_payer(
        &[set_active_instruction],
        Some(&signer_keypair.pubkey()),
        &[&signer_keypair],
        latest_blockhash,
    );
    let set_active_result = client.send_and_confirm_transaction(&set_active_tx).await;
    assert!(set_active_result.is_ok(), "{:?}", set_active_result.err().unwrap());
    println!("Ramp set active successfully");
}

use ramp_solana::{instructions::InitializeProgramInstruction, processors};
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
use once_cell::sync::Lazy;
use solana_commitment_config::CommitmentConfig;
use solana_client::nonblocking::rpc_client::RpcClient;

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
    let signer_keypair = Keypair::read_from_file(path).unwrap();
    let ramp_keypair = Keypair::read_from_file("../target/deploy/ramp_solana-keypair.json").unwrap();

        let latest_blockhash = client.get_latest_blockhash().await.unwrap();

        let init_instruction = Instruction::new_with_borsh(
            ramp_keypair.pubkey(),
             &processors::RampInstruction {
                 instruction: processors::Instruction::InitializeProgram(
                     InitializeProgramInstruction {
                        vault_address: signer_keypair.pubkey(),
                        native_fee_percentage: 10,
                     }
                 ),
             },
             vec![
                 AccountMeta::new(ramp_keypair.pubkey(), true),
                 AccountMeta::new_readonly(signer_keypair.pubkey(), true),
                 AccountMeta::new_readonly(system_program_id, false),
             ],
         );

         let init_tx = Transaction::new_signed_with_payer(
            &[init_instruction],
            Some(&signer_keypair.pubkey()),
            &[&ramp_keypair, &signer_keypair],
            latest_blockhash,
        );
        let init_result = client.send_and_confirm_transaction(&init_tx).await;

        assert!(init_result.is_ok(), "{:?}", init_result.err().unwrap());
}

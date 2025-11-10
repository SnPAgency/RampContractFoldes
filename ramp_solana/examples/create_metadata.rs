use once_cell::sync::Lazy;
use solana_commitment_config::CommitmentConfig;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_system_interface::{instruction::create_account};
use solana_sdk::{
    pubkey::Pubkey, signer::EncodableKey, transaction::Transaction
};
use solana_sdk::signature::{Keypair, Signer};
use spl_token_interface::ID as TOKEN_PROGRAM_ID;
use spl_token_metadata_interface::state::TokenMetadata;
use spl_token_metadata_interface::instruction::{initialize as initialize_token_metadata};

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
    let fee_payer = Keypair::read_from_file("keys/dev-keypair.json").unwrap();
    let mint = Keypair::read_from_file("keys/token_keypair.json").unwrap();

    //let metadata_pda = Pubkey::find_program_address(
    //    &[
    //        "metadata".as_bytes(),
    //        mpl_token_metadata::ID.as_ref(),
    //        &mint.pubkey().as_ref(),
    //    ],
    //    &Pubkey::from_str_const(mpl_token_metadata::ID.to_string().as_str())
    //);

    let metadata = Keypair::read_from_file("keys/metadata_keypair.json").unwrap_or_else(
        |_| {
            let key = Keypair::new();
            key.write_to_file("keys/metadata_keypair.json").unwrap();
            key
        }
    );

    let token_metadata  = TokenMetadata {
        update_authority: Some(fee_payer.pubkey()).try_into().unwrap(),
        mint: mint.pubkey(),
        name: "USDT COin".to_string(),
        symbol : "USDT".to_string(),
        uri : "https://example.com/image.json".to_string(),
        additional_metadata: vec![("some".to_string(),"desc".to_string())]
    };
    let metadata_rent = client.get_minimum_balance_for_rent_exemption(token_metadata.tlv_size_of().unwrap()).await.unwrap();
    // Instruction to initialize token metadata
    let initialize_metadata_instruction = initialize_token_metadata(
        &TOKEN_PROGRAM_ID,            
        &metadata.pubkey(),                    
        &fee_payer.pubkey(),               
        &mint.pubkey(),                    
        &fee_payer.pubkey(),               
        token_metadata.name.to_string(),   
        token_metadata.symbol.to_string(), 
        token_metadata.uri.to_string(),
    );

    let accp = Pubkey::try_from(*mpl_token_metadata::ID.as_array()).unwrap();
    let create_metadata_account_instruction = create_account(
        &fee_payer.pubkey(),    
        &metadata.pubkey(),         
        metadata_rent,              
        token_metadata.tlv_size_of().unwrap() as u64,      
        &accp, 
    );

    let met_acc_tx = Transaction::new_signed_with_payer(
        &[create_metadata_account_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &metadata],
        client.get_latest_blockhash().await.unwrap(),
    );
    let met_acc_tx_signature = client.send_and_confirm_transaction(&met_acc_tx).await;
    assert!(met_acc_tx_signature.is_ok(), "{:?}", met_acc_tx_signature.err().unwrap());
    let met_ix = vec![initialize_metadata_instruction];

    let metadata_tx = Transaction::new_signed_with_payer(
        &met_ix,
        Some(&fee_payer.pubkey()),
        &[&fee_payer],
        client.get_latest_blockhash().await.unwrap(),
    );
    let metadata_tx_signature = client.send_and_confirm_transaction(&metadata_tx).await;
    assert!(metadata_tx_signature.is_ok(), "{:?}", metadata_tx_signature.err().unwrap());
}

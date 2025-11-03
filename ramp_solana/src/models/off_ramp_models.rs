use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;


#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum Region {
    KEN,
    NGA,
    UGA,
    RWA,
    GHN,
    EGY,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum Medium {
    Primary,
    Secondary,
    Tertiary,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct RampDeposit {
    pub asset: Pubkey,
    pub asset_name: String,
    pub amount: u64,
    pub sender: Pubkey,
    pub medium: Medium,
    pub region: Region,
    pub data: Vec<u8>,
}


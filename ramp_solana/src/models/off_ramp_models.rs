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
    pub amount: u64,
    pub region: Region,
    pub medium: Medium,
    pub data: Vec<u8>,
}


use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RampState {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub is_active: bool,
    pub assets: Vec<Pubkey>,
}

impl Default for RampState {
    fn default() -> Self {
        Self {
            is_initialized: false,
            owner: Pubkey::default(),
            is_active: false,
            assets: Vec::new(),
        }
    }
}
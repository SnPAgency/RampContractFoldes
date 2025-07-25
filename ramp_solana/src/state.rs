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

impl RampState {
    pub fn get_space_with_assets(max_assets: usize) -> usize {
        // Bool (1) + Pubkey (32) + Bool (1) + Vec len (4) + N * Pubkey (32)
        1 + 32 + 1 + 4 + (max_assets * 32)
    }
}
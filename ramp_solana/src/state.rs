use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AssetEntry {
    pub asset: Pubkey,
    pub info: AssetInfo,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RampState {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub is_active: bool,
    pub vault_address: Pubkey,
    pub asset_entries: Vec<AssetEntry>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct AssetInfo {
    pub asset_fee_percentage: u128,
    pub asset_revenue: u128,
}

impl AssetInfo {
    pub fn new(fee_percentage: u128) -> Self {
        Self {
            asset_fee_percentage: fee_percentage,
            asset_revenue: 0,
        }
    }

    pub fn add_revenue(&mut self, amount: u128) {
        self.asset_revenue = self.asset_revenue.saturating_add(amount);
    }

    pub fn get_fee_percentage(&self) -> u128 {
        self.asset_fee_percentage
    }

    pub fn get_revenue(&self) -> u128 {
        self.asset_revenue
    }
}

impl Default for RampState {
    fn default() -> Self {
        Self {
            is_initialized: false,
            owner: Pubkey::default(),
            is_active: false,
            vault_address: Pubkey::default(),
            asset_entries: Vec::new(),
        }
    }
}

impl RampState {
    pub fn is_allowed_asset(&self, asset: &Pubkey) -> bool {
        self.asset_entries.iter().any(|entry| entry.asset == *asset)
    }

    pub fn get_asset_info(&self, asset: &Pubkey) -> Option<&AssetInfo> {
        self.asset_entries.iter()
            .find(|entry| entry.asset == *asset)
            .map(|entry| &entry.info)
    }

    pub fn get_assets(&self) -> Vec<Pubkey> {
        self.asset_entries.iter().map(|entry| entry.asset).collect()
    }

    pub fn get_space_with_assets(max_assets: usize) -> usize {
        // Very generous space calculation for Solana compatibility
        // Account for discriminator (8) + all fields with padding
        let discriminator = 8;
        let base_size = 1 + 32 + 1 + 32 + 4; // is_initialized + owner + is_active + vault_address + vec len
        let asset_entry_size = 32 + 32; // Pubkey (32) + AssetInfo (u128 + u128 = 32)
        let entries_size = max_assets * asset_entry_size;
        //let generous_padding = 500; // Very generous padding for serialization overhead
        discriminator + base_size + entries_size// + generous_padding
    }

    pub fn add_asset(&mut self, asset: Pubkey, fee_percentage: u128) -> Result<(), &'static str> {
        if self.is_allowed_asset(&asset) {
            return Err("Asset already exists");
        }
        self.asset_entries.push(AssetEntry {
            asset,
            info: AssetInfo::new(fee_percentage),
        });
        Ok(())
    }

    pub fn remove_asset(&mut self, asset: &Pubkey) -> Result<(), &'static str> {
        let initial_len = self.asset_entries.len();
        self.asset_entries.retain(|entry| entry.asset != *asset);
        if self.asset_entries.len() == initial_len {
            return Err("Asset not found");
        }
        Ok(())
    }
}
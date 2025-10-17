use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub struct AssetEntry {
    pub asset: Pubkey,
    pub info: AssetInfo,
}

impl Default for AssetEntry {
    fn default() -> Self {
        Self {
            asset: Pubkey::default(),
            info: AssetInfo::new(0),
        }
    }
}

impl AssetEntry {
    pub fn remove(&mut self) {
        self.asset = Pubkey::default();
        self.info = AssetInfo::new(0);
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub struct RampState {
    pub owner: Pubkey,
    pub is_active: bool,
    pub vault_address: Pubkey,
    pub asset_entries: [AssetEntry; 10],
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Copy)]
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

    #[cfg(test)]
    pub fn get_revenue(&self) -> u128 {
        self.asset_revenue
    }
}

impl Default for RampState {
    fn default() -> Self {
        Self {
            owner: Pubkey::default(),
            is_active: false,
            vault_address: Pubkey::default(),
            asset_entries: [AssetEntry::default(); 10],
        }
    }
}

impl RampState {
    pub fn is_allowed_asset(&self, asset: &Pubkey) -> bool {
        if asset == &Pubkey::default() {
            return false;
        }
        self.asset_entries.iter().any(|entry| entry.asset == *asset)
    }

    pub fn get_asset_info(&mut self, asset: &Pubkey) -> Option<&mut AssetInfo> {
        self.asset_entries.iter_mut()
            .find(|entry| entry.asset == *asset)
            .map(|entry| &mut entry.info)
    }

    pub fn get_asset_info_ref(&self, asset: &Pubkey) -> Option<&AssetInfo> {
        self.asset_entries.iter()
            .find(|entry| entry.asset == *asset)
            .map(|entry| &entry.info)
    }

    #[cfg(test)]
    pub fn get_assets(&self) -> Vec<Pubkey> {
        self.asset_entries.iter().map(|entry| entry.asset).collect()
    }

    pub fn set_active(&mut self, is_active: bool) {
        self.is_active = is_active;
    }

    pub fn set_new_owner(&mut self, owner: Pubkey) {
        self.owner = owner;
    }

    pub fn add_asset(&mut self, asset: Pubkey, fee_percentage: u128) -> Result<(), &'static str> {
        if self.is_allowed_asset(&asset) {
            return Err("Asset already exists");
        }
        let asset_entry = AssetEntry {
            asset,
            info: AssetInfo::new(fee_percentage),
        };
        if self.asset_entries.iter_mut().find(|entry| entry.asset == asset).is_some() {
            return Err("Asset already exists");
        }

        //let empty_slot = self.asset_entries.iter_mut().find(|entry| entry.asset == Pubkey::default());
        match self.asset_entries.iter_mut().find(|entry| entry.asset == Pubkey::default()) {
            Some(entry) => {
                *entry = asset_entry;
            }
            None => {
                return Err("No empty slot found");
            }
        }
        Ok(())
    }

    pub fn remove_asset(&mut self, asset: &Pubkey) -> Result<(), &'static str> {
        self.asset_entries.iter_mut().find(|entry| entry.asset == *asset).map(|entry| {
            entry.remove();
        });
        Ok(())
    }
}
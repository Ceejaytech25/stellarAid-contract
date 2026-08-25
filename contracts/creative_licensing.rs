// Creative Licensing Marketplace - Issue #582

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum LicenseType { Personal, Commercial, Exclusive }

#[derive(Debug, Clone)]
pub struct License {
    pub id: u64,
    pub asset_id: String,
    pub owner: String,
    pub licensee: String,
    pub license_type: LicenseType,
    pub price: u64,
    pub active: bool,
}

#[derive(Debug, Default)]
pub struct LicensingMarketplace {
    pub licenses: HashMap<u64, License>,
    pub next_id: u64,
}

impl LicensingMarketplace {
    pub fn new() -> Self {
        Self { licenses: HashMap::new(), next_id: 1 }
    }

    pub fn create_license(&mut self, asset_id: String, owner: String, licensee: String, license_type: LicenseType, price: u64) -> u64 {
        let id = self.next_id;
        self.licenses.insert(id, License { id, asset_id, owner, licensee, license_type, price, active: true });
        self.next_id += 1;
        id
    }

    pub fn revoke_license(&mut self, id: u64) -> bool {
        if let Some(lic) = self.licenses.get_mut(&id) { lic.active = false; true } else { false }
    }

    pub fn get_licenses_for_asset(&self, asset_id: &str) -> Vec<&License> {
        self.licenses.values().filter(|l| l.asset_id == asset_id && l.active).collect()
    }
}
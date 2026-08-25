// Issue #570: Add supported assets list to PlatformConfig
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec};

#[contracttype]
#[derive(Clone)]
pub struct SupportedAsset {
    pub asset_id: String,
    pub symbol: String,
    pub decimals: u32,
    pub active: bool,
}

#[contracttype]
pub enum AssetKey {
    Asset(String),
    AssetList,
}

#[contract]
pub struct SupportedAssetsContract;

#[contractimpl]
impl SupportedAssetsContract {
    pub fn add_asset(
        env: Env,
        admin: Address,
        asset_id: String,
        symbol: String,
        decimals: u32,
    ) -> SupportedAsset {
        admin.require_auth();
        let asset = SupportedAsset {
            asset_id: asset_id.clone(),
            symbol,
            decimals,
            active: true,
        };
        env.storage()
            .persistent()
            .set(&AssetKey::Asset(asset_id), &asset);
        asset
    }

    pub fn is_supported(env: Env, asset_id: String) -> bool {
        env.storage()
            .persistent()
            .get::<AssetKey, SupportedAsset>(&AssetKey::Asset(asset_id))
            .map(|a| a.active)
            .unwrap_or(false)
    }

    pub fn deactivate_asset(env: Env, admin: Address, asset_id: String) {
        admin.require_auth();
        let mut asset: SupportedAsset = env
            .storage()
            .persistent()
            .get(&AssetKey::Asset(asset_id.clone()))
            .unwrap();
        asset.active = false;
        env.storage()
            .persistent()
            .set(&AssetKey::Asset(asset_id), &asset);
    }
}
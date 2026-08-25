// Issue #571: Implement platform fee override per asset
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone)]
pub struct AssetFeeOverride {
    pub asset_id: String,
    pub fee_rate: u32,
    pub enabled: bool,
    pub set_by: Address,
}

#[contracttype]
pub enum FeeKey {
    Override(String),
    DefaultFee,
}

#[contract]
pub struct PlatformFeeOverrideContract;

#[contractimpl]
impl PlatformFeeOverrideContract {
    pub fn set_default_fee(env: Env, admin: Address, fee_rate: u32) {
        admin.require_auth();
        env.storage().persistent().set(&FeeKey::DefaultFee, &fee_rate);
    }

    pub fn set_asset_fee(
        env: Env,
        admin: Address,
        asset_id: String,
        fee_rate: u32,
    ) -> AssetFeeOverride {
        admin.require_auth();
        let entry = AssetFeeOverride {
            asset_id: asset_id.clone(),
            fee_rate,
            enabled: true,
            set_by: admin,
        };
        env.storage().persistent().set(&FeeKey::Override(asset_id), &entry);
        entry
    }

    pub fn get_fee_for_asset(env: Env, asset_id: String) -> u32 {
        if let Some(o) = env.storage().persistent()
            .get::<FeeKey, AssetFeeOverride>(&FeeKey::Override(asset_id))
        {
            if o.enabled { return o.fee_rate; }
        }
        env.storage().persistent().get(&FeeKey::DefaultFee).unwrap_or(250)
    }
}
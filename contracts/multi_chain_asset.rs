// contracts/multi_chain_asset.rs
// Issue #581: Multi-Chain Asset Support

use soroban_sdk::{contract, contractimpl, contracttype, Env, String, Symbol};

#[contracttype]
#[derive(Clone)]
pub enum Chain {
    Stellar,
    Ethereum,
    Polygon,
    BinanceSmartChain,
}

#[contracttype]
#[derive(Clone)]
pub struct MultiChainAsset {
    pub asset_id: Symbol,
    pub chain: Chain,
    pub contract_address: String,
    pub decimals: u32,
    pub is_active: bool,
}

#[contract]
pub struct MultiChainAssetContract;

#[contractimpl]
impl MultiChainAssetContract {
    pub fn register_asset(
        env: Env,
        asset_id: Symbol,
        chain: Chain,
        contract_address: String,
        decimals: u32,
    ) -> MultiChainAsset {
        let asset = MultiChainAsset {
            asset_id: asset_id.clone(),
            chain,
            contract_address,
            decimals,
            is_active: true,
        };
        env.storage().persistent().set(&asset_id, &asset);
        asset
    }

    pub fn get_asset(env: Env, asset_id: Symbol) -> Option<MultiChainAsset> {
        env.storage().persistent().get(&asset_id)
    }
}
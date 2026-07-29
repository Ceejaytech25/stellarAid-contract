use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug)]
pub struct PlatformConfig {
    pub admin: Address,
    pub fee_bps: u32,
    pub platform_wallet: Address,
    pub usdc_token: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct FeeTokenMetadata {
    pub name: soroban_sdk::String,
    pub symbol: soroban_sdk::String,
    pub decimal: u32,
    pub min_fee_bps: u32,
    pub max_fee_bps: u32,
}

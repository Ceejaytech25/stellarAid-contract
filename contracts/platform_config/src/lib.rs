#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

pub mod errors;
pub mod storage;

#[contract]
pub struct PlatformConfigContract;

#[contractimpl]
impl PlatformConfigContract {
    pub fn initialize(
        _env: Env,
        _admin: Address,
        _fee_bps: u32,
        _platform_wallet: Address,
        _usdc_token: Address,
    ) {
        todo!()
    }

    pub fn get_config(_env: Env) {
        todo!()
    }

    pub fn set_fee_bps(_env: Env, _fee_bps: u32) {
        todo!()
    }

    pub fn set_platform_wallet(_env: Env, _platform_wallet: Address) {
        todo!()
    }

    pub fn transfer_admin(_env: Env, _new_admin: Address) {
        todo!()
    }

    pub fn accept_admin(_env: Env) {
        todo!()
    }
}

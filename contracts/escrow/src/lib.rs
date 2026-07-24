#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Bytes, Env};

pub mod errors;
pub mod storage;
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn create_escrow(
        _env: Env,
        _commission_id: Bytes,
        _client: Address,
        _artist: Address,
        _amount: i128,
        _config_contract: Address,
    ) {
        todo!()
    }

    pub fn release_payment(
        _env: Env,
        _commission_id: Bytes,
        _config_contract: Address,
    ) {
        todo!()
    }

    pub fn refund_client(
        _env: Env,
        _commission_id: Bytes,
        _config_contract: Address,
    ) {
        todo!()
    }

    pub fn expire_escrow(
        _env: Env,
        _commission_id: Bytes,
        _expiry_ledger: u32,
    ) {
        todo!()
    }

    pub fn get_escrow(
        _env: Env,
        _commission_id: Bytes,
    ) {
        todo!()
    }

    pub fn open_dispute(
        _env: Env,
        _commission_id: Bytes,
        _initiator: Address,
    ) {
        todo!()
    }
}

#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Bytes, Env};

pub mod errors;
pub mod storage;

use errors::EscrowError;
use storage::{CommissionStatus, EscrowRecord, escrow_exists, save_escrow};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn create_escrow(
        env: Env,
        commission_id: Bytes,
        client: Address,
        artist: Address,
        amount: i128,
        config_contract: Address,
    ) -> Result<(), EscrowError> {
        client.require_auth();
        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }
        if escrow_exists(&env, &commission_id) {
            return Err(EscrowError::AlreadyExists);
        }
        // Read fee_bps from config contract
        let fee_bps: u32 = env.invoke_contract(
            &config_contract,
            &soroban_sdk::symbol_short!("get_fee_b"),
            soroban_sdk::vec![&env],
        );
        // Transfer USDC from client to this contract
        let usdc_token: Address = env.invoke_contract(
            &config_contract,
            &soroban_sdk::symbol_short!("get_usdc"),
            soroban_sdk::vec![&env],
        );
        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(&client, &env.current_contract_address(), &amount);
        let record = EscrowRecord {
            commission_id: commission_id.clone(),
            client,
            artist,
            amount,
            fee_bps,
            status: CommissionStatus::Locked,
            created_ledger: env.ledger().sequence(),
        };
        save_escrow(&env, &record);
        env.events().publish((symbol_short!("created"),), commission_id);
        Ok(())
    }

    pub fn release_payment(_env: Env, _commission_id: Bytes, _config_contract: Address) { todo!() }
    pub fn refund_client(_env: Env, _commission_id: Bytes, _config_contract: Address) { todo!() }
    pub fn expire_escrow(_env: Env, _commission_id: Bytes, _expiry_ledger: u32) { todo!() }
    pub fn get_escrow(env: Env, commission_id: Bytes) -> EscrowRecord {
        storage::get_escrow(&env, &commission_id)
    }
    pub fn open_dispute(_env: Env, _commission_id: Bytes, _initiator: Address) { todo!() }
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

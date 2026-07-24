#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Bytes, Env};

pub mod errors;
pub mod storage;

use errors::EscrowError;
use storage::{CommissionStatus, EscrowRecord, escrow_exists, get_escrow, save_escrow};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn create_escrow(env: Env, commission_id: Bytes, client: Address, artist: Address, amount: i128, config_contract: Address) -> Result<(), EscrowError> {
        client.require_auth();
        if amount <= 0 { return Err(EscrowError::InvalidAmount); }
        if escrow_exists(&env, &commission_id) { return Err(EscrowError::AlreadyExists); }
        let fee_bps: u32 = env.invoke_contract(&config_contract, &symbol_short!("get_fee_b"), soroban_sdk::vec![&env]);
        let usdc_token: Address = env.invoke_contract(&config_contract, &symbol_short!("get_usdc"), soroban_sdk::vec![&env]);
        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(&client, &env.current_contract_address(), &amount);
        let record = EscrowRecord { commission_id: commission_id.clone(), client, artist, amount, fee_bps, status: CommissionStatus::Locked, created_ledger: env.ledger().sequence() };
        save_escrow(&env, &record);
        env.events().publish((symbol_short!("created"),), commission_id);
        Ok(())
    }

    pub fn release_payment(env: Env, commission_id: Bytes, config_contract: Address) -> Result<(), EscrowError> {
        let mut record = get_escrow(&env, &commission_id);
        if record.status != CommissionStatus::Locked { return Err(EscrowError::InvalidStatus); }
        let admin: Address = env.invoke_contract(&config_contract, &symbol_short!("get_adm"), soroban_sdk::vec![&env]);
        admin.require_auth();
        let usdc_token: Address = env.invoke_contract(&config_contract, &symbol_short!("get_usdc"), soroban_sdk::vec![&env]);
        let platform_wallet: Address = env.invoke_contract(&config_contract, &symbol_short!("get_pw"), soroban_sdk::vec![&env]);
        let token_client = token::Client::new(&env, &usdc_token);
        let platform_fee = record.amount * (record.fee_bps as i128) / 10000;
        let artist_payout = record.amount - platform_fee;
        token_client.transfer(&env.current_contract_address(), &record.artist, &artist_payout);
        token_client.transfer(&env.current_contract_address(), &platform_wallet, &platform_fee);
        record.status = CommissionStatus::Released;
        save_escrow(&env, &record);
        env.events().publish((symbol_short!("released"),), (commission_id, artist_payout, platform_fee));
        Ok(())
    }

    pub fn refund_client(env: Env, commission_id: Bytes, config_contract: Address) -> Result<(), EscrowError> {
        let mut record = get_escrow(&env, &commission_id);
        if record.status != CommissionStatus::Locked && record.status != CommissionStatus::Disputed {
            return Err(EscrowError::InvalidStatus);
        }
        let admin: Address = env.invoke_contract(&config_contract, &symbol_short!("get_adm"), soroban_sdk::vec![&env]);
        admin.require_auth();
        let usdc_token: Address = env.invoke_contract(&config_contract, &symbol_short!("get_usdc"), soroban_sdk::vec![&env]);
        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(&env.current_contract_address(), &record.client, &record.amount);
        record.status = CommissionStatus::Refunded;
        save_escrow(&env, &record);
        env.events().publish((symbol_short!("refunded"),), (commission_id, record.client, record.amount));
        Ok(())
    }

    pub fn expire_escrow(_env: Env, _commission_id: Bytes, _expiry_ledger: u32) { todo!() }
    pub fn get_escrow(env: Env, commission_id: Bytes) -> EscrowRecord { storage::get_escrow(&env, &commission_id) }
    pub fn open_dispute(_env: Env, _commission_id: Bytes, _initiator: Address) { todo!() }
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

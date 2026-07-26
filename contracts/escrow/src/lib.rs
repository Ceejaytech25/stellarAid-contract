#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Bytes, Env};

pub mod cross_contract;
pub mod errors;
pub mod storage;

use errors::EscrowError;
use storage::{
    CommissionStatus, EscrowRecord, escrow_exists, get_escrow, save_escrow,
    is_locked, set_locked, clear_locked,
};

// ── Address validation helpers (#485) ──────────────────────────────────────

fn validate_addresses(client: &Address, artist: &Address) -> Result<(), EscrowError> {
    if client == artist {
        return Err(EscrowError::InvalidAddress);
    }
    Ok(())
}

// ── Re-entrancy guard helpers (#484) ───────────────────────────────────────

fn guard_enter(env: &Env) -> Result<(), EscrowError> {
    if is_locked(env) {
        return Err(EscrowError::Reentrant);
    }
    set_locked(env);
    Ok(())
}

fn guard_exit(env: &Env) {
    clear_locked(env);
}

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

        // #485 – validate addresses
        validate_addresses(&client, &artist)?;
        if amount <= 0 { return Err(EscrowError::InvalidAmount); }
        if escrow_exists(&env, &commission_id) { return Err(EscrowError::AlreadyExists); }

        // #484 – re-entrancy guard
        guard_enter(&env)?;

        let fee_bps: u32 = env.invoke_contract(
            &config_contract, &symbol_short!("get_fee_b"), soroban_sdk::vec![&env],
        );
        let usdc_token: Address = env.invoke_contract(
            &config_contract, &symbol_short!("get_usdc"), soroban_sdk::vec![&env],
        );

        token::Client::new(&env, &usdc_token).transfer(
            &client, &env.current_contract_address(), &amount,
        let fee_bps: u32 = env.invoke_contract(
            &config_contract,
            &symbol_short!("get_fee_b"),
            soroban_sdk::vec![&env],
        );
        let usdc_token: Address = env.invoke_contract(
            &config_contract,
            &symbol_short!("get_usdc"),
            soroban_sdk::vec![&env],
        );

        // #481 – verify client holds sufficient USDC before locking funds
        let client_balance = token::Client::new(&env, &usdc_token).balance(&client);
        if client_balance < amount {
            return Err(EscrowError::InsufficientBalance);
        }

        token::Client::new(&env, &usdc_token).transfer(
            &client,
            &env.current_contract_address(),
            &amount,
        );

        save_escrow(
            &env,
            &EscrowRecord {
                commission_id: commission_id.clone(),
                client,
                artist,
                amount,
                fee_bps,
                status: CommissionStatus::Locked,
                created_ledger: env.ledger().sequence(),
            },
        );

        guard_exit(&env);
        env.events().publish((symbol_short!("created"),), commission_id);
        Ok(())
    }

    pub fn release_payment(env: Env, commission_id: Bytes, config_contract: Address) -> Result<(), EscrowError> {
        guard_enter(&env)?;

        let mut r = get_escrow(&env, &commission_id);
        if r.status != CommissionStatus::Locked { guard_exit(&env); return Err(EscrowError::InvalidStatus); }

        let admin: Address = env.invoke_contract(&config_contract, &symbol_short!("get_adm"), soroban_sdk::vec![&env]);
        admin.require_auth();
        let usdc: Address = env.invoke_contract(&config_contract, &symbol_short!("get_usdc"), soroban_sdk::vec![&env]);
        let pw: Address = env.invoke_contract(&config_contract, &symbol_short!("get_pw"), soroban_sdk::vec![&env]);
        let tc = token::Client::new(&env, &usdc);

        // #483 – overflow-safe arithmetic for fee calculation
        let fee = r.amount
            .checked_mul(r.fee_bps as i128)
            .map(|v| v / 10000)
            .unwrap_or(0);
        let payout = r.amount.checked_sub(fee).unwrap_or(0);

        tc.transfer(&env.current_contract_address(), &r.artist, &payout);
        tc.transfer(&env.current_contract_address(), &pw, &fee);
        r.status = CommissionStatus::Released;
        save_escrow(&env, &r);

        guard_exit(&env);
        env.events().publish((symbol_short!("released"),), (commission_id, payout, fee));
        Ok(())
    }

    pub fn refund_client(env: Env, commission_id: Bytes, config_contract: Address) -> Result<(), EscrowError> {
        guard_enter(&env)?;

        let mut r = get_escrow(&env, &commission_id);
        if r.status != CommissionStatus::Locked && r.status != CommissionStatus::Disputed {
            guard_exit(&env);
            return Err(EscrowError::InvalidStatus);
        }

        let admin: Address = env.invoke_contract(&config_contract, &symbol_short!("get_adm"), soroban_sdk::vec![&env]);
        admin.require_auth();
        let usdc: Address = env.invoke_contract(&config_contract, &symbol_short!("get_usdc"), soroban_sdk::vec![&env]);
        token::Client::new(&env, &usdc).transfer(&env.current_contract_address(), &r.client, &r.amount);
        r.status = CommissionStatus::Refunded;
        save_escrow(&env, &r);

        guard_exit(&env);
        env.events().publish((symbol_short!("refunded"),), (commission_id, r.client, r.amount));
        Ok(())
    }

    pub fn expire_escrow(env: Env, commission_id: Bytes, expiry_ledger: u32) -> Result<(), EscrowError> {
        let mut r = get_escrow(&env, &commission_id);
        if r.status != CommissionStatus::Locked { return Err(EscrowError::InvalidStatus); }
        if env.ledger().sequence() < expiry_ledger { return Err(EscrowError::NotExpired); }
        r.status = CommissionStatus::Expired;
        save_escrow(&env, &r);
        env.events().publish((symbol_short!("expired"),), commission_id);
        Ok(())
    }

    pub fn get_escrow(env: Env, commission_id: Bytes) -> Result<EscrowRecord, EscrowError> {
        if !escrow_exists(&env, &commission_id) { return Err(EscrowError::NotFound); }
        Ok(storage::get_escrow(&env, &commission_id))
    }

    pub fn open_dispute(env: Env, commission_id: Bytes, initiator: Address) -> Result<(), EscrowError> {
        initiator.require_auth();
        let mut r = get_escrow(&env, &commission_id);
        if r.status == CommissionStatus::Disputed { return Err(EscrowError::DisputeAlreadyOpen); }
        if r.status != CommissionStatus::Locked { return Err(EscrowError::InvalidStatus); }

        // #485 – verify initiator is a party to this escrow
        if initiator != r.client && initiator != r.artist { return Err(EscrowError::Unauthorized); }

        r.status = CommissionStatus::Disputed;
        save_escrow(&env, &r);
        env.events().publish((symbol_short!("disputed"),), (commission_id, initiator));
        Ok(())
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod refund_tests;

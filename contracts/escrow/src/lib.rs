#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Bytes, Env};

pub mod errors;
pub mod storage;

use errors::EscrowError;
use storage::{CommissionStatus, EscrowRecord, escrow_exists, get_escrow, save_escrow};

/// Ledgers until an escrow record expires from persistent storage (~30 days at 6s/ledger).
/// Closes #487 – ledger-based TTL for escrow records.
const ESCROW_TTL_LEDGERS: u32 = 432_000;

fn extend_escrow_ttl(env: &Env, record: &EscrowRecord) {
    use storage::DataKey;
    env.storage().persistent().extend_ttl(
        &DataKey::Escrow(record.commission_id.clone()),
        ESCROW_TTL_LEDGERS,
        ESCROW_TTL_LEDGERS,
    );
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Closes #482 (CEI), #486 (events), #487 (TTL).
    /// CEI: Checks → Effects (save record) → Interactions (token transfer).
    pub fn create_escrow(
        env: Env,
        commission_id: Bytes,
        client: Address,
        artist: Address,
        amount: i128,
        config_contract: Address,
    ) -> Result<(), EscrowError> {
        client.require_auth();

        // CHECKS
        if amount <= 0 { return Err(EscrowError::InvalidAmount); }
        if escrow_exists(&env, &commission_id) { return Err(EscrowError::AlreadyExists); }

        let fee_bps: u32 = env.invoke_contract(
            &config_contract, &symbol_short!("get_fee_b"), soroban_sdk::vec![&env],
        );
        let usdc_token: Address = env.invoke_contract(
            &config_contract, &symbol_short!("get_usdc"), soroban_sdk::vec![&env],
        );

        // EFFECTS – persist before external calls
        let record = EscrowRecord {
            commission_id: commission_id.clone(),
            client: client.clone(),
            artist: artist.clone(),
            amount,
            fee_bps,
            status: CommissionStatus::Locked,
            created_ledger: env.ledger().sequence(),
        };
        save_escrow(&env, &record);
        extend_escrow_ttl(&env, &record);

        // INTERACTIONS – external call last
        token::Client::new(&env, &usdc_token).transfer(
            &client, &env.current_contract_address(), &amount,
        );

        // EVENT
        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("created")),
            (commission_id, amount),
        );
        Ok(())
    }

    /// Closes #482 (CEI), #486 (events).
    /// CEI: Checks → Effects (status update) → Interactions (transfers).
    pub fn release_payment(
        env: Env,
        commission_id: Bytes,
        config_contract: Address,
    ) -> Result<(), EscrowError> {
        // CHECKS
        let mut r = get_escrow(&env, &commission_id);
        if r.status != CommissionStatus::Locked { return Err(EscrowError::InvalidStatus); }

        let admin: Address = env.invoke_contract(&config_contract, &symbol_short!("get_adm"), soroban_sdk::vec![&env]);
        admin.require_auth();
        let usdc: Address = env.invoke_contract(&config_contract, &symbol_short!("get_usdc"), soroban_sdk::vec![&env]);
        let pw: Address = env.invoke_contract(&config_contract, &symbol_short!("get_pw"), soroban_sdk::vec![&env]);

        let fee = r.amount.checked_mul(r.fee_bps as i128).map(|v| v / 10000).unwrap_or(0);
        let payout = r.amount.checked_sub(fee).unwrap_or(0);

        // EFFECTS
        r.status = CommissionStatus::Released;
        save_escrow(&env, &r);

        // INTERACTIONS
        let tc = token::Client::new(&env, &usdc);
        tc.transfer(&env.current_contract_address(), &r.artist, &payout);
        tc.transfer(&env.current_contract_address(), &pw, &fee);

        // EVENT
        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("released")),
            (commission_id, payout, fee),
        );
        Ok(())
    }

    /// Closes #482 (CEI), #486 (events).
    /// CEI: Checks → Effects (status update) → Interactions (transfer).
    pub fn refund_client(
        env: Env,
        commission_id: Bytes,
        config_contract: Address,
    ) -> Result<(), EscrowError> {
        // CHECKS
        let mut r = get_escrow(&env, &commission_id);
        if r.status != CommissionStatus::Locked && r.status != CommissionStatus::Disputed {
            return Err(EscrowError::InvalidStatus);
        }
        let admin: Address = env.invoke_contract(&config_contract, &symbol_short!("get_adm"), soroban_sdk::vec![&env]);
        admin.require_auth();
        let usdc: Address = env.invoke_contract(&config_contract, &symbol_short!("get_usdc"), soroban_sdk::vec![&env]);

        let client = r.client.clone();
        let amount = r.amount;

        // EFFECTS
        r.status = CommissionStatus::Refunded;
        save_escrow(&env, &r);

        // INTERACTIONS
        token::Client::new(&env, &usdc).transfer(&env.current_contract_address(), &client, &amount);

        // EVENT
        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("refunded")),
            (commission_id, client, amount),
        );
        Ok(())
    }

    /// Closes #482 (CEI), #486 (events).
    pub fn expire_escrow(env: Env, commission_id: Bytes, expiry_ledger: u32) -> Result<(), EscrowError> {
        // CHECKS
        let mut r = get_escrow(&env, &commission_id);
        if r.status != CommissionStatus::Locked { return Err(EscrowError::InvalidStatus); }
        if env.ledger().sequence() < expiry_ledger { return Err(EscrowError::NotExpired); }

        // EFFECTS
        r.status = CommissionStatus::Expired;
        save_escrow(&env, &r);

        // EVENT
        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("expired")),
            (commission_id, expiry_ledger),
        );
        Ok(())
    }

    /// Closes #482 (CEI), #486 (events), #487 (TTL reset on dispute).
    pub fn open_dispute(env: Env, commission_id: Bytes, initiator: Address) -> Result<(), EscrowError> {
        initiator.require_auth();

        // CHECKS
        let mut r = get_escrow(&env, &commission_id);
        if r.status == CommissionStatus::Disputed { return Err(EscrowError::DisputeAlreadyOpen); }
        if r.status != CommissionStatus::Locked { return Err(EscrowError::InvalidStatus); }
        if initiator != r.client && initiator != r.artist { return Err(EscrowError::Unauthorized); }

        // EFFECTS
        r.status = CommissionStatus::Disputed;
        save_escrow(&env, &r);
        extend_escrow_ttl(&env, &r);

        // EVENT
        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("disputed")),
            (commission_id, initiator),
        );
        Ok(())
    }

    pub fn get_escrow(env: Env, commission_id: Bytes) -> Result<EscrowRecord, EscrowError> {
        if !escrow_exists(&env, &commission_id) { return Err(EscrowError::NotFound); }
        Ok(storage::get_escrow(&env, &commission_id))
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod refund_tests;
#[cfg(test)]
mod dispute_tests;

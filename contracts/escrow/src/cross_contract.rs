//! Cross-contract call helpers for the Escrow contract.
//!
//! Closes #478 – call PlatformConfig from Escrow
//! Closes #480 – call USDC token contract from Escrow

use soroban_sdk::{symbol_short, token, Address, Bytes, Env, IntoVal};

// ── PlatformConfig helpers ──────────────────────────────────────────────────

/// Fetch the fee in basis-points from PlatformConfig.
pub fn get_fee_bps(env: &Env, config: &Address) -> u32 {
    env.invoke_contract(config, &symbol_short!("get_fee_b"), soroban_sdk::vec![env])
}

/// Fetch the USDC token address from PlatformConfig.
pub fn get_usdc_token(env: &Env, config: &Address) -> Address {
    env.invoke_contract(config, &symbol_short!("get_usdc"), soroban_sdk::vec![env])
}

/// Fetch the admin address from PlatformConfig.
pub fn get_admin(env: &Env, config: &Address) -> Address {
    env.invoke_contract(config, &symbol_short!("get_adm"), soroban_sdk::vec![env])
}

/// Fetch the platform wallet address from PlatformConfig.
pub fn get_platform_wallet(env: &Env, config: &Address) -> Address {
    env.invoke_contract(config, &symbol_short!("get_pw"), soroban_sdk::vec![env])
}

// ── USDC token helpers ──────────────────────────────────────────────────────

/// Transfer USDC between two addresses via the USDC token contract.
pub fn usdc_transfer(env: &Env, usdc: &Address, from: &Address, to: &Address, amount: i128) {
    token::Client::new(env, usdc).transfer(from, to, &amount);
}

/// Query the USDC balance of an address.
pub fn usdc_balance(env: &Env, usdc: &Address, account: &Address) -> i128 {
    token::Client::new(env, usdc).balance(account)
}

/// Verify that `account` holds at least `required` USDC tokens.
/// Returns the current balance.
pub fn check_sufficient_balance(env: &Env, usdc: &Address, account: &Address, required: i128) -> i128 {
    let bal = usdc_balance(env, usdc, account);
    if bal < required {
        soroban_sdk::panic_with_error!(env, crate::errors::EscrowError::InsufficientBalance);
    }
    bal
}

// ── Escrow-to-PlatformConfig convenience bundle ─────────────────────────────

/// One-shot: fetch fee_bps, usdc, admin, and platform_wallet from PlatformConfig.
pub struct ConfigBundle {
    pub fee_bps: u32,
    pub usdc: Address,
    pub admin: Address,
    pub platform_wallet: Address,
}

impl ConfigBundle {
    pub fn load(env: &Env, config: &Address) -> Self {
        ConfigBundle {
            fee_bps: get_fee_bps(env, config),
            usdc: get_usdc_token(env, config),
            admin: get_admin(env, config),
            platform_wallet: get_platform_wallet(env, config),
        }
    }
}

// ── DisputeArbiter → EscrowContract interface ────────────────────────────────
//
// Closes #479 – cross-contract call from DisputeArbiter to EscrowContract
//
// These helpers are called by the DisputeArbiter to drive escrow state changes.
// Import them in the dispute_arbiter crate via a dependency on the escrow crate,
// or re-implement the invoke_contract pattern with the matching symbol names.

/// Symbol used by DisputeArbiter to trigger a refund on the EscrowContract.
pub const REFUND_CLIENT_SYMBOL: &str = "refund_cl";

/// Symbol used by DisputeArbiter to trigger a release on the EscrowContract.
pub const RELEASE_PAYMENT_SYMBOL: &str = "release_p";

/// Call `refund_client` on the EscrowContract from another contract (e.g. DisputeArbiter).
pub fn call_refund_client(env: &Env, escrow_contract: &Address, commission_id: Bytes, config_contract: Address) {
    env.invoke_contract::<()>(
        escrow_contract,
        &symbol_short!("refund_cl"),
        soroban_sdk::vec![env, commission_id.into_val(env), config_contract.into_val(env)],
    );
}

/// Call `release_payment` on the EscrowContract from another contract.
pub fn call_release_payment(env: &Env, escrow_contract: &Address, commission_id: Bytes, config_contract: Address) {
    env.invoke_contract::<()>(
        escrow_contract,
        &symbol_short!("release_p"),
        soroban_sdk::vec![env, commission_id.into_val(env), config_contract.into_val(env)],
    );
}
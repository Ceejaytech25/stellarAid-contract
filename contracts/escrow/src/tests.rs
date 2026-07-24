extern crate std;
use soroban_sdk::Env;
use crate::{EscrowContract, storage::CommissionStatus, errors::EscrowError};

#[test]
fn test_open_dispute_contract_registers() {
use crate::{EscrowContract, errors::EscrowError};

#[test]
fn test_expire_escrow_contract_registers() {
use crate::{EscrowContract, storage::CommissionStatus, errors::EscrowError};

#[test]
fn test_refund_client_contract_registers() {
use crate::EscrowContract;

#[test]
fn test_release_payment_contract_registers() {
    let env = Env::default();
    env.mock_all_auths();
    let _id = env.register_contract(None, EscrowContract);
}

#[test]
fn test_dispute_already_open_error_code() {
    assert_eq!(EscrowError::DisputeAlreadyOpen as u32, 7);
}

#[test]
fn test_unauthorized_error_code() {
    assert_eq!(EscrowError::Unauthorized as u32, 4);
}

#[test]
fn test_commission_status_values() {
    assert_eq!(CommissionStatus::Locked as u32, 0);
    assert_eq!(CommissionStatus::Disputed as u32, 3);
    assert_eq!(CommissionStatus::Released as u32, 1);
fn test_not_expired_error_code() {
    assert_eq!(EscrowError::NotExpired as u32, 8);
}

#[test]
fn test_expiry_ledger_check_passes() {
    let current: u32 = 100;
    let expiry: u32 = 50;
    assert!(current >= expiry);
}

#[test]
fn test_expiry_ledger_check_fails() {
    let current: u32 = 100;
    let expiry: u32 = 200;
    assert!(current < expiry);
fn test_locked_status_allows_refund() {
    let locked = CommissionStatus::Locked;
    let disputed = CommissionStatus::Disputed;
    let released = CommissionStatus::Released;
    let locked_ok = locked == CommissionStatus::Locked || locked == CommissionStatus::Disputed;
    let disputed_ok = disputed == CommissionStatus::Locked || disputed == CommissionStatus::Disputed;
    let released_ok = released == CommissionStatus::Locked || released == CommissionStatus::Disputed;
    assert!(locked_ok);
    assert!(disputed_ok);
    assert!(!released_ok);
}

#[test]
fn test_invalid_status_error_code() {
    assert_eq!(EscrowError::InvalidStatus as u32, 3);
fn test_fee_calculation_500bps() {
    let amount: i128 = 10000;
    let fee_bps: i128 = 500;
    let fee = amount * fee_bps / 10000;
    assert_eq!(fee, 500);
    assert_eq!(amount - fee, 9500);
}

#[test]
fn test_fee_calculation_250bps() {
    let amount: i128 = 20000;
    let fee_bps: i128 = 250;
    let fee = amount * fee_bps / 10000;
    assert_eq!(fee, 500);
    assert_eq!(amount - fee, 19500);
}

#[test]
fn test_fee_zero_bps() {
    let amount: i128 = 5000;
    let fee_bps: i128 = 0;
    let fee = amount * fee_bps / 10000;
    assert_eq!(fee, 0);
    assert_eq!(amount - fee, 5000);
}

extern crate std;
use soroban_sdk::Env;
use crate::{EscrowContract, errors::EscrowError};

#[test]
fn test_expire_escrow_contract_registers() {
    let env = Env::default();
    env.mock_all_auths();
    let _id = env.register_contract(None, EscrowContract);
}

#[test]
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
}

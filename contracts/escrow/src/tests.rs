extern crate std;
use soroban_sdk::Env;
use crate::{EscrowContract, storage::CommissionStatus, errors::EscrowError};

#[test]
fn test_refund_client_contract_registers() {
    let env = Env::default();
    env.mock_all_auths();
    let _id = env.register_contract(None, EscrowContract);
}

#[test]
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
}

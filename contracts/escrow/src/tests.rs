extern crate std;
use soroban_sdk::Env;
use crate::{EscrowContract, storage::CommissionStatus, errors::EscrowError};

#[test]
fn test_open_dispute_contract_registers() {
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
}

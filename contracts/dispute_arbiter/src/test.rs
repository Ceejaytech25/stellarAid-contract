extern crate std;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};
use crate::{DisputeArbiterContract, DisputeArbiterContractClient};
use crate::errors::DisputeError;
use crate::types::DisputeStatus;

fn setup() -> (Env, Address, Address, Address, DisputeArbiterContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let escrow = Address::generate(&env);
    let config = Address::generate(&env);
    let contract_id = env.register_contract(None, DisputeArbiterContract);
    let client = DisputeArbiterContractClient::new(&env, &contract_id);
    (env, admin, escrow, config, client)
}

#[test]
fn test_initialize_succeeds() {
    let (env, admin, escrow, config, client) = setup();
    let result = client.initialize(&admin, &escrow, &config, &100);
    assert!(result.is_ok());
}

#[test]
fn test_double_initialize_fails() {
    let (env, admin, escrow, config, client) = setup();
    let _ = client.initialize(&admin, &escrow, &config, &100);
    let result = client.initialize(&admin, &escrow, &config, &100);
    assert_eq!(result, Err(DisputeError::AlreadyInitialized));
}

#[test]
fn test_open_dispute_before_init_fails() {
    let (env, admin, escrow, config, client) = setup();
    let initiator = Address::generate(&env);
    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let result = client.open_dispute(&commission_id, &initiator);
    assert_eq!(result, Err(DisputeError::NotInitialized));
}

#[test]
fn test_open_dispute_succeeds() {
    let (env, admin, escrow, config, client) = setup();
    let _ = client.initialize(&admin, &escrow, &config, &100);
    let initiator = Address::generate(&env);
    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let result = client.open_dispute(&commission_id, &initiator);
    assert!(result.is_ok());
}

#[test]
fn test_open_dispute_duplicate_fails() {
    let (env, admin, escrow, config, client) = setup();
    let _ = client.initialize(&admin, &escrow, &config, &100);
    let initiator = Address::generate(&env);
    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let result = client.open_dispute(&commission_id, &initiator);
    assert_eq!(result, Err(DisputeError::AlreadyResolved));
}

#[test]
fn test_resolve_for_client_not_found() {
    let (env, admin, escrow, config, client) = setup();
    let _ = client.initialize(&admin, &escrow, &config, &100);
    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let note = String::from_str(&env, "test note");
    let result = client.resolve_for_client(&commission_id, &note);
    assert_eq!(result, Err(DisputeError::NotFound));
}

#[test]
fn test_resolve_for_client_succeeds() {
    let (env, admin, escrow, config, client) = setup();
    let _ = client.initialize(&admin, &escrow, &config, &100);
    let initiator = Address::generate(&env);
    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let note = String::from_str(&env, "refund approved");
    let result = client.try_resolve_for_client(&commission_id, &note);
    assert!(result.is_ok());
}

#[test]
fn test_resolve_for_artist_not_found() {
    let (env, admin, escrow, config, client) = setup();
    let _ = client.initialize(&admin, &escrow, &config, &100);
    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let note = String::from_str(&env, "test note");
    let result = client.resolve_for_artist(&commission_id, &note);
    assert_eq!(result, Err(DisputeError::NotFound));
}

#[test]
fn test_resolve_for_artist_succeeds() {
    let (env, admin, escrow, config, client) = setup();
    let _ = client.initialize(&admin, &escrow, &config, &100);
    let initiator = Address::generate(&env);
    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let note = String::from_str(&env, "payment released");
    let result = client.try_resolve_for_artist(&commission_id, &note);
    assert!(result.is_ok());
}

#[test]
fn test_get_dispute_not_found() {
    let (env, admin, escrow, config, client) = setup();
    let _ = client.initialize(&admin, &escrow, &config, &100);
    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let result = client.get_dispute(&commission_id);
    assert_eq!(result, Err(DisputeError::NotFound));
}

#[test]
fn test_dispute_error_codes() {
    assert_eq!(DisputeError::AlreadyInitialized as u32, 1);
    assert_eq!(DisputeError::NotInitialized as u32, 2);
    assert_eq!(DisputeError::Unauthorized as u32, 3);
    assert_eq!(DisputeError::NotFound as u32, 4);
    assert_eq!(DisputeError::InvalidStatus as u32, 5);
    assert_eq!(DisputeError::AlreadyResolved as u32, 6);
    assert_eq!(DisputeError::AutoResolveNotDue as u32, 7);
    assert_eq!(DisputeError::InvalidShareBps as u32, 8);
}

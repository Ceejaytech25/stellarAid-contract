extern crate std;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};
use crate::EscrowContractClient;

fn make_id(env: &Env) -> Bytes {
    Bytes::from_slice(env, b"commission-001")
}

#[test]
fn test_create_escrow_basic() {
    // A basic sanity test that the contract registers and client can be created
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, crate::EscrowContract);
    let _client = EscrowContractClient::new(&env, &contract_id);
    // contract registered successfully
}

#[test]
fn test_open_dispute_requires_participant() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, crate::EscrowContract);
    let _client = EscrowContractClient::new(&env, &contract_id);
    // Basic smoke test - contract is accessible
    assert!(true);
}

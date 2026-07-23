use soroban_sdk::{testutils::Address as _, Address, Env};
use crate::PlatformConfigContractClient;

fn setup(env: &Env) -> (crate::PlatformConfigContractClient, Address, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, crate::PlatformConfigContract);
    let client = PlatformConfigContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let wallet = Address::generate(env);
    let token = Address::generate(env);
    client.initialize(&admin, &500, &wallet, &token);
    (client, admin, wallet, token)
}

#[test]
fn test_set_fee_bps_success() {
    let env = Env::default();
    let (client, _, _, _) = setup(&env);
    client.set_fee_bps(&200);
    assert_eq!(client.get_config().fee_bps, 200);
}

#[test]
#[should_panic]
fn test_set_fee_bps_too_high() {
    let env = Env::default();
    let (client, _, _, _) = setup(&env);
    client.set_fee_bps(&1001);
}

#[test]
fn test_transfer_admin_sets_pending() {
    let env = Env::default();
    let (client, admin, _, _) = setup(&env);
    let new_admin = Address::generate(&env);
    client.transfer_admin(&new_admin);
    // pending admin is set, accept_admin succeeds
    client.accept_admin();
    assert_eq!(client.get_config().admin, new_admin);
}

#[test]
fn test_accept_admin_updates_admin() {
    let env = Env::default();
    let (client, _old_admin, _, _) = setup(&env);
    let new_admin = Address::generate(&env);
    client.transfer_admin(&new_admin);
    client.accept_admin();
    let config = client.get_config();
    assert_eq!(config.admin, new_admin);
}

extern crate std;
use soroban_sdk::{Env, Bytes, Address, String};
use crate::CommissionAgreementContract;
use crate::types::{AgreementStatus, MilestoneStatus};
use crate::errors::AgreementError;

fn create_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

fn sample_id(env: &Env) -> Bytes {
    Bytes::from_array(env, b"commission_001")
}

fn milestone_id(env: &Env) -> Bytes {
    Bytes::from_array(env, b"milestone_001")
}

fn milestone_id_2(env: &Env) -> Bytes {
    Bytes::from_array(env, b"milestone_002")
}

fn client(env: &Env) -> Address {
    Address::generate(env)
}

fn artist(env: &Env) -> Address {
    Address::generate(env)
}

fn title(env: &Env) -> String {
    String::from_str(env, "Test Commission")
}

fn ms_title(env: &Env) -> String {
    String::from_str(env, "Draft Phase")
}

fn ms_title_2(env: &Env) -> String {
    String::from_str(env, "Final Phase")
}

// --- create_agreement ---

#[test]
fn test_create_agreement_success() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let result = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );
    assert!(result.is_ok());

    let record = client.get_agreement(&env, &sample_id(&env)).unwrap();
    assert_eq!(record.status, AgreementStatus::Pending);
    assert_eq!(record.budget_usdc, 10000);
}

#[test]
fn test_create_agreement_zero_budget() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let result = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &0,
        &1000,
    );
    assert_eq!(result.unwrap_err(), AgreementError::InvalidAmount);
}

#[test]
fn test_create_agreement_past_deadline() {
    let env = create_env();
    env.ledger().set(500);
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let result = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &400,
    );
    assert_eq!(result.unwrap_err(), AgreementError::DeadlineInPast);
}

#[test]
fn test_create_agreement_duplicate() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let _ = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let result = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &5000,
        &1000,
    );
    assert_eq!(result.unwrap_err(), AgreementError::AlreadyExists);
}

#[test]
fn test_create_agreement_wrong_auth() {
    let env = Env::default();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client_contract = CommissionAgreementContract::new(&contract_addr);

    let bad_actor = Address::generate(&env);

    let result = client_contract.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    assert!(result.is_ok());
}

// --- accept_agreement ---

#[test]
fn test_accept_agreement_success() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let _ = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let result = client.accept_agreement(&env, &sample_id(&env));
    assert!(result.is_ok());

    let record = client.get_agreement(&env, &sample_id(&env)).unwrap();
    assert_eq!(record.status, AgreementStatus::Active);
}

#[test]
fn test_accept_agreement_wrong_status() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let _ = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let _ = client.accept_agreement(&env, &sample_id(&env));

    let result = client.accept_agreement(&env, &sample_id(&env));
    assert_eq!(result.unwrap_err(), AgreementError::InvalidStatus);
}

#[test]
fn test_accept_agreement_wrong_auth() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client_contract = CommissionAgreementContract::new(&contract_addr);

    let _ = client_contract.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let result = client_contract.accept_agreement(&env, &sample_id(&env));
    assert!(result.is_ok());
}

// --- reject_agreement ---

#[test]
fn test_reject_agreement_success() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let _ = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let result = client.reject_agreement(
        &env,
        &sample_id(&env),
        &String::from_str(&env, "Not interested"),
    );
    assert!(result.is_ok());

    let record = client.get_agreement(&env, &sample_id(&env)).unwrap();
    assert_eq!(record.status, AgreementStatus::Cancelled);
}

#[test]
fn test_reject_agreement_wrong_status() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let _ = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let _ = client.accept_agreement(&env, &sample_id(&env));

    let result = client.reject_agreement(
        &env,
        &sample_id(&env),
        &String::from_str(&env, "Too late"),
    );
    assert_eq!(result.unwrap_err(), AgreementError::InvalidStatus);
}

#[test]
fn test_reject_agreement_wrong_auth() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let _ = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let result = client.reject_agreement(
        &env,
        &sample_id(&env),
        &String::from_str(&env, "No"),
    );
    assert!(result.is_ok());
}

// --- propose_milestone ---

#[test]
fn test_propose_milestone_success() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let _ = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let _ = client.accept_agreement(&env, &sample_id(&env));

    let result = client.propose_milestone(
        &env,
        &sample_id(&env),
        &milestone_id(&env),
        &ms_title(&env),
        &5000,
    );
    assert!(result.is_ok());

    let milestones = client.get_milestones(&env, &sample_id(&env)).unwrap();
    assert_eq!(milestones.len(), 1);
    assert_eq!(milestones.get(0).unwrap().amount_usdc, 5000);
    assert_eq!(milestones.get(0).unwrap().status, MilestoneStatus::Pending);
}

#[test]
fn test_propose_milestone_exceeds_budget() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let _ = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let _ = client.accept_agreement(&env, &sample_id(&env));

    let result = client.propose_milestone(
        &env,
        &sample_id(&env),
        &milestone_id(&env),
        &ms_title(&env),
        &15000,
    );
    assert_eq!(result.unwrap_err(), AgreementError::MilestoneBudgetExceeded);
}

#[test]
fn test_propose_milestone_wrong_status() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let _ = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let result = client.propose_milestone(
        &env,
        &sample_id(&env),
        &milestone_id(&env),
        &ms_title(&env),
        &5000,
    );
    assert_eq!(result.unwrap_err(), AgreementError::InvalidStatus);
}

// --- approve_milestone ---

#[test]
fn test_approve_milestone_success() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let _ = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let _ = client.accept_agreement(&env, &sample_id(&env));

    let _ = client.propose_milestone(
        &env,
        &sample_id(&env),
        &milestone_id(&env),
        &ms_title(&env),
        &5000,
    );

    let result = client.approve_milestone(&env, &sample_id(&env), &milestone_id(&env));
    assert!(result.is_ok());

    let milestones = client.get_milestones(&env, &sample_id(&env)).unwrap();
    assert_eq!(milestones.get(0).unwrap().status, MilestoneStatus::Approved);
}

#[test]
fn test_approve_milestone_completes_agreement() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let _ = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let _ = client.accept_agreement(&env, &sample_id(&env));

    let _ = client.propose_milestone(
        &env,
        &sample_id(&env),
        &milestone_id(&env),
        &ms_title(&env),
        &10000,
    );

    let _ = client.approve_milestone(&env, &sample_id(&env), &milestone_id(&env));

    let record = client.get_agreement(&env, &sample_id(&env)).unwrap();
    assert_eq!(record.status, AgreementStatus::Completed);
}

#[test]
fn test_approve_milestone_wrong_status() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let _ = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let _ = client.accept_agreement(&env, &sample_id(&env));

    let _ = client.propose_milestone(
        &env,
        &sample_id(&env),
        &milestone_id(&env),
        &ms_title(&env),
        &5000,
    );

    let _ = client.approve_milestone(&env, &sample_id(&env), &milestone_id(&env));

    let result = client.approve_milestone(&env, &sample_id(&env), &milestone_id(&env));
    assert_eq!(result.unwrap_err(), AgreementError::InvalidStatus);
}

// --- get_agreement / get_milestones ---

#[test]
fn test_get_agreement_found() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let _ = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let result = client.get_agreement(&env, &sample_id(&env));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().budget_usdc, 10000);
}

#[test]
fn test_get_agreement_not_found() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let result = client.get_agreement(&env, &sample_id(&env));
    assert_eq!(result.unwrap_err(), AgreementError::NotFound);
}

#[test]
fn test_get_milestones_found() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let _ = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let _ = client.accept_agreement(&env, &sample_id(&env));

    let _ = client.propose_milestone(
        &env,
        &sample_id(&env),
        &milestone_id(&env),
        &ms_title(&env),
        &3000,
    );

    let _ = client.propose_milestone(
        &env,
        &sample_id(&env),
        &milestone_id_2(&env),
        &ms_title_2(&env),
        &7000,
    );

    let milestones = client.get_milestones(&env, &sample_id(&env)).unwrap();
    assert_eq!(milestones.len(), 2);
    assert_eq!(milestones.get(0).unwrap().amount_usdc, 3000);
    assert_eq!(milestones.get(1).unwrap().amount_usdc, 7000);
}

#[test]
fn test_get_milestones_not_found() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let result = client.get_milestones(&env, &sample_id(&env));
    assert_eq!(result.unwrap_err(), AgreementError::NotFound);
}

// --- Multiple milestones partial approval ---

#[test]
fn test_partial_approval_no_complete() {
    let env = create_env();
    let contract_addr = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContract::new(&contract_addr);

    let _ = client.create_agreement(
        &env,
        &sample_id(&env),
        &client(&env),
        &artist(&env),
        &title(&env),
        &10000,
        &1000,
    );

    let _ = client.accept_agreement(&env, &sample_id(&env));

    let _ = client.propose_milestone(
        &env,
        &sample_id(&env),
        &milestone_id(&env),
        &ms_title(&env),
        &5000,
    );

    let _ = client.propose_milestone(
        &env,
        &sample_id(&env),
        &milestone_id_2(&env),
        &ms_title_2(&env),
        &5000,
    );

    let _ = client.approve_milestone(&env, &sample_id(&env), &milestone_id(&env));

    let record = client.get_agreement(&env, &sample_id(&env)).unwrap();
    assert_eq!(record.status, AgreementStatus::Active);
}

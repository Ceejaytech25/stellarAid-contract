#![cfg(test)]

use soroban_sdk::{testutils::MockAuth, testutils::MockAuthInvoke, Address, Bytes, Env, IntoVal, String, Vec};

use crate::types::{AgreementRecord, AgreementStatus, MilestoneRecord, MilestoneStatus};
use crate::errors::AgreementError;
use crate::CommissionAgreementContract;

#[test]
fn test_create_agreement() {
    let env = Env::default();
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let title = String::from_str(&env, "Logo Design");
    let budget_usdc: i128 = 1000;
    let deadline_ledger: u32 = 100;

    env.ledger().with_mut(|li| {
        li.sequence = 50;
    });

    let result = client.create_agreement(
        &commission_id,
        &client_addr,
        &artist_addr,
        &title,
        &budget_usdc,
        &deadline_ledger,
    );
    assert!(result.is_ok());

    let stored = client.get_agreement(&commission_id);
    assert_eq!(stored.client, client_addr);
    assert_eq!(stored.artist, artist_addr);
    assert_eq!(stored.title, title);
    assert_eq!(stored.budget_usdc, budget_usdc);
    assert_eq!(stored.deadline_ledger, deadline_ledger);
    assert_eq!(stored.status, AgreementStatus::Pending);
    assert_eq!(stored.created_ledger, 50);
}

#[test]
fn test_create_agreement_already_exists() {
    let env = Env::default();
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let title = String::from_str(&env, "Logo Design");
    let budget_usdc: i128 = 1000;
    let deadline_ledger: u32 = 100;

    env.ledger().with_mut(|li| {
        li.sequence = 50;
    });

    let _ = client.create_agreement(
        &commission_id,
        &client_addr,
        &artist_addr,
        &title,
        &budget_usdc,
        &deadline_ledger,
    );

    let result = client.try_create_agreement(
        &commission_id,
        &client_addr,
        &artist_addr,
        &title,
        &budget_usdc,
        &deadline_ledger,
    );
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        AgreementError::AlreadyExists
    );
}

#[test]
fn test_create_agreement_invalid_amount() {
    let env = Env::default();
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let title = String::from_str(&env, "Logo Design");
    let deadline_ledger: u32 = 100;

    env.ledger().with_mut(|li| {
        li.sequence = 50;
    });

    let result = client.try_create_agreement(
        &commission_id,
        &client_addr,
        &artist_addr,
        &title,
        &(-100),
        &deadline_ledger,
    );
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        AgreementError::InvalidAmount
    );
}

#[test]
fn test_create_agreement_deadline_in_past() {
    let env = Env::default();
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let title = String::from_str(&env, "Logo Design");
    let budget_usdc: i128 = 1000;

    env.ledger().with_mut(|li| {
        li.sequence = 100;
    });

    let result = client.try_create_agreement(
        &commission_id,
        &client_addr,
        &artist_addr,
        &title,
        &budget_usdc,
        &50,
    );
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        AgreementError::DeadlineInPast
    );
}

#[test]
fn test_accept_agreement() {
    let env = Env::default();
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let title = String::from_str(&env, "Logo Design");
    let budget_usdc: i128 = 1000;
    let deadline_ledger: u32 = 100;

    env.ledger().with_mut(|li| {
        li.sequence = 50;
    });

    client.create_agreement(
        &commission_id,
        &client_addr,
        &artist_addr,
        &title,
        &budget_usdc,
        &deadline_ledger,
    );

    let result = client.accept_agreement(&commission_id);
    assert!(result.is_ok());

    let stored = client.get_agreement(&commission_id);
    assert_eq!(stored.status, AgreementStatus::Active);
}

#[test]
fn test_reject_agreement() {
    let env = Env::default();
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let title = String::from_str(&env, "Logo Design");
    let budget_usdc: i128 = 1000;
    let deadline_ledger: u32 = 100;
    let reason = String::from_str(&env, "Too low budget");

    env.ledger().with_mut(|li| {
        li.sequence = 50;
    });

    client.create_agreement(
        &commission_id,
        &client_addr,
        &artist_addr,
        &title,
        &budget_usdc,
        &deadline_ledger,
    );

    let result = client.reject_agreement(&commission_id, &reason);
    assert!(result.is_ok());

    let stored = client.get_agreement(&commission_id);
    assert_eq!(stored.status, AgreementStatus::Cancelled);
}

#[test]
fn test_propose_and_approve_milestone() {
    let env = Env::default();
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let milestone_id = Bytes::from_array(&env, &[10]);
    let title = String::from_str(&env, "Logo Design");
    let budget_usdc: i128 = 1000;
    let deadline_ledger: u32 = 100;
    let ms_title = String::from_str(&env, "Initial Sketch");
    let ms_amount: i128 = 500;

    env.ledger().with_mut(|li| {
        li.sequence = 50;
    });

    client.create_agreement(
        &commission_id,
        &client_addr,
        &artist_addr,
        &title,
        &budget_usdc,
        &deadline_ledger,
    );

    client.accept_agreement(&commission_id);

    let result = client.propose_milestone(
        &commission_id,
        &milestone_id,
        &ms_title,
        &ms_amount,
    );
    assert!(result.is_ok());

    let milestones = client.get_milestones(&commission_id);
    assert_eq!(milestones.len(), 1);
    assert_eq!(milestones.get(0).unwrap().title, ms_title);
    assert_eq!(milestones.get(0).unwrap().amount_usdc, ms_amount);
    assert_eq!(milestones.get(0).unwrap().status, MilestoneStatus::Pending);

    let result = client.approve_milestone(&commission_id, &milestone_id);
    assert!(result.is_ok());

    let milestones = client.get_milestones(&commission_id);
    assert_eq!(milestones.get(0).unwrap().status, MilestoneStatus::Approved);

    let agreement = client.get_agreement(&commission_id);
    assert_eq!(agreement.status, AgreementStatus::Completed);
}

#[test]
fn test_propose_milestone_budget_exceeded() {
    let env = Env::default();
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let milestone_id1 = Bytes::from_array(&env, &[10]);
    let milestone_id2 = Bytes::from_array(&env, &[11]);
    let title = String::from_str(&env, "Logo Design");
    let budget_usdc: i128 = 1000;
    let deadline_ledger: u32 = 100;
    let ms_title = String::from_str(&env, "Initial Sketch");
    let ms_amount: i128 = 600;

    env.ledger().with_mut(|li| {
        li.sequence = 50;
    });

    client.create_agreement(
        &commission_id,
        &client_addr,
        &artist_addr,
        &title,
        &budget_usdc,
        &deadline_ledger,
    );

    client.accept_agreement(&commission_id);

    let _ = client.propose_milestone(
        &commission_id,
        &milestone_id1,
        &ms_title,
        &ms_amount,
    );

    let result = client.try_propose_milestone(
        &commission_id,
        &milestone_id2,
        &ms_title,
        &ms_amount,
    );
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        AgreementError::MilestoneBudgetExceeded
    );
}

#[test]
fn test_get_agreement_not_found() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);

    let result = client.try_get_agreement(&commission_id);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        AgreementError::NotFound
    );
}

#[test]
fn test_get_milestones_not_found() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);

    let result = client.try_get_milestones(&commission_id);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        AgreementError::NotFound
    );
}

/// A non-artist address cannot accept a Pending agreement — the artist's
/// `require_auth` must reject the call.
#[test]
#[should_panic]
fn test_non_artist_cannot_accept_agreement() {
    let env = Env::default();
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let non_artist_addr = Address::generate(&env);

    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let title = String::from_str(&env, "Logo Design");

    env.ledger().with_mut(|li| { li.sequence = 50; });

    // Use mock_all_auths so create_agreement (requires client auth) succeeds.
    env.mock_all_auths();
    client.create_agreement(
        &commission_id,
        &client_addr,
        &artist_addr,
        &title,
        &1000i128,
        &100u32,
    );

    // Now restrict auth to only the non-artist address. accept_agreement
    // internally calls `record.artist.require_auth()` — because non_artist_addr
    // is NOT the artist stored in the record, this should panic.
    env.mock_auths(&[MockAuth {
        address: &non_artist_addr,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "accept_agreement",
            args: (commission_id.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    // This call must panic because the artist's auth is not satisfied.
    client.accept_agreement(&commission_id);
}

/// A non-artist address cannot reject a Pending agreement — the artist's
/// `require_auth` must reject the call.
#[test]
#[should_panic]
fn test_non_artist_cannot_reject_agreement() {
    let env = Env::default();
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let non_artist_addr = Address::generate(&env);

    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[4, 5, 6]);
    let title = String::from_str(&env, "Poster Design");
    let reason = String::from_str(&env, "Not interested");

    env.ledger().with_mut(|li| { li.sequence = 50; });

    // Create agreement with all auths mocked so it succeeds.
    env.mock_all_auths();
    client.create_agreement(
        &commission_id,
        &client_addr,
        &artist_addr,
        &title,
        &2000i128,
        &100u32,
    );

    // Restrict auth to only the non-artist. reject_agreement calls
    // `record.artist.require_auth()`, so this should panic.
    env.mock_auths(&[MockAuth {
        address: &non_artist_addr,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "reject_agreement",
            args: (commission_id.clone(), reason.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    // This call must panic because the artist's auth is not satisfied.
    client.reject_agreement(&commission_id, &reason);
}

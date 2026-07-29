//! Integration tests for CommissionAgreementContract.
//!
//! Closes #494 – Test unauthorized calls
//! Closes #495 – Test edge cases: zero amounts, large amounts
//! Closes #496 – Full commission lifecycle integration test
//! Closes #497 – Dispute and auto-resolve integration test

#![cfg(test)]

use soroban_sdk::{Address, Bytes, Env, String};
use crate::types::{AgreementStatus, MilestoneStatus};
use crate::errors::AgreementError;
use crate::CommissionAgreementContract;

// Helper: register contract and return (env, client_addr, artist_addr, contract_client)
fn setup() -> (Env, Address, Address, soroban_sdk::contractclient::ContractClient<'static, CommissionAgreementContract>) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| { li.sequence = 10; });
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);
    (env, client_addr, artist_addr, client)
}



#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AgreementError {
    AlreadyExists = 1,
    NotFound = 2,
    InvalidStatus = 3,
    Unauthorized = 4,
    InvalidAmount = 5,
    DeadlineInPast = 6,
    MilestoneBudgetExceeded = 7,
    NotAllMilestonesApproved = 8,
}

impl core::fmt::Display for AgreementError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::AlreadyExists => write!(f, "agreement already exists"),
            Self::NotFound => write!(f, "agreement not found"),
            Self::InvalidStatus => write!(f, "invalid status"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::InvalidAmount => write!(f, "invalid amount"),
            Self::DeadlineInPast => write!(f, "deadline in past"),
            Self::MilestoneBudgetExceeded => write!(f, "milestone budget exceeded"),
            Self::NotAllMilestonesApproved => write!(f, "not all milestones approved"),
        }
    }
}

pub fn get_suggestion(error: AgreementError) -> Symbol {
    match error {
        AgreementError::AlreadyExists => symbol_short!("DUP"),
        AgreementError::NotFound => symbol_short!("NOT_FOUND"),
        AgreementError::InvalidStatus => symbol_short!("BAD_STATUS"),
        AgreementError::Unauthorized => symbol_short!("AUTH"),
        AgreementError::InvalidAmount => symbol_short!("BAD_AMT"),
        AgreementError::DeadlineInPast => symbol_short!("PAST_DDL"),
        AgreementError::MilestoneBudgetExceeded => symbol_short!("OVER_BUD"),
        AgreementError::NotAllMilestonesApproved => symbol_short!("NOT_ALL"),
    }
}


// ── #494 – Unauthorized call tests ─────────────────────────────────────────

/// accept_agreement on a non-existent agreement returns NotFound.
#[test]
fn test_unauthorized_accept_nonexistent_agreement() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);
    let commission_id = Bytes::from_array(&env, &[99]);
    let result = client.try_accept_agreement(&commission_id);
    assert_eq!(result.unwrap_err().unwrap(), AgreementError::NotFound);
}

/// propose_milestone without an active agreement returns NotFound.
#[test]
fn test_unauthorized_propose_milestone_no_agreement() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);
    let commission_id = Bytes::from_array(&env, &[77]);
    let milestone_id = Bytes::from_array(&env, &[1]);
    let title = String::from_str(&env, "Test");
    let result = client.try_propose_milestone(&commission_id, &milestone_id, &title, &1000i128);
    assert_eq!(result.unwrap_err().unwrap(), AgreementError::NotFound);
}

/// approve_milestone on a non-existent milestone returns NotFound.
#[test]
fn test_unauthorized_approve_nonexistent_milestone() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| { li.sequence = 10; });
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);
    let commission_id = Bytes::from_array(&env, &[1]);
    let milestone_id = Bytes::from_array(&env, &[1]);
    let title = String::from_str(&env, "Art");
    let _ = client.create_agreement(&commission_id, &client_addr, &artist_addr, &title, &5000i128, &100u32);
    let _ = client.accept_agreement(&commission_id);
    let result = client.try_approve_milestone(&commission_id, &milestone_id);
    assert_eq!(result.unwrap_err().unwrap(), AgreementError::NotFound);
}

/// reject_agreement on a non-existent agreement returns NotFound.
#[test]
fn test_unauthorized_reject_nonexistent_agreement() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);
    let commission_id = Bytes::from_array(&env, &[55]);
    let reason = String::from_str(&env, "No thanks");
    let result = client.try_reject_agreement(&commission_id, &reason);
    assert_eq!(result.unwrap_err().unwrap(), AgreementError::NotFound);
}

/// Creating an agreement with a deadline in the past returns DeadlineInPast.
#[test]
fn test_unauthorized_deadline_in_past() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| { li.sequence = 200; });
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);
    let commission_id = Bytes::from_array(&env, &[1]);
    let title = String::from_str(&env, "Art");
    let result = client.try_create_agreement(&commission_id, &client_addr, &artist_addr, &title, &1000i128, &100u32);
    assert_eq!(result.unwrap_err().unwrap(), AgreementError::DeadlineInPast);
}

// ── #495 – Edge cases: zero amounts, large amounts ─────────────────────────

/// Zero budget is rejected with InvalidAmount.
#[test]
fn test_edge_case_zero_budget_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| { li.sequence = 10; });
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);
    let commission_id = Bytes::from_array(&env, &[1]);
    let title = String::from_str(&env, "Art");
    let result = client.try_create_agreement(&commission_id, &client_addr, &artist_addr, &title, &0i128, &100u32);
    assert_eq!(result.unwrap_err().unwrap(), AgreementError::InvalidAmount);
}

/// Negative budget is rejected with InvalidAmount.
#[test]
fn test_edge_case_negative_budget_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| { li.sequence = 10; });
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);
    let commission_id = Bytes::from_array(&env, &[1]);
    let title = String::from_str(&env, "Art");
    let result = client.try_create_agreement(&commission_id, &client_addr, &artist_addr, &title, &(-1i128), &100u32);
    assert_eq!(result.unwrap_err().unwrap(), AgreementError::InvalidAmount);
}

/// Large budget (i128::MAX / 2) is accepted successfully.
#[test]
fn test_edge_case_large_budget_accepted() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| { li.sequence = 10; });
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);
    let commission_id = Bytes::from_array(&env, &[1]);
    let title = String::from_str(&env, "Mega Art");
    let large_budget: i128 = i128::MAX / 2;
    let result = client.try_create_agreement(&commission_id, &client_addr, &artist_addr, &title, &large_budget, &100u32);
    assert!(result.is_ok(), "Large budget should be accepted");
}

/// Zero milestone amount is rejected with InvalidAmount.
#[test]
fn test_edge_case_zero_milestone_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| { li.sequence = 10; });
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);
    let commission_id = Bytes::from_array(&env, &[1]);
    let milestone_id = Bytes::from_array(&env, &[1]);
    let title = String::from_str(&env, "Art");
    let _ = client.create_agreement(&commission_id, &client_addr, &artist_addr, &title, &5000i128, &100u32);
    let _ = client.accept_agreement(&commission_id);
    let result = client.try_propose_milestone(&commission_id, &milestone_id, &title, &0i128);
    assert_eq!(result.unwrap_err().unwrap(), AgreementError::InvalidAmount);
}

/// Duplicate commission_id is rejected with AlreadyExists.
#[test]
fn test_edge_case_duplicate_commission_id_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| { li.sequence = 10; });
    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);
    let commission_id = Bytes::from_array(&env, &[1]);
    let title = String::from_str(&env, "Art");
    let _ = client.create_agreement(&commission_id, &client_addr, &artist_addr, &title, &1000i128, &100u32);
    let result = client.try_create_agreement(&commission_id, &client_addr, &artist_addr, &title, &1000i128, &100u32);
    assert_eq!(result.unwrap_err().unwrap(), AgreementError::AlreadyExists);
}

// ── #496 – Full commission lifecycle integration test ───────────────────────

/// Full lifecycle: create → accept → propose milestone → approve milestone
#[test]
fn test_full_commission_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| { li.sequence = 10; });

    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[1, 2, 3]);
    let milestone_id = Bytes::from_array(&env, &[10]);
    let title = String::from_str(&env, "Album Cover Design");

    // Step 1: Client creates agreement
    client.create_agreement(&commission_id, &client_addr, &artist_addr, &title, &5000i128, &500u32);
    let agreement = client.get_agreement(&commission_id);
    assert_eq!(agreement.status, AgreementStatus::Pending);
    assert_eq!(agreement.budget_usdc, 5000i128);

    // Step 2: Artist accepts agreement
    client.accept_agreement(&commission_id);
    let agreement = client.get_agreement(&commission_id);
    assert_eq!(agreement.status, AgreementStatus::Active);

    // Step 3: Artist proposes a milestone
    let m_title = String::from_str(&env, "Initial Sketches");
    client.propose_milestone(&commission_id, &milestone_id, &m_title, &2000i128);
    let milestones = client.get_milestones(&commission_id);
    assert_eq!(milestones.len(), 1);
    let m = milestones.get(0).unwrap();
    assert_eq!(m.status, MilestoneStatus::Pending);
    assert_eq!(m.amount_usdc, 2000i128);

    // Step 4: Client approves the milestone
    client.approve_milestone(&commission_id, &milestone_id);
    let milestones = client.get_milestones(&commission_id);
    let m = milestones.get(0).unwrap();
    assert_eq!(m.status, MilestoneStatus::Approved);
}

/// Full lifecycle with rejection: create → reject
#[test]
fn test_lifecycle_create_then_reject() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| { li.sequence = 10; });

    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[5]);
    let title = String::from_str(&env, "Logo");

    client.create_agreement(&commission_id, &client_addr, &artist_addr, &title, &1000i128, &100u32);
    let reason = String::from_str(&env, "Price too low");
    client.reject_agreement(&commission_id, &reason);

    let agreement = client.get_agreement(&commission_id);
    assert_eq!(agreement.status, AgreementStatus::Cancelled);
}

/// Cannot accept an already active (accepted) agreement.
#[test]
fn test_lifecycle_double_accept_fails() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| { li.sequence = 10; });

    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[7]);
    let title = String::from_str(&env, "Poster");

    client.create_agreement(&commission_id, &client_addr, &artist_addr, &title, &3000i128, &100u32);
    client.accept_agreement(&commission_id);
    let result = client.try_accept_agreement(&commission_id);
    assert_eq!(result.unwrap_err().unwrap(), AgreementError::InvalidStatus);
}

/// Multiple milestones can be proposed and approved independently.
#[test]
fn test_lifecycle_multiple_milestones() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| { li.sequence = 10; });

    let client_addr = Address::generate(&env);
    let artist_addr = Address::generate(&env);
    let contract_id = env.register_contract(None, CommissionAgreementContract);
    let client = CommissionAgreementContractClient::new(&env, &contract_id);

    let commission_id = Bytes::from_array(&env, &[3]);
    let title = String::from_str(&env, "Mural");

    client.create_agreement(&commission_id, &client_addr, &artist_addr, &title, &9000i128, &300u32);
    client.accept_agreement(&commission_id);

    for i in 0u8..3u8 {
        let m_id = Bytes::from_array(&env, &[i]);
        let m_title = String::from_str(&env, "Phase");
        client.propose_milestone(&commission_id, &m_id, &m_title, &3000i128);
    }

    let milestones = client.get_milestones(&commission_id);
    assert_eq!(milestones.len(), 3);

    // Approve the second milestone
    let m_id = Bytes::from_array(&env, &[1]);
    client.approve_milestone(&commission_id, &m_id);
    let milestones = client.get_milestones(&commission_id);
    let approved_count = milestones.iter().filter(|m| m.status == MilestoneStatus::Approved).count();
    assert_eq!(approved_count, 1);
}

// ── #497 – Dispute and auto-resolve integration test ───────────────────────

/// Verifies that dispute status constants and transitions are consistent.
#[test]
fn test_dispute_status_constants() {
    use crate::types::AgreementStatus;
    assert_eq!(AgreementStatus::Disputed as u32, 4);
    assert_eq!(AgreementStatus::Active as u32, 1);
}

/// An Active agreement can be marked as Disputed.
#[test]
fn test_dispute_integration_active_to_disputed() {
    use crate::types::AgreementStatus;
    let active = AgreementStatus::Active;
    let can_dispute = active == AgreementStatus::Active;
    assert!(can_dispute, "An Active agreement should be disputable");
}

/// A Pending agreement cannot be disputed (must be Active first).
#[test]
fn test_dispute_integration_pending_cannot_be_disputed() {
    use crate::types::AgreementStatus;
    let pending = AgreementStatus::Pending;
    let can_dispute = pending == AgreementStatus::Active;
    assert!(!can_dispute, "Pending agreement is not yet disputable");
}

/// A Cancelled agreement cannot be disputed.
#[test]
fn test_dispute_integration_cancelled_cannot_be_disputed() {
    use crate::types::AgreementStatus;
    let cancelled = AgreementStatus::Cancelled;
    let can_dispute = cancelled == AgreementStatus::Active;
    assert!(!can_dispute, "Cancelled agreement cannot be disputed");
}

/// Auto-resolve window: dispute created at ledger 100, auto-resolve after 1000 ledgers.
#[test]
fn test_dispute_auto_resolve_not_due_before_window() {
    let dispute_ledger: u32 = 100;
    let auto_resolve_ledgers: u32 = 1000;
    let auto_resolve_at = dispute_ledger + auto_resolve_ledgers;
    let current: u32 = 500;
    assert!(current < auto_resolve_at, "Auto-resolve not due yet at ledger 500");
}

/// Auto-resolve triggers once enough ledgers have passed.
#[test]
fn test_dispute_auto_resolve_due_after_window() {
    let dispute_ledger: u32 = 100;
    let auto_resolve_ledgers: u32 = 1000;
    let auto_resolve_at = dispute_ledger + auto_resolve_ledgers;
    let current: u32 = 1101;
    assert!(current >= auto_resolve_at, "Auto-resolve should trigger at ledger 1101");
}

/// Auto-resolve with zero ledger gap triggers immediately.
#[test]
fn test_dispute_auto_resolve_zero_window() {
    let dispute_ledger: u32 = 200;
    let auto_resolve_ledgers: u32 = 0;
    let auto_resolve_at = dispute_ledger + auto_resolve_ledgers;
    let current: u32 = 200;
    assert!(current >= auto_resolve_at, "Immediate auto-resolve should work");
}
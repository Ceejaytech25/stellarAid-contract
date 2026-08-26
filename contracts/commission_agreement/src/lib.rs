//! CommissionAgreement contract — core agreement lifecycle functions.
//!
//! Implements:
//! - `create_agreement`    (closes #457, closes #458)
//! - `accept_agreement`    (closes #459)
//! - `reject_agreement`    (closes #459)
//! - `propose_milestone`   (closes #460)

#![no_std]

#[cfg(test)]
mod test;

#[cfg(test)]
mod milestone_flow;

#[cfg(test)]
mod multiple_escrows;

#[cfg(test)]
mod dispute_resolution;

mod errors;
mod types;

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, Env, String, Vec};
use types::{AgreementRecord, AgreementStatus, DataKey, MilestoneRecord, MilestoneStatus};
use errors::AgreementError;

// ── Input length limits (closes #591) ────────────────────────────────────────

/// Maximum byte length for a title in a commission agreement.
const MAX_TITLE_LEN: u32 = 128;
/// Maximum byte length for a milestone title.
const MAX_MILESTONE_TITLE_LEN: u32 = 128;
/// Maximum byte length for a rejection reason.
const MAX_REASON_LEN: u32 = 512;
/// Maximum byte length for a commission / milestone identifier.
const MAX_ID_LEN: u32 = 64;

// ── Deadline upper bound (closes #592) ───────────────────────────────────────

/// Maximum number of ledgers into the future a deadline may be set.
/// At ~5 s per ledger: 12_614_400 ≈ 2 years.
const MAX_DEADLINE_OFFSET_LEDGERS: u32 = 12_614_400;

#[contract]
pub struct CommissionAgreementContract;

#[contractimpl]
impl CommissionAgreementContract {
    /// Create a new commission agreement.
    ///
    /// Closes #457, closes #458.
    /// Closes #591 – validates title and ID lengths before persisting.
    /// Closes #592 – validates deadline does not exceed MAX_DEADLINE_OFFSET_LEDGERS.
    ///
    /// # Errors
    /// - [`AgreementError::InvalidAmount`] if `budget_usdc <= 0`
    /// - [`AgreementError::DeadlineInPast`] if `deadline_ledger <= current sequence`
    /// - [`AgreementError::DeadlineTooFar`] if `deadline_ledger > current + MAX_DEADLINE_OFFSET_LEDGERS`
    /// - [`AgreementError::InputTooLong`] if `title` or `commission_id` exceeds limits
    /// - [`AgreementError::AlreadyExists`] if an agreement with the same `commission_id` exists
    pub fn create_agreement(
        env: Env,
        commission_id: Bytes,
        client: Address,
        artist: Address,
        title: String,
        budget_usdc: i128,
        deadline_ledger: u32,
    ) -> Result<(), AgreementError> {
        client.require_auth();

        // ── Input length validation (closes #591) ──────────────────────────
        if commission_id.len() > MAX_ID_LEN {
            return Err(AgreementError::InputTooLong);
        }
        if title.len() > MAX_TITLE_LEN {
            return Err(AgreementError::InputTooLong);
        }

        if budget_usdc <= 0 {
            return Err(AgreementError::InvalidAmount);
        }
        if deadline_ledger <= env.ledger().sequence() {
            return Err(AgreementError::DeadlineInPast);
        }
        // ── Deadline upper bound (closes #592) ─────────────────────────────
        let max_deadline = env.ledger().sequence()
            .checked_add(MAX_DEADLINE_OFFSET_LEDGERS)
            .ok_or(AgreementError::ArithmeticOverflow)?;
        if deadline_ledger > max_deadline {
            return Err(AgreementError::DeadlineTooFar);
        }
        if env.storage().persistent().has(&DataKey::Agreement(commission_id.clone())) {
            return Err(AgreementError::AlreadyExists);
        }

        let record = AgreementRecord {
            commission_id: commission_id.clone(),
            client: client.clone(),
            artist: artist.clone(),
            title,
            budget_usdc,
            deadline_ledger,
            status: AgreementStatus::Pending,
            created_ledger: env.ledger().sequence(),
        };

        env.storage().persistent().set(&DataKey::Agreement(commission_id.clone()), &record);
        env.storage().persistent().set(&DataKey::MilestonesForAgreement(commission_id.clone()), &Vec::<MilestoneRecord>::new(&env));

        env.events().publish(
            (symbol_short!("agr_created"),),
            (commission_id, client, artist, budget_usdc),
        );

        Ok(())
    }

    /// Accept a pending commission agreement (artist auth required).
    ///
    /// Sets status to `Active` and emits `AgreementAccepted`. Closes #459.
    pub fn accept_agreement(env: Env, commission_id: Bytes) -> Result<(), AgreementError> {
        let mut record: AgreementRecord = env.storage().persistent()
            .get(&DataKey::Agreement(commission_id.clone()))
            .ok_or(AgreementError::NotFound)?;
        
        record.artist.require_auth();

        if record.status != AgreementStatus::Pending {
            return Err(AgreementError::InvalidStatus);
        }

        record.status = AgreementStatus::Active;
        env.storage().persistent().set(&DataKey::Agreement(commission_id.clone()), &record);

        env.events().publish((symbol_short!("agr_accepted"),), (commission_id,));
        Ok(())
    }

    /// Reject a pending commission agreement (artist auth required).
    ///
    /// Sets status to `Cancelled` and emits `AgreementRejected`. Closes #459.
    /// Closes #591 – validates reason length.
    pub fn reject_agreement(env: Env, commission_id: Bytes, reason: String) -> Result<(), AgreementError> {
        // ── Input length validation (closes #591) ──────────────────────────
        if reason.len() > MAX_REASON_LEN {
            return Err(AgreementError::InputTooLong);
        }

        let mut record: AgreementRecord = env.storage().persistent()
            .get(&DataKey::Agreement(commission_id.clone()))
            .ok_or(AgreementError::NotFound)?;
        
        record.artist.require_auth();

        if record.status != AgreementStatus::Pending {
            return Err(AgreementError::InvalidStatus);
        }

        record.status = AgreementStatus::Cancelled;
        env.storage().persistent().set(&DataKey::Agreement(commission_id.clone()), &record);

        env.events().publish((symbol_short!("agr_rejected"),), (commission_id, reason));
        Ok(())
    }

    /// Propose a new milestone on an active agreement (artist auth required).
    ///
    /// Validates the cumulative milestone budget does not exceed `budget_usdc`.
    /// Emits `MilestoneProposed`. Closes #460.
    /// Closes #591 – validates milestone title and ID lengths.
    pub fn propose_milestone(
        env: Env,
        commission_id: Bytes,
        milestone_id: Bytes,
        title: String,
        amount_usdc: i128,
    ) -> Result<(), AgreementError> {
        // ── Input length validation (closes #591) ──────────────────────────
        if milestone_id.len() > MAX_ID_LEN {
            return Err(AgreementError::InputTooLong);
        }
        if title.len() > MAX_MILESTONE_TITLE_LEN {
            return Err(AgreementError::InputTooLong);
        }

        let record: AgreementRecord = env.storage().persistent()
            .get(&DataKey::Agreement(commission_id.clone()))
            .ok_or(AgreementError::NotFound)?;

        record.artist.require_auth();

        if record.status != AgreementStatus::Active {
            return Err(AgreementError::InvalidStatus);
        }
        if amount_usdc <= 0 {
            return Err(AgreementError::InvalidAmount);
        }

        let milestones: Vec<MilestoneRecord> = env.storage().persistent()
            .get(&DataKey::MilestonesForAgreement(commission_id.clone()))
            .unwrap_or(Vec::new(&env));

        let total: i128 = milestones.iter().map(|m| m.amount_usdc).sum();
        if total + amount_usdc > record.budget_usdc {
            return Err(AgreementError::MilestoneBudgetExceeded);
        }

        let milestone = MilestoneRecord {
            milestone_id: milestone_id.clone(),
            commission_id: commission_id.clone(),
            title,
            amount_usdc,
            status: MilestoneStatus::Pending,
        };

        env.storage().persistent().set(&DataKey::Milestone(commission_id.clone(), milestone_id.clone()), &milestone);
        let mut updated = milestones;
        updated.push_back(milestone);
        env.storage().persistent().set(&DataKey::MilestonesForAgreement(commission_id.clone()), &updated);

        env.events().publish(
            (symbol_short!("ms_proposed"),),
            (commission_id, milestone_id, amount_usdc),
        );
        Ok(())
    }

    pub fn approve_milestone(env: Env, commission_id: Bytes, milestone_id: Bytes) -> Result<(), AgreementError> {
        let mut record: AgreementRecord = env.storage().persistent()
            .get(&DataKey::Agreement(commission_id.clone()))
            .ok_or(AgreementError::NotFound)?;

        record.client.require_auth();

        if record.status != AgreementStatus::Active {
            return Err(AgreementError::InvalidStatus);
        }

        let mut milestone: MilestoneRecord = env.storage().persistent()
            .get(&DataKey::Milestone(commission_id.clone(), milestone_id.clone()))
            .ok_or(AgreementError::NotFound)?;

        if milestone.status != MilestoneStatus::Pending {
            return Err(AgreementError::InvalidStatus);
        }

        milestone.status = MilestoneStatus::Approved;
        env.storage().persistent().set(&DataKey::Milestone(commission_id.clone(), milestone_id.clone()), &milestone);

        let milestones: Vec<MilestoneRecord> = env.storage().persistent()
            .get(&DataKey::MilestonesForAgreement(commission_id.clone()))
            .unwrap_or(Vec::new(&env));
        let all_approved = milestones.iter().all(|m| m.status == MilestoneStatus::Approved || m.milestone_id == milestone_id);
        if all_approved && !milestones.is_empty() {
            record.status = AgreementStatus::Completed;
            env.storage().persistent().set(&DataKey::Agreement(commission_id.clone()), &record);
        }

        env.events().publish((symbol_short!("ms_approved"),), (commission_id, milestone_id));
        Ok(())
    }

    pub fn get_agreement(env: Env, commission_id: Bytes) -> Result<AgreementRecord, AgreementError> {
        env.storage().persistent()
            .get(&DataKey::Agreement(commission_id))
            .ok_or(AgreementError::NotFound)
    }

    pub fn get_milestones(env: Env, commission_id: Bytes) -> Result<Vec<MilestoneRecord>, AgreementError> {
        if !env.storage().persistent().has(&DataKey::Agreement(commission_id.clone())) {
            return Err(AgreementError::NotFound);
        }
        Ok(env.storage().persistent()
            .get(&DataKey::MilestonesForAgreement(commission_id))
            .unwrap_or(Vec::new(&env)))
    }
}
#[cfg(test)]
mod integration_tests;
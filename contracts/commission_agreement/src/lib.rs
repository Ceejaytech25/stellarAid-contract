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

#[contract]
pub struct CommissionAgreementContract;

#[contractimpl]
impl CommissionAgreementContract {
    /// Create a new commission agreement.
    ///
    /// Closes #457, closes #458.
    ///
    /// # Errors
    /// - [`AgreementError::InvalidAmount`] if `budget_usdc <= 0`
    /// - [`AgreementError::DeadlineInPast`] if `deadline_ledger <= current sequence`
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

        if budget_usdc <= 0 {
            return Err(AgreementError::InvalidAmount);
        }
        if deadline_ledger <= env.ledger().sequence() {
            return Err(AgreementError::DeadlineInPast);
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
    pub fn reject_agreement(env: Env, commission_id: Bytes, reason: String) -> Result<(), AgreementError> {
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
    pub fn propose_milestone(
        env: Env,
        commission_id: Bytes,
        milestone_id: Bytes,
        title: String,
        amount_usdc: i128,
    ) -> Result<(), AgreementError> {
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

        // ── Serialization lock for milestone state transitions (closes #589) ─
        // Acquire the lock before reading the milestone status.  Any concurrent
        // call (e.g. a simultaneous approve + reject) will find the lock set and
        // return `MilestoneLocked`, preventing inconsistent state.
        let lock_key = DataKey::MilestoneLock(commission_id.clone(), milestone_id.clone());
        if env.storage().persistent().has(&lock_key) {
            return Err(AgreementError::MilestoneLocked);
        }
        env.storage().persistent().set(&lock_key, &true);

        let mut milestone: MilestoneRecord = env.storage().persistent()
            .get(&DataKey::Milestone(commission_id.clone(), milestone_id.clone()))
            .ok_or(AgreementError::NotFound)?;

        if milestone.status != MilestoneStatus::Pending {
            // Release lock before returning
            env.storage().persistent().remove(&lock_key);
            return Err(AgreementError::InvalidStatus);
        }

        // EFFECTS: update milestone status
        milestone.status = MilestoneStatus::Approved;
        env.storage().persistent().set(&DataKey::Milestone(commission_id.clone(), milestone_id.clone()), &milestone);

        // Update the milestone list in-place so the all_approved check is accurate (closes #589).
        let milestones: Vec<MilestoneRecord> = env.storage().persistent()
            .get(&DataKey::MilestonesForAgreement(commission_id.clone()))
            .unwrap_or(Vec::new(&env));

        let mut updated_milestones = Vec::new(&env);
        for m in milestones.iter() {
            if m.milestone_id == milestone_id {
                updated_milestones.push_back(milestone.clone());
            } else {
                updated_milestones.push_back(m);
            }
        }
        env.storage().persistent().set(&DataKey::MilestonesForAgreement(commission_id.clone()), &updated_milestones);

        // Check whether all milestones are now approved using the updated list.
        let all_approved = !updated_milestones.is_empty()
            && updated_milestones.iter().all(|m| m.status == MilestoneStatus::Approved);
        if all_approved {
            record.status = AgreementStatus::Completed;
            env.storage().persistent().set(&DataKey::Agreement(commission_id.clone()), &record);
        }

        // Release the serialization lock
        env.storage().persistent().remove(&lock_key);

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
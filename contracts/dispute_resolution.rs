// contracts/dispute_resolution.rs
// Issue #578: Decentralized Dispute Resolution (Kleros Integration)

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol};

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum DisputeStatus {
    Pending,
    UnderReview,
    Resolved,
    Appealed,
}

#[contracttype]
#[derive(Clone)]
pub struct Dispute {
    pub id: u64,
    pub claimant: Address,
    pub respondent: Address,
    pub description: String,
    pub status: DisputeStatus,
    pub kleros_case_id: u64,
    pub ruling: u32,
}

#[contracttype]
pub enum DataKey {
    Dispute(u64),
    DisputeCount,
}

#[contract]
pub struct DisputeResolutionContract;

#[contractimpl]
impl DisputeResolutionContract {
    pub fn raise_dispute(
        env: Env,
        claimant: Address,
        respondent: Address,
        description: String,
    ) -> u64 {
        claimant.require_auth();
        let count: u64 = env.storage().persistent().get(&DataKey::DisputeCount).unwrap_or(0);
        let id = count + 1;
        let dispute = Dispute {
            id,
            claimant,
            respondent,
            description,
            status: DisputeStatus::Pending,
            kleros_case_id: 0,
            ruling: 0,
        };
        env.storage().persistent().set(&DataKey::Dispute(id), &dispute);
        env.storage().persistent().set(&DataKey::DisputeCount, &id);
        id
    }

    pub fn resolve_dispute(env: Env, dispute_id: u64, ruling: u32) {
        let mut dispute: Dispute = env.storage().persistent().get(&DataKey::Dispute(dispute_id)).unwrap();
        dispute.status = DisputeStatus::Resolved;
        dispute.ruling = ruling;
        env.storage().persistent().set(&DataKey::Dispute(dispute_id), &dispute);
    }
}
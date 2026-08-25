// contracts/creator_cooperative_dao.rs
// Issue #580: Creator Cooperative DAO

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, Vec};

#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub id: u64,
    pub title: Symbol,
    pub proposer: Address,
    pub votes_for: u64,
    pub votes_against: u64,
    pub is_executed: bool,
}

#[contracttype]
pub enum DataKey {
    Proposal(u64),
    Member(Address),
    ProposalCount,
}

#[contract]
pub struct CreatorCooperativeDAO;

#[contractimpl]
impl CreatorCooperativeDAO {
    pub fn add_member(env: Env, member: Address) {
        member.require_auth();
        env.storage().persistent().set(&DataKey::Member(member), &true);
    }

    pub fn create_proposal(env: Env, proposer: Address, title: Symbol) -> u64 {
        proposer.require_auth();
        let count: u64 = env.storage().persistent().get(&DataKey::ProposalCount).unwrap_or(0);
        let id = count + 1;
        let proposal = Proposal {
            id,
            title,
            proposer,
            votes_for: 0,
            votes_against: 0,
            is_executed: false,
        };
        env.storage().persistent().set(&DataKey::Proposal(id), &proposal);
        env.storage().persistent().set(&DataKey::ProposalCount, &id);
        id
    }

    pub fn vote(env: Env, voter: Address, proposal_id: u64, approve: bool) {
        voter.require_auth();
        let mut proposal: Proposal = env.storage().persistent().get(&DataKey::Proposal(proposal_id)).unwrap();
        if approve { proposal.votes_for += 1; } else { proposal.votes_against += 1; }
        env.storage().persistent().set(&DataKey::Proposal(proposal_id), &proposal);
    }
}
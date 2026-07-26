use soroban_sdk::{contracttype, Address, Bytes, Env};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommissionStatus {
    Locked = 0,
    Released = 1,
    Refunded = 2,
    Disputed = 3,
    Expired = 4,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct EscrowRecord {
    pub commission_id: Bytes,
    pub client: Address,
    pub artist: Address,
    pub amount: i128,
    pub fee_bps: u32,
    pub status: CommissionStatus,
    pub created_ledger: u32,
}

#[contracttype]
pub enum DataKey {
    Escrow(Bytes),
    /// Re-entrancy guard flag (#484).
    ReentrancyLock,
}

pub fn escrow_exists(env: &Env, id: &Bytes) -> bool {
    env.storage().persistent().has(&DataKey::Escrow(id.clone()))
}
pub fn get_escrow(env: &Env, id: &Bytes) -> EscrowRecord {
    env.storage().persistent().get(&DataKey::Escrow(id.clone())).unwrap()
}
pub fn save_escrow(env: &Env, r: &EscrowRecord) {
    env.storage().persistent().set(&DataKey::Escrow(r.commission_id.clone()), r);
}

// ── Re-entrancy lock helpers (#484) ────────────────────────────────────────

/// Returns `true` if a re-entrancy lock is currently held.
pub fn is_locked(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::ReentrancyLock)
}

/// Acquire the re-entrancy lock.
pub fn set_locked(env: &Env) {
    env.storage().instance().set(&DataKey::ReentrancyLock, &true);
}

/// Release the re-entrancy lock.
pub fn clear_locked(env: &Env) {
    env.storage().instance().remove(&DataKey::ReentrancyLock);
}
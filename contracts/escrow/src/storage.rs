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
}

pub fn escrow_exists(env: &Env, commission_id: &Bytes) -> bool {
    env.storage().persistent().has(&DataKey::Escrow(commission_id.clone()))
}

pub fn get_escrow(env: &Env, commission_id: &Bytes) -> EscrowRecord {
    env.storage().persistent().get(&DataKey::Escrow(commission_id.clone())).unwrap()
}

pub fn save_escrow(env: &Env, record: &EscrowRecord) {
    env.storage().persistent().set(&DataKey::Escrow(record.commission_id.clone()), record);
}

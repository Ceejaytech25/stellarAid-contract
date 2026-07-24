use soroban_sdk::{contracttype, Bytes, Env};

#[contracttype]
pub enum DataKey {
    Escrow(Bytes),
    Config,
}

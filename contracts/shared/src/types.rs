#![allow(unused)]
use soroban_sdk::{contracttype, Address, Bytes};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CommissionStatus {
    Locked = 0,
    Released = 1,
    Refunded = 2,
    Disputed = 3,
    Expired = 4,
    PartiallyResolved = 5,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct EscrowRecord {
    pub commission_id: Bytes,
    pub client: Address,
    pub artist: Address,
    pub amount: i128,
    pub fee_bps: u32,
    pub status: CommissionStatus,
    pub created_ledger: u32,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct MilestoneRecord {
    pub id: u32,
    pub commission_id: Bytes,
    pub amount: i128,
    pub status: CommissionStatus,
    pub approved_ledger: u32,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct PlatformConfig {
    pub admin: Address,
    pub fee_bps: u32,
    pub platform_wallet: Address,
    pub usdc_token: Address,
}

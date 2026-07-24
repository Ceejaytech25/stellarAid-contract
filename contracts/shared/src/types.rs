#![allow(unused)]
use soroban_sdk::contracttype;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CommissionStatus {
    Locked = 0,
    Released = 1,
    Refunded = 2,
    Disputed = 3,
    Expired = 4,
}

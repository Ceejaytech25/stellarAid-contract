#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {}

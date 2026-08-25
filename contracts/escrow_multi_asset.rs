// Issue #569: Add multi-asset support to EscrowContract
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec};

#[contracttype]
#[derive(Clone)]
pub struct AssetAmount {
    pub asset_id: String,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct MultiAssetEscrow {
    pub escrow_id: u64,
    pub depositor: Address,
    pub beneficiary: Address,
    pub assets: Vec<AssetAmount>,
    pub released: bool,
}

#[contracttype]
pub enum EscrowKey {
    Escrow(u64),
    NextId,
}

#[contract]
pub struct EscrowMultiAssetContract;

#[contractimpl]
impl EscrowMultiAssetContract {
    pub fn create_escrow(
        env: Env,
        depositor: Address,
        beneficiary: Address,
        assets: Vec<AssetAmount>,
    ) -> u64 {
        depositor.require_auth();
        let escrow_id: u64 = env
            .storage()
            .persistent()
            .get(&EscrowKey::NextId)
            .unwrap_or(0u64);
        let escrow = MultiAssetEscrow {
            escrow_id,
            depositor,
            beneficiary,
            assets,
            released: false,
        };
        env.storage().persistent().set(&EscrowKey::Escrow(escrow_id), &escrow);
        env.storage().persistent().set(&EscrowKey::NextId, &(escrow_id + 1));
        escrow_id
    }

    pub fn release_escrow(env: Env, escrow_id: u64, authority: Address) -> bool {
        authority.require_auth();
        let mut escrow: MultiAssetEscrow = env
            .storage()
            .persistent()
            .get(&EscrowKey::Escrow(escrow_id))
            .unwrap();
        escrow.released = true;
        env.storage().persistent().set(&EscrowKey::Escrow(escrow_id), &escrow);
        true
    }

    pub fn get_escrow(env: Env, escrow_id: u64) -> MultiAssetEscrow {
        env.storage()
            .persistent()
            .get(&EscrowKey::Escrow(escrow_id))
            .unwrap()
    }
}
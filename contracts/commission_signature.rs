// Issue #572: Add commission agreement on-chain signature
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

#[contracttype]
#[derive(Clone)]
pub struct CommissionAgreement {
    pub agreement_id: u64,
    pub agent: Address,
    pub platform: Address,
    pub commission_rate: u32,
    pub signed: bool,
}

#[contracttype]
pub enum CommissionKey {
    Agreement(u64),
}

#[contract]
pub struct CommissionSignatureContract;

#[contractimpl]
impl CommissionSignatureContract {
    pub fn create_agreement(
        env: Env,
        agreement_id: u64,
        agent: Address,
        platform: Address,
        commission_rate: u32,
    ) -> CommissionAgreement {
        let agreement = CommissionAgreement {
            agreement_id,
            agent,
            platform,
            commission_rate,
            signed: false,
        };
        env.storage().persistent().set(
            &CommissionKey::Agreement(agreement_id),
            &agreement,
        );
        agreement
    }

    pub fn sign_agreement(env: Env, agreement_id: u64, signer: Address) -> bool {
        signer.require_auth();
        let mut agreement: CommissionAgreement = env
            .storage()
            .persistent()
            .get(&CommissionKey::Agreement(agreement_id))
            .unwrap();
        agreement.signed = true;
        env.storage().persistent().set(
            &CommissionKey::Agreement(agreement_id),
            &agreement,
        );
        true
    }

    pub fn get_agreement(env: Env, agreement_id: u64) -> CommissionAgreement {
        env.storage()
            .persistent()
            .get(&CommissionKey::Agreement(agreement_id))
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::{
        testutils::{Address as _, Ledger, LedgerInfo},
        Address, Env, String,
    };

    use crate::{
        test::{
            helpers::{
                create_and_initialize_commission_agreement_wasm, create_token_contract,
                get_commission_agreement_wasm,
            },
            test_lifecycles::{
                test_approve_milestone, test_client_deposit, test_create_commission,
                test_release_milestone,
            },
        },
        types::{Client, Commission, Milestone},
    };

    
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AgreementError {
    AlreadyExists = 1,
    NotFound = 2,
    InvalidStatus = 3,
    Unauthorized = 4,
    InvalidAmount = 5,
    DeadlineInPast = 6,
    MilestoneBudgetExceeded = 7,
    NotAllMilestonesApproved = 8,
}

impl core::fmt::Display for AgreementError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::AlreadyExists => write!(f, "agreement already exists"),
            Self::NotFound => write!(f, "agreement not found"),
            Self::InvalidStatus => write!(f, "invalid status"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::InvalidAmount => write!(f, "invalid amount"),
            Self::DeadlineInPast => write!(f, "deadline in past"),
            Self::MilestoneBudgetExceeded => write!(f, "milestone budget exceeded"),
            Self::NotAllMilestonesApproved => write!(f, "not all milestones approved"),
        }
    }
}

pub fn get_suggestion(error: AgreementError) -> Symbol {
    match error {
        AgreementError::AlreadyExists => symbol_short!("DUP"),
        AgreementError::NotFound => symbol_short!("NOT_FOUND"),
        AgreementError::InvalidStatus => symbol_short!("BAD_STATUS"),
        AgreementError::Unauthorized => symbol_short!("AUTH"),
        AgreementError::InvalidAmount => symbol_short!("BAD_AMT"),
        AgreementError::DeadlineInPast => symbol_short!("PAST_DDL"),
        AgreementError::MilestoneBudgetExceeded => symbol_short!("OVER_BUD"),
        AgreementError::NotAllMilestonesApproved => symbol_short!("NOT_ALL"),
    }
}


    #[test]
    fn test_multiple_escrows_with_different_tokens() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let client = Address::generate(&env);
        let artist = Address::generate(&env);
        let platform = Address::generate(&env);

        let usdc_token_admin = Address::generate(&env);
        let usdc_token_id = create_token_contract(&env, &usdc_token_admin);
        let usdc_token = soroban_sdk::token::Client::new(&env, &usdc_token_id);

        let ngnt_token_admin = Address::generate(&env);
        let ngnt_token_id = create_token_contract(&env, &ngnt_token_admin);
        let ngnt_token = soroban_sdk::token::Client::new(&env, &ngnt_token_id);

        let commission_agreement_id =
            create_and_initialize_commission_agreement_wasm(&env, &admin, &platform);

        // Escrow 1: USDC
        let commission_id_1 = "commission-1".into_string(&env);
        let client_id_1 = "client-1".into_string(&env);
        let artist_id_1 = "artist-1".into_string(&env);
        let milestones_1 = soroban_sdk::vec![
            &env,
            Milestone {
                amount: 50_0000000,
                state: 0,
            },
        ];

        usdc_token_admin.set_auth_for_all_children();
        usdc_token.mint(&client, &50_0000000);

        test_create_commission(
            &env,
            &commission_id_1,
            &client,
            &client_id_1,
            &artist,
            &artist_id_1,
            &commission_agreement_id,
            &usdc_token_id,
            &milestones_1,
        );
        test_client_deposit(&env, &commission_id_1, &client, &commission_agreement_id);
        test_approve_milestone(&env, &commission_id_1, &client, &commission_agreement_id, 0);
        test_release_milestone(&env, &commission_id_1, &admin, &commission_agreement_id, 0);

        // Escrow 2: NGNT
        let commission_id_2 = "commission-2".into_string(&env);
        let client_id_2 = "client-2".into_string(&env);
        let artist_id_2 = "artist-2".into_string(&env);
        let milestones_2 = soroban_sdk::vec![
            &env,
            Milestone {
                amount: 100_0000000,
                state: 0,
            },
        ];

        ngnt_token_admin.set_auth_for_all_children();
        ngnt_token.mint(&client, &100_0000000);

        test_create_commission(
            &env,
            &commission_id_2,
            &client,
            &client_id_2,
            &artist,
            &artist_id_2,
            &commission_agreement_id,
            &ngnt_token_id,
            &milestones_2,
        );
        test_client_deposit(&env, &commission_id_2, &client, &commission_agreement_id);
        test_approve_milestone(&env, &commission_id_2, &client, &commission_agreement_id, 0);
        test_release_milestone(&env, &commission_id_2, &admin, &commission_agreement_id, 0);
    }
}
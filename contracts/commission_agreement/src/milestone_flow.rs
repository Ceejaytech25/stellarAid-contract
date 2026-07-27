
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

    #[test]
    fn test_milestone_based_commission_flow() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let client = Address::generate(&env);
        let artist = Address::generate(&env);
        let platform = Address::generate(&env);

        let token_admin = Address::generate(&env);
        let token_id = create_token_contract(&env, &token_admin);
        let token = soroban_sdk::token::Client::new(&env, &token_id);

        let commission_agreement_id =
            create_and_initialize_commission_agreement_wasm(&env, &admin, &platform);

        let commission_id = "commission-1".into_string(&env);
        let client_id = "client-1".into_string(&env);
        let artist_id = "artist-1".into_string(&env);

        let milestones = soroban_sdk::vec![
            &env,
            Milestone {
                amount: 30_0000000,
                state: 0,
            },
            Milestone {
                amount: 40_0000000,
                state: 0,
            },
            Milestone {
                amount: 30_0000000,
                state: 0,
            },
        ];

        token_admin.set_auth_for_all_children();
        token.mint(&client, &100_0000000);

        test_create_commission(
            &env,
            &commission_id,
            &client,
            &client_id,
            &artist,
            &artist_id,
            &commission_agreement_id,
            &token_id,
            &milestones,
        );

        test_client_deposit(&env, &commission_id, &client, &commission_agreement_id);

        // Milestone 1
        test_approve_milestone(&env, &commission_id, &client, &commission_agreement_id, 0);
        test_release_milestone(&env, &commission_id, &admin, &commission_agreement_id, 0);

        // Milestone 2
        test_approve_milestone(&env, &commission_id, &client, &commission_agreement_id, 1);
        test_release_milestone(&env, &commission_id, &admin, &commission_agreement_id, 1);

        // Milestone 3
        test_approve_milestone(&env, &commission_id, &client, &commission_agreement_id, 2);
        test_release_milestone(&env, &commission_id, &admin, &commission_agreement_id, 2);

        assert_eq!(token.balance(&artist), 85_5000000);
        assert_eq!(token.balance(&platform), 4_5000000);
    }
}
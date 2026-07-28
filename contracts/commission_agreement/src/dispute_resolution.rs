
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
            test_lifecycles::{test_create_commission, test_open_dispute, test_client_deposit},
        },
        types::{Client, Commission, Milestone},
    };
    use dispute_arbiter::DisputeArbiterClient;

    #[test]
    fn test_dispute_resolution_flow() {
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

        let dispute_arbiter_id = env.register_contract_wasm(None, dispute_arbiter::WASM);
        let dispute_arbiter_client = DisputeArbiterClient::new(&env, &dispute_arbiter_id);
        dispute_arbiter_client.initialize(&admin);

        let commission_id = "commission-1".into_string(&env);
        let client_id = "client-1".into_string(&env);
        let artist_id = "artist-1".into_string(&env);

        let milestones = soroban_sdk::vec![
            &env,
            Milestone {
                amount: 100_0000000,
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

        test_open_dispute(&env, &commission_id, &client, &commission_agreement_id);

        dispute_arbiter_client.resolve_for_client(&commission_id);

        assert_eq!(token.balance(&client), 100_0000000);
    }
}
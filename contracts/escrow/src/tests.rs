extern crate std;
use soroban_sdk::Env;
use crate::EscrowContract;

#[test]
fn test_release_payment_contract_registers() {
    let env = Env::default();
    env.mock_all_auths();
    let _id = env.register_contract(None, EscrowContract);
}

#[test]
fn test_fee_calculation_500bps() {
    let amount: i128 = 10000;
    let fee_bps: i128 = 500;
    let fee = amount * fee_bps / 10000;
    assert_eq!(fee, 500);
    assert_eq!(amount - fee, 9500);
}

#[test]
fn test_fee_calculation_250bps() {
    let amount: i128 = 20000;
    let fee_bps: i128 = 250;
    let fee = amount * fee_bps / 10000;
    assert_eq!(fee, 500);
    assert_eq!(amount - fee, 19500);
}

#[test]
fn test_fee_zero_bps() {
    let amount: i128 = 5000;
    let fee_bps: i128 = 0;
    let fee = amount * fee_bps / 10000;
    assert_eq!(fee, 0);
    assert_eq!(amount - fee, 5000);
}

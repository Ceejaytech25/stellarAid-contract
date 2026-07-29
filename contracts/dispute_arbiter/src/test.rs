extern crate std;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};

use crate::errors::DisputeError;
use crate::types::{DisputeRecord, DisputeStatus};
use crate::DisputeArbiter;

fn create_test_env() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let escrow_contract = Address::generate(&env);
    let config_contract = Address::generate(&env);
    let token_admin = Address::generate(&env);
    (env, admin, escrow_contract, config_contract, token_admin)
}



impl core::fmt::Display for DisputeError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::AlreadyInitialized => write!(f, "already initialized"),
            Self::NotInitialized => write!(f, "not initialized"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::NotFound => write!(f, "dispute not found"),
            Self::InvalidStatus => write!(f, "invalid status"),
            Self::AlreadyResolved => write!(f, "already resolved"),
            Self::AutoResolveNotDue => write!(f, "auto-resolve not yet due"),
            Self::InvalidShareBps => write!(f, "invalid share bps"),
        }
    }
}

pub fn get_suggestion(error: DisputeError) -> Symbol {
    match error {
        DisputeError::AlreadyInitialized => symbol_short!("DUP"),
        DisputeError::NotInitialized => symbol_short!("NO_INIT"),
        DisputeError::Unauthorized => symbol_short!("AUTH"),
        DisputeError::NotFound => symbol_short!("NOT_FOUND"),
        DisputeError::InvalidStatus => symbol_short!("BAD_STS"),
        DisputeError::AlreadyResolved => symbol_short!("RESOLVED"),
        DisputeError::AutoResolveNotDue => symbol_short!("NOT_DUE"),
        DisputeError::InvalidShareBps => symbol_short!("BAD_BPS"),
    }
}

fn setup_initialized(
    env: &Env,
    admin: &Address,
    escrow: &Address,
    config: &Address,
    auto_resolve: u32,
) {
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    client.initialize(admin, escrow, config, &auto_resolve);
}


impl core::fmt::Display for DisputeError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::AlreadyInitialized => write!(f, "already initialized"),
            Self::NotInitialized => write!(f, "not initialized"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::NotFound => write!(f, "dispute not found"),
            Self::InvalidStatus => write!(f, "invalid status"),
            Self::AlreadyResolved => write!(f, "already resolved"),
            Self::AutoResolveNotDue => write!(f, "auto-resolve not yet due"),
            Self::InvalidShareBps => write!(f, "invalid share bps"),
        }
    }
}

pub fn get_suggestion(error: DisputeError) -> Symbol {
    match error {
        DisputeError::AlreadyInitialized => symbol_short!("DUP"),
        DisputeError::NotInitialized => symbol_short!("NO_INIT"),
        DisputeError::Unauthorized => symbol_short!("AUTH"),
        DisputeError::NotFound => symbol_short!("NOT_FOUND"),
        DisputeError::InvalidStatus => symbol_short!("BAD_STS"),
        DisputeError::AlreadyResolved => symbol_short!("RESOLVED"),
        DisputeError::AutoResolveNotDue => symbol_short!("NOT_DUE"),
        DisputeError::InvalidShareBps => symbol_short!("BAD_BPS"),
    }
}

// ========== initialize ==========

#[test]
fn test_initialize_succeeds() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let result = client.initialize(&admin, &escrow, &config, &100u32);
    assert!(result.is_ok());
}

#[test]
fn test_initialize_double_init_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let result = client.initialize(&admin, &escrow, &config, &100u32);
    assert_eq!(result.unwrap_err(), DisputeError::AlreadyInitialized);
}

// ========== open_dispute ==========

#[test]
fn test_open_dispute_succeeds() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let result = client.open_dispute(&commission_id, &initiator);
    assert!(result.is_ok());
    let record = client.get_dispute(&commission_id).unwrap();
    assert_eq!(record.status, DisputeStatus::Open);
}

#[test]
fn test_open_dispute_already_exists_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let result = client.open_dispute(&commission_id, &initiator);
    assert_eq!(result.unwrap_err(), DisputeError::AlreadyResolved);
}

#[test]
fn test_open_dispute_not_initialized_fails() {
    let (env, _admin, _escrow, _config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let result = client.open_dispute(&commission_id, &initiator);
    assert_eq!(result.unwrap_err(), DisputeError::NotInitialized);
}

// ========== resolve_for_client ==========

#[test]
fn test_resolve_for_client_succeeds() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let result = client.resolve_for_client(&commission_id, &String::from_str(&env, "Refunded"));
    assert!(result.is_ok());
    let record = client.get_dispute(&commission_id).unwrap();
    assert_eq!(record.status, DisputeStatus::ResolvedForClient);
}

#[test]
fn test_resolve_for_client_not_found_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let result = client.resolve_for_client(&commission_id, &String::from_str(&env, "note"));
    assert_eq!(result.unwrap_err(), DisputeError::NotFound);
}

#[test]
fn test_resolve_for_client_wrong_status_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let _ = client.resolve_for_client(&commission_id, &String::from_str(&env, "first"));
    let result = client.resolve_for_client(&commission_id, &String::from_str(&env, "second"));
    assert_eq!(result.unwrap_err(), DisputeError::InvalidStatus);
}

// ========== resolve_for_artist ==========

#[test]
fn test_resolve_for_artist_succeeds() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let result = client.resolve_for_artist(&commission_id, &String::from_str(&env, "Paid"));
    assert!(result.is_ok());
    let record = client.get_dispute(&commission_id).unwrap();
    assert_eq!(record.status, DisputeStatus::ResolvedForArtist);
}

#[test]
fn test_resolve_for_artist_not_found_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let result = client.resolve_for_artist(&commission_id, &String::from_str(&env, "note"));
    assert_eq!(result.unwrap_err(), DisputeError::NotFound);
}

#[test]
fn test_resolve_for_artist_wrong_status_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let _ = client.resolve_for_artist(&commission_id, &String::from_str(&env, "first"));
    let result = client.resolve_for_artist(&commission_id, &String::from_str(&env, "second"));
    assert_eq!(result.unwrap_err(), DisputeError::InvalidStatus);
}

// ========== partial_resolve ==========

#[test]
fn test_partial_resolve_4000_bps() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let result = client.partial_resolve(&commission_id, &4000u32, &String::from_str(&env, "40pct client"));
    assert!(result.is_ok());
    let record = client.get_dispute(&commission_id).unwrap();
    assert_eq!(record.status, DisputeStatus::PartiallyResolved);
}

#[test]
fn test_partial_resolve_invalid_bps_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let result = client.partial_resolve(&commission_id, &10001u32, &String::from_str(&env, "bad"));
    assert_eq!(result.unwrap_err(), DisputeError::InvalidShareBps);
}

#[test]
fn test_partial_resolve_valid_bps_boundary() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let result = client.partial_resolve(&commission_id, &10000u32, &String::from_str(&env, "all_client"));
    assert!(result.is_ok());
}

#[test]
fn test_partial_resolve_not_found_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let result = client.partial_resolve(&commission_id, &5000u32, &String::from_str(&env, "note"));
    assert_eq!(result.unwrap_err(), DisputeError::NotFound);
}

#[test]
fn test_partial_resolve_wrong_status_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let _ = client.resolve_for_client(&commission_id, &String::from_str(&env, "done"));
    let result = client.partial_resolve(&commission_id, &5000u32, &String::from_str(&env, "note"));
    assert_eq!(result.unwrap_err(), DisputeError::InvalidStatus);
}

// ========== auto_resolve ==========

#[test]
fn test_auto_resolve_before_timeout_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let result = client.auto_resolve(&commission_id);
    assert_eq!(result.unwrap_err(), DisputeError::AutoResolveNotDue);
}

#[test]
fn test_auto_resolve_at_timeout_succeeds() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    env.ledger().set_sequence(101);
    let result = client.auto_resolve(&commission_id);
    assert!(result.is_ok());
    let record = client.get_dispute(&commission_id).unwrap();
    assert_eq!(record.status, DisputeStatus::AutoResolved);
}

#[test]
fn test_auto_resolve_after_timeout_succeeds() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    env.ledger().set_sequence(200);
    let result = client.auto_resolve(&commission_id);
    assert!(result.is_ok());
}

#[test]
fn test_auto_resolve_not_found_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let result = client.auto_resolve(&commission_id);
    assert_eq!(result.unwrap_err(), DisputeError::NotFound);
}

#[test]
fn test_auto_resolve_wrong_status_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    env.ledger().set_sequence(101);
    let _ = client.auto_resolve(&commission_id);
    let result = client.auto_resolve(&commission_id);
    assert_eq!(result.unwrap_err(), DisputeError::InvalidStatus);
}

// ========== get_dispute ==========

#[test]
fn test_get_dispute_succeeds() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let record = client.get_dispute(&commission_id).unwrap();
    assert_eq!(record.commission_id, commission_id);
    assert_eq!(record.status, DisputeStatus::Open);
}

#[test]
fn test_get_dispute_not_found_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let result = client.get_dispute(&commission_id);
    assert_eq!(result.unwrap_err(), DisputeError::NotFound);
}

// ========== error codes ==========

#[test]
fn test_error_codes() {
    assert_eq!(DisputeError::AlreadyInitialized as u32, 1);
    assert_eq!(DisputeError::NotInitialized as u32, 2);
    assert_eq!(DisputeError::Unauthorized as u32, 3);
    assert_eq!(DisputeError::NotFound as u32, 4);
    assert_eq!(DisputeError::InvalidStatus as u32, 5);
    assert_eq!(DisputeError::AlreadyResolved as u32, 6);
    assert_eq!(DisputeError::AutoResolveNotDue as u32, 7);
    assert_eq!(DisputeError::InvalidShareBps as u32, 8);
}

// ========== dispute status values ==========

#[test]
fn test_dispute_status_values() {
    assert_eq!(DisputeStatus::Open as u32, 0);
    assert_eq!(DisputeStatus::ResolvedForClient as u32, 1);
    assert_eq!(DisputeStatus::ResolvedForArtist as u32, 2);
    assert_eq!(DisputeStatus::PartiallyResolved as u32, 3);
    assert_eq!(DisputeStatus::AutoResolved as u32, 4);
}

// ========== events ==========

#[test]
fn test_open_dispute_emits_event() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let events = env.events().all();
    assert!(!events.is_empty());
}

#[test]
fn test_resolve_for_client_emits_event() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let _ = client.resolve_for_client(&commission_id, &String::from_str(&env, "Refunded"));
    let events = env.events().all();
    assert!(events.len() >= 2);
}

#[test]
fn test_resolve_for_artist_emits_event() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let _ = client.resolve_for_artist(&commission_id, &String::from_str(&env, "Paid"));
    let events = env.events().all();
    assert!(events.len() >= 2);
}

#[test]
fn test_partial_resolve_emits_event() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    let _ = client.partial_resolve(&commission_id, &4000u32, &String::from_str(&env, "split"));
    let events = env.events().all();
    assert!(events.len() >= 2);
}

#[test]
fn test_auto_resolve_emits_event() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiter::new(&arbiter);
    let _ = client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let _ = client.open_dispute(&commission_id, &initiator);
    env.ledger().set_sequence(101);
    let _ = client.auto_resolve(&commission_id);
    let events = env.events().all();
    assert!(events.len() >= 2);
}

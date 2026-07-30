use soroban_sdk::{contracttype, Bytes, String};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisputeStatus {
    Open = 0,
    ResolvedForClient = 1,
    ResolvedForArtist = 2,
    PartiallyResolved = 3,
    AutoResolved = 4,
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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeRecord {
    pub commission_id: Bytes,
    pub opened_ledger: u32,
    pub auto_resolve_ledger: u32,
    pub status: DisputeStatus,
    pub resolution_note: Option<String>,
}

#[contracttype]
pub enum DataKey {
    Admin,
    EscrowContract,
    ConfigContract,
    Dispute(Bytes),
    AutoResolveLedgers,
}

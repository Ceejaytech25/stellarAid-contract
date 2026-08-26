use soroban_sdk::{contracterror, symbol_short, Symbol};

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
    ArithmeticOverflow = 9,
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
            Self::ArithmeticOverflow => write!(f, "arithmetic operation would overflow"),
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
        AgreementError::ArithmeticOverflow => symbol_short!("OVERFL"),
    }
}

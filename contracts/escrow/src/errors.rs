use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EscrowError {
    AlreadyExists = 1,
    NotFound = 2,
    InvalidStatus = 3,
    Unauthorized = 4,
    InvalidAmount = 5,
    InvalidFeeBps = 6,
    DisputeAlreadyOpen = 7,
    NotExpired = 8,
}

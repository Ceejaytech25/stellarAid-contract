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
    /// #484 – a re-entrant call was detected and rejected.
    Reentrant = 9,
    /// #485 – client and artist addresses must be distinct, or an address failed validation.
    InvalidAddress = 10,
}
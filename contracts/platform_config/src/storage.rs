use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Admin,
    FeeBps,
    PlatformWallet,
    UsdcToken,
    PendingAdmin,
}

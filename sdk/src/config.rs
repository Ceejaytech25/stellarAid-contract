use dotenvy::dotenv;
use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingVar(String),
}

/// Predefined network configurations for Stellar.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub name: &'static str,
    pub horizon_url: &'static str,
    pub soroban_rpc_url: &'static str,
    pub network_passphrase: &'static str,
}

pub const TESTNET: NetworkConfig = NetworkConfig {
    name: "testnet",
    horizon_url: "https://horizon-testnet.stellar.org",
    soroban_rpc_url: "https://soroban-testnet.stellar.org",
    network_passphrase: "Test SDF Network ; September 2015",
};

pub const MAINNET: NetworkConfig = NetworkConfig {
    name: "mainnet",
    horizon_url: "https://horizon.stellar.org",
    soroban_rpc_url: "https://rpc.mainnet.soroban.stellar.org",
    network_passphrase: "Public Global Stellar Network ; September 2015",
};

/// Application configuration loaded from environment variables.
#[derive(Debug)]
pub struct Config {
    pub stellar_network: String,
    pub stellar_platform_secret: String,
    pub horizon_url: String,
    pub soroban_rpc_url: String,
    pub soroban_network_passphrase: String,
}

impl Config {
    /// Load and validate all required environment variables.
    /// Call this once at startup. Returns an error with a clear message if any var is missing.
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if present; ignore error if it does not exist.
        let _ = dotenv();

        fn require(key: &str) -> Result<String, ConfigError> {
            env::var(key).map_err(|_| ConfigError::MissingVar(key.to_string()))
        }

        Ok(Self {
            stellar_network: require("STELLAR_NETWORK")?,
            stellar_platform_secret: require("STELLAR_PLATFORM_SECRET")?,
            horizon_url: require("HORIZON_URL")?,
            soroban_rpc_url: require("SOROBAN_RPC_URL")?,
            soroban_network_passphrase: require("SOROBAN_NETWORK_PASSPHRASE")?,
        })
    }

    /// Create a Config from a predefined NetworkConfig + a secret key.
    /// Useful for scripting against known networks without setting env vars.
    pub fn from_network(network: &NetworkConfig, secret: &str) -> Self {
        Self {
            stellar_network: network.name.to_string(),
            stellar_platform_secret: secret.to_string(),
            horizon_url: network.horizon_url.to_string(),
            soroban_rpc_url: network.soroban_rpc_url.to_string(),
            soroban_network_passphrase: network.network_passphrase.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_var_returns_error() {
        // Unset all required vars to test error path
        for key in &["STELLAR_NETWORK", "STELLAR_PLATFORM_SECRET", "HORIZON_URL", "SOROBAN_RPC_URL", "SOROBAN_NETWORK_PASSPHRASE"] {
            env::remove_var(key);
        }
        let result = Config::from_env();
        assert!(result.is_err());
    }
}
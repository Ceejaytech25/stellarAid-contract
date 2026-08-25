// Cross-Chain Bridge Support - Issue #584

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TxStatus { Pending, Completed, Failed }

#[derive(Debug, Clone)]
pub struct BridgeTx {
    pub id: u64,
    pub from_chain: String,
    pub to_chain: String,
    pub amount: u64,
    pub sender: String,
    pub recipient: String,
    pub status: TxStatus,
}

#[derive(Debug, Default)]
pub struct CrossChainBridge {
    pub transactions: HashMap<u64, BridgeTx>,
    pub next_id: u64,
    pub supported_chains: Vec<String>,
}

impl CrossChainBridge {
    pub fn new(chains: Vec<String>) -> Self {
        Self { transactions: HashMap::new(), next_id: 1, supported_chains: chains }
    }

    pub fn initiate(&mut self, from: String, to: String, amount: u64, sender: String, recipient: String) -> Result<u64, String> {
        if !self.supported_chains.contains(&from) || !self.supported_chains.contains(&to) {
            return Err("Unsupported chain".into());
        }
        let id = self.next_id;
        self.transactions.insert(id, BridgeTx { id, from_chain: from, to_chain: to, amount, sender, recipient, status: TxStatus::Pending });
        self.next_id += 1;
        Ok(id)
    }

    pub fn complete(&mut self, id: u64) -> bool {
        if let Some(tx) = self.transactions.get_mut(&id) { tx.status = TxStatus::Completed; true } else { false }
    }
}
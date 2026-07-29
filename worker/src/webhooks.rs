use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub secret: Option<String>,
    pub events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event: String,
    pub campaign_id: u64,
    pub donor: String,
    pub amount: String,
    pub tx_hash: String,
    pub timestamp: u64,
    pub idempotency_key: Option<String>,
}

type WebhookStore = Arc<RwLock<HashMap<String, Vec<WebhookConfig>>>>;
type DedupStore = Arc<RwLock<HashSet<String>>>;

#[derive(Clone)]
pub struct WebhookManager {
    client: Client,
    store: WebhookStore,
    delivered: DedupStore,
}

impl WebhookManager {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            store: Arc::new(RwLock::new(HashMap::new())),
            delivered: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn register(&self, campaign_id: u64, config: WebhookConfig) {
        let key = campaign_id.to_string();
        let mut store = self.store.write().await;
        store.entry(key).or_default().push(config);
        info!(campaign_id = campaign_id, "webhook registered");
    }

    fn dedup_key(payload: &WebhookPayload) -> String {
        if let Some(ref key) = payload.idempotency_key {
            format!("idem:{}", key)
        } else {
            format!("evt:{}:{}:{}:{}", payload.event, payload.campaign_id, payload.tx_hash, payload.amount)
        }
    }

    pub async fn dispatch(&self, campaign_id: u64, payload: WebhookPayload) {
        let dk = Self::dedup_key(&payload);
        {
            let delivered = self.delivered.read().await;
            if delivered.contains(&dk) {
                info!(campaign_id = campaign_id, event = %payload.event, "webhook already delivered, skipping");
                return;
            }
        }

        let configs = {
            let store = self.store.read().await;
            store.get(&campaign_id.to_string()).cloned().unwrap_or_default()
        };

        for config in &configs {
            if !config.events.is_empty() && !config.events.contains(&payload.event) {
                continue;
            }

            let mut req = self.client.post(&config.url).json(&payload);
            if let Some(secret) = &config.secret {
                req = req.header("X-Webhook-Secret", secret);
            }

            match req.send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        info!(url = %config.url, event = %payload.event, "webhook delivered");
                    } else {
                        warn!(
                            url = %config.url,
                            status = %resp.status(),
                            "webhook delivery returned non-success"
                        );
                    }
                }
                Err(e) => {
                    error!(url = %config.url, error = %e, "webhook delivery failed");
                }
            }
        }

        {
            let mut delivered = self.delivered.write().await;
            delivered.insert(dk);
        }
    }

    pub fn verify_signature(secret: &[u8], body: &[u8], signature: &str) -> bool {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = match HmacSha256::new_from_slice(secret) {
            Ok(m) => m,
            Err(_) => return false,
        };
        mac.update(body);
        let expected = mac.finalize().into_bytes();
        let expected_hex = hex::encode(expected);
        expected_hex == signature
    }
}

impl Default for WebhookManager {
    fn default() -> Self {
        Self::new()
    }
}

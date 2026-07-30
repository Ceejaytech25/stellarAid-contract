use sdk::horizon::client::{HorizonClient, HorizonError};
use crate::models::donation_status::{DonationEvent, DonationStatus};
use tracing::{info, warn, error, Instrument, Span};

#[derive(Debug)]
pub struct VerificationResult {
    pub status: DonationStatus,
    pub ledger: Option<u64>,
    pub created_at: Option<String>,
}

pub async fn cross_check_transaction(
    horizon: &HorizonClient,
    tx_hash: &str,
    current_status: DonationStatus,
) -> Result<VerificationResult, HorizonError> {
    let span = Span::current();
    span.record("tx_hash", tx_hash);
    span.record("current_status", %current_status);

    info!(
        tx_hash = tx_hash,
        current_status = %current_status,
        "starting donation verification"
    );

    let tx = match horizon.get_transaction(tx_hash).await {
        Ok(tx) => {
            info!(
                tx_hash = tx_hash,
                successful = tx.successful,
                ledger = tx.ledger,
                "horizon transaction fetched"
            );
            tx
        }
        Err(e) => {
            error!(
                tx_hash = tx_hash,
                error = %e,
                "failed to fetch transaction from horizon"
            );
            return Err(e);
        }
    };

    let event = if tx.successful {
        DonationEvent::Confirm
    } else {
        DonationEvent::Fail
    };

    let status = match current_status {
        DonationStatus::Submitted => {
            let new = DonationStatus::Confirming
                .transition(event)
                .unwrap_or(DonationStatus::Failed);
            info!(
                tx_hash = tx_hash,
                from = %DonationStatus::Submitted,
                to = %new,
                "donation status transition"
            );
            new
        }
        DonationStatus::Confirming => {
            let new = current_status.transition(event).unwrap_or(DonationStatus::Failed);
            info!(
                tx_hash = tx_hash,
                from = %current_status,
                to = %new,
                "donation status transition"
            );
            new
        }
        other => {
            warn!(
                tx_hash = tx_hash,
                status = %other,
                "unexpected donation status, no transition applied"
            );
            other
        }
    };

    Ok(VerificationResult {
        status,
        ledger: tx.ledger,
        created_at: Some(tx.created_at),
    })
}

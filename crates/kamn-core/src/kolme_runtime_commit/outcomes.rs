//! Runtime-commit receipt, outcome, and notification ownership.

use super::{
    notification_event_to_kolme_provider_receipt_contract, KamnKolmeNotificationEvent,
    KamnKolmeRuntimeProviderOutcome, KolmeCommitReceiptFinality,
};

/// Receipt emitted by the runtime commit client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitReceipt {
    /// Provider identifier.
    pub provider: String,
    /// Deterministic commit identifier.
    pub commit_id: String,
    /// Finality state for the receipt.
    pub finality: KolmeCommitReceiptFinality,
}

/// Typed commit submission result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeRuntimeCommitOutcome {
    /// Request was accepted and submitted.
    Submitted(KolmeRuntimeCommitReceipt),
    /// Request matched an existing idempotency key.
    Duplicate(KolmeRuntimeCommitReceipt),
    /// Request was rejected with an explicit reason.
    Rejected {
        /// Deterministic rejection reason from provider/runtime policy.
        reason: String,
    },
}

/// Provider receipt payload returned by adapter-facing transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitProviderReceipt {
    /// Provider identifier returned by upstream.
    pub provider: String,
    /// Commit identifier returned by upstream.
    pub commit_id: String,
    /// Receipt finality classification returned by upstream.
    pub finality: KolmeCommitReceiptFinality,
}

/// Typed notification event emitted by Kolme `/notifications` websocket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeRuntimeCommitNotificationEvent {
    /// Finalized transaction notification emitted from a new block event.
    NewBlock {
        /// Transaction hash observed in the block payload.
        txhash: String,
        /// Optional block height where the transaction finalized.
        block_height: Option<u64>,
    },
    /// Failed transaction notification emitted by processor execution path.
    FailedTransaction {
        /// Transaction hash observed in failed-transaction payload.
        txhash: String,
        /// Optional proposed block height for the failed transaction.
        proposed_height: Option<u64>,
    },
    /// Latest block watermark notification.
    LatestBlock {
        /// Latest observed block height.
        height: u64,
    },
}

impl From<KamnKolmeNotificationEvent> for KolmeRuntimeCommitNotificationEvent {
    fn from(value: KamnKolmeNotificationEvent) -> Self {
        match value {
            KamnKolmeNotificationEvent::NewBlock {
                txhash,
                block_height,
            } => Self::NewBlock {
                txhash,
                block_height,
            },
            KamnKolmeNotificationEvent::FailedTransaction {
                txhash,
                proposed_height,
            } => Self::FailedTransaction {
                txhash,
                proposed_height,
            },
            KamnKolmeNotificationEvent::LatestBlock { height } => Self::LatestBlock { height },
        }
    }
}

impl KolmeRuntimeCommitNotificationEvent {
    /// Converts notification event to a provider receipt when it carries tx finality information.
    pub fn to_provider_receipt(&self, provider: &str) -> Option<KolmeRuntimeCommitProviderReceipt> {
        let event = match self {
            Self::NewBlock {
                txhash,
                block_height,
            } => KamnKolmeNotificationEvent::NewBlock {
                txhash: txhash.clone(),
                block_height: *block_height,
            },
            Self::FailedTransaction {
                txhash,
                proposed_height,
            } => KamnKolmeNotificationEvent::FailedTransaction {
                txhash: txhash.clone(),
                proposed_height: *proposed_height,
            },
            Self::LatestBlock { height } => {
                KamnKolmeNotificationEvent::LatestBlock { height: *height }
            }
        };
        let receipt = notification_event_to_kolme_provider_receipt_contract(provider, &event)?;
        Some(KolmeRuntimeCommitProviderReceipt {
            provider: receipt.provider,
            commit_id: receipt.commit_id,
            finality: receipt.finality,
        })
    }
}

/// Provider submission outcome used by adapter-backed runtime commit clients.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeRuntimeCommitProviderOutcome {
    /// Provider accepted the submission.
    Submitted(KolmeRuntimeCommitProviderReceipt),
    /// Provider detected duplicate idempotency key.
    Duplicate(KolmeRuntimeCommitProviderReceipt),
    /// Provider rejected the submission with explicit reason.
    Rejected {
        /// Deterministic provider rejection reason.
        reason: String,
    },
}

impl From<KamnKolmeRuntimeProviderOutcome> for KolmeRuntimeCommitProviderOutcome {
    fn from(value: KamnKolmeRuntimeProviderOutcome) -> Self {
        match value {
            KamnKolmeRuntimeProviderOutcome::Submitted {
                provider,
                commit_id,
                finality,
            } => Self::Submitted(KolmeRuntimeCommitProviderReceipt {
                provider,
                commit_id,
                finality,
            }),
            KamnKolmeRuntimeProviderOutcome::Duplicate {
                provider,
                commit_id,
                finality,
            } => Self::Duplicate(KolmeRuntimeCommitProviderReceipt {
                provider,
                commit_id,
                finality,
            }),
            KamnKolmeRuntimeProviderOutcome::Rejected { reason } => Self::Rejected { reason },
        }
    }
}

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiTaskTransitionReceiptRecord {
    pub(crate) receipt_id: String,
    pub(crate) correlation_id: String,
    pub(crate) idempotency_key: String,
    pub(crate) actor_did: String,
    pub(crate) task_id: String,
    pub(crate) transaction_id: String,
    pub(crate) action: String,
    pub(crate) prior_state: String,
    pub(crate) resulting_state: String,
    pub(crate) terms_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) completion_evidence_digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiEscrowTransitionReceiptRecord {
    pub(crate) receipt_id: String,
    pub(crate) correlation_id: String,
    pub(crate) idempotency_key: String,
    pub(crate) actor_did: String,
    pub(crate) escrow_id: String,
    pub(crate) task_id: String,
    pub(crate) transaction_id: String,
    pub(crate) action: String,
    pub(crate) prior_state: String,
    pub(crate) resulting_state: String,
    pub(crate) network: String,
    pub(crate) amount_lamports: u64,
    pub(crate) terms_digest: String,
    pub(crate) release_policy: String,
}

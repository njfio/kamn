use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiPersistedEscrowRecord {
    pub(crate) escrow_id: String,
    pub(crate) state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) transaction_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) funder_did: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) beneficiary_did: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) amount_lamports: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) network: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) terms_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) release_authority_did: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) release_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) fund_idempotency_key: Option<String>,
    #[serde(flatten, default)]
    pub(crate) settlement: ServiceApiSettlementMetadata,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiSettlementIntentRecord {
    pub(crate) settlement_intent_id: String,
    pub(crate) escrow_id: String,
    #[serde(default)]
    pub(crate) task_id: String,
    pub(crate) actor_did: String,
    pub(crate) idempotency_key: String,
    pub(crate) recipient_pubkey: String,
    pub(crate) amount_lamports: u64,
    #[serde(default)]
    pub(crate) asset: String,
    pub(crate) network: String,
    #[serde(default)]
    pub(crate) terms_digest: String,
    pub(crate) expected_signature: String,
    pub(crate) signed_transaction_digest: String,
    pub(crate) signed_transaction_json: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) bridge_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) bridge_receipt_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) bridge_receipt_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) bridge_transaction_signature: Option<String>,
    pub(crate) state: String,
    pub(crate) submission_attempt_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) last_error_code: Option<String>,
}

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiPersistedBridgeRecord {
    pub(crate) bridge_id: String,
    pub(crate) source_message_id: String,
    pub(crate) bridge_status: String,
    pub(crate) target_message_id: String,
    pub(crate) forward_tx_hash: String,
    #[serde(default)]
    pub(crate) target_network: String,
    #[serde(default)]
    pub(crate) payload_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) settlement_authority: Option<ServiceApiBridgeSettlementTermsRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) prepared_transaction: Option<ServiceApiPreparedBridgeTransactionRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) bridge_receipt: Option<ServiceApiBridgeReceiptRecord>,
    #[serde(default)]
    pub(crate) submission_attempt_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) last_error_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiPreparedBridgeTransactionRecord {
    pub(crate) transaction_signature: String,
    pub(crate) signed_transaction_digest: String,
    pub(crate) signed_transaction_json: String,
    pub(crate) transaction_subject: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiBridgeReceiptRecord {
    pub(crate) receipt_id: String,
    pub(crate) receipt_digest: String,
    pub(crate) bridge_id: String,
    pub(crate) source_message_id: String,
    pub(crate) target_network: String,
    pub(crate) payload_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) settlement_authority: Option<ServiceApiBridgeSettlementTermsRecord>,
    pub(crate) transaction_signature: String,
    pub(crate) network: String,
    pub(crate) commitment: String,
    pub(crate) finalized_slot: u64,
    pub(crate) action: String,
    pub(crate) resource_id: String,
    pub(crate) state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiBridgeSettlementTermsRecord {
    pub(crate) escrow_id: String,
    pub(crate) task_id: String,
    pub(crate) actor_did: String,
    pub(crate) recipient_pubkey: String,
    pub(crate) amount_lamports: u64,
    pub(crate) asset: String,
    pub(crate) network: String,
    pub(crate) terms_digest: String,
}

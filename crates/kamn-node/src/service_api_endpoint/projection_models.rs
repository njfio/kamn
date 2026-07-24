use super::*;

pub(crate) const TASK_PROJECTION_SCHEMA_VERSION: &str =
    "kamn.runtime.task-disclosure-projection.v2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiTaskPublicProjection {
    pub(crate) schema_version: String,
    pub(crate) task_id: String,
    pub(crate) transaction_id: String,
    pub(crate) task_state: String,
    pub(crate) escrow_id: String,
    pub(crate) escrow_state: String,
    pub(crate) amount_lamports: u64,
    pub(crate) network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) settlement_tx_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) settlement_commitment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) bridge_receipt_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) bridge_transaction_signature: Option<String>,
    pub(crate) receipt_chain_commitment: String,
    pub(crate) public_commitment: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiParticipantReceiptProjection {
    pub(crate) receipt_id: String,
    pub(crate) receipt_digest: String,
    pub(crate) action: String,
    pub(crate) resource_id: String,
    pub(crate) resulting_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiParticipantTaskProjection {
    pub(crate) view_scope: String,
    pub(crate) participant_role: String,
    #[serde(flatten)]
    pub(crate) public: ServiceApiTaskPublicProjection,
    pub(crate) task_receipt_ids: Vec<String>,
    pub(crate) receipt_chain_receipts: Vec<ServiceApiParticipantReceiptProjection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) completion_evidence_digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiVerifierTaskProjection {
    pub(crate) view_scope: String,
    #[serde(flatten)]
    pub(crate) public: ServiceApiTaskPublicProjection,
}

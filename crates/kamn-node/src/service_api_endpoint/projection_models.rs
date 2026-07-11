use super::*;

pub(crate) const TASK_PROJECTION_SCHEMA_VERSION: &str =
    "kamn.runtime.task-disclosure-projection.v1";

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
    pub(crate) public_commitment: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiParticipantTaskProjection {
    pub(crate) view_scope: String,
    pub(crate) participant_role: String,
    #[serde(flatten)]
    pub(crate) public: ServiceApiTaskPublicProjection,
    pub(crate) task_receipt_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) completion_evidence_digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiVerifierTaskProjection {
    pub(crate) view_scope: String,
    #[serde(flatten)]
    pub(crate) public: ServiceApiTaskPublicProjection,
}

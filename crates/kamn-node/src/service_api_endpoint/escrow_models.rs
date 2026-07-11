use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiEscrowStatusBody {
    pub(crate) escrow_id: String,
    pub(crate) state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) transaction_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) funder_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) beneficiary_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) amount_lamports: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) terms_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) release_authority_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) release_policy: Option<String>,
    pub(crate) claim_scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) receipt_id: Option<String>,
    #[serde(flatten)]
    pub(crate) settlement: ServiceApiSettlementMetadata,
}

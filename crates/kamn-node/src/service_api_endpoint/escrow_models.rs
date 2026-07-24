use super::*;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) receipt_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) settlement_receipt_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) settlement_receipt_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) settlement_receipt_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) settlement_receipt_resource_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) settlement_receipt_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) authoritative_settlement: Option<ServiceApiAuthoritativeSettlement>,
    #[serde(flatten)]
    pub(crate) settlement: ServiceApiSettlementMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiAuthoritativeSettlement {
    pub(crate) bridge_id: String,
    pub(crate) bridge_receipt_id: String,
    pub(crate) bridge_receipt_digest: String,
    pub(crate) settlement_receipt_id: String,
    pub(crate) settlement_receipt_digest: String,
    pub(crate) action: String,
    pub(crate) resource_id: String,
    pub(crate) actor_did: String,
    pub(crate) resulting_state: String,
    pub(crate) task_id: String,
    pub(crate) escrow_id: String,
    pub(crate) recipient: String,
    pub(crate) amount_lamports: u64,
    pub(crate) asset: String,
    pub(crate) network: String,
    pub(crate) transaction_signature: String,
    pub(crate) commitment: String,
    pub(crate) finalized_slot: u64,
    pub(crate) receipt_chain_commitment: String,
    pub(crate) terms_digest: String,
    pub(crate) idempotency_key: String,
}

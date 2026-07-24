use super::*;
use crate::service_api_endpoint::message_store::ServiceApiBridgeReceiptRecord;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiBridgeSubmitBody {
    pub(crate) bridge_id: String,
    pub(crate) source_message_id: String,
    pub(crate) bridge_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiBridgeStatusBody {
    pub(crate) bridge_id: String,
    pub(crate) bridge_status: String,
    pub(crate) target_message_id: String,
    pub(crate) forward_tx_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) receipt_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) receipt_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) transaction_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) finalized_slot: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) bridge_receipt: Option<ServiceApiBridgeReceiptRecord>,
}

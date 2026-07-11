use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiEndpointConfig {
    pub(crate) bind_addr: String,
    pub(crate) max_requests: u64,
    pub(crate) idle_timeout_ms: u64,
    pub(crate) body_limit_bytes: u64,
    pub(crate) concurrency_limit: u64,
    pub(crate) rate_limit_per_second: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiEndpointResponse {
    pub(crate) status_code: u16,
    pub(crate) content_type: &'static str,
    pub(crate) body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiErrorBody {
    pub(crate) error: String,
    pub(crate) reason_code: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiHealthBody {
    pub(crate) status: String,
    pub(crate) runtime_mode: String,
    pub(crate) role: String,
    pub(crate) observability_source: String,
    pub(crate) observability_health: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiMessageCreateBody {
    pub(crate) message_id: String,
    pub(crate) status: String,
    pub(crate) runtime_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiMessageRelayBody {
    pub(crate) message_id: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiMessageGetBody {
    pub(crate) message_id: String,
    pub(crate) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sender_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) recipient_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) body: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiRelaySpoolEntry {
    pub(crate) message_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sender_did: Option<String>,
    pub(crate) recipient_did: String,
    pub(crate) body: String,
    pub(crate) queued_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiChannelCreateBody {
    pub(crate) channel_id: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiChannelMessagesBody {
    pub(crate) channel_id: String,
    pub(crate) messages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub(crate) struct ServiceApiSettlementMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) settlement_receipt_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) settlement_tx_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) settlement_network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) settlement_commitment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiTaskCreateBody {
    pub(crate) task_id: String,
    pub(crate) state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) transaction_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) creator_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) provider_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) terms_digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiTaskGetBody {
    pub(crate) task_id: String,
    pub(crate) state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiTaskTransitionBody {
    pub(crate) task_id: String,
    pub(crate) state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) transaction_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) creator_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) provider_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) terms_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) receipt_id: Option<String>,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiContentRegisterBody {
    pub(crate) content_id: String,
    pub(crate) retention_class: String,
    pub(crate) lifecycle_state: String,
    pub(crate) redaction_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiContentLifecycleBody {
    pub(crate) content_id: String,
    pub(crate) lifecycle_state: String,
    pub(crate) redaction_status: String,
}

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiAgentGetBody {
    pub(crate) did: String,
    pub(crate) reputation_score: u64,
    pub(crate) agent_type: String,
    pub(crate) model_family: String,
    pub(crate) capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiAgentRegisterRequestBody {
    pub(crate) agent_type: String,
    pub(crate) model_family: String,
    pub(crate) capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiAgentSearchRequestBody {
    pub(crate) capability: Option<String>,
    pub(crate) model_family: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiWebsocketStateTransitionBody {
    pub(crate) event: String,
    pub(crate) runtime_mode: String,
    pub(crate) role: String,
    pub(crate) sequence: u64,
}

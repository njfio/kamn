use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiDataLayerRuntimeEvidenceRecord {
    pub(crate) schema_version: String,
    pub(crate) m0_content_hash: String,
    pub(crate) m1_merkle_root: String,
    pub(crate) m2_authorization_reason_code: String,
    pub(crate) m2_audit_record_hash: String,
    pub(crate) m3_blind_index_token: String,
    pub(crate) m3_match_count: usize,
    pub(crate) m4_transition_reason_code: String,
    pub(crate) m5_record_hash: String,
    pub(crate) m6_projection_edge_count: usize,
    pub(crate) m7_observability_health: String,
    pub(crate) m8_retention_due_count: usize,
    pub(crate) m9_dispatch_ack_status: String,
    pub(crate) m9_dispatch_reason_code: String,
    pub(crate) m10_archived_partition_count: usize,
    pub(crate) m11_decision: String,
    pub(crate) m11_reason_codes_csv: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiPersistedMessageRecord {
    pub(crate) message_id: String,
    pub(crate) status: String,
    pub(crate) channel_id: Option<String>,
    #[serde(default)]
    pub(crate) sender_did: Option<String>,
    #[serde(default)]
    pub(crate) recipient_did: Option<String>,
    #[serde(default)]
    pub(crate) body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) data_layer_runtime_evidence: Option<ServiceApiDataLayerRuntimeEvidenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiPersistedTaskRecord {
    pub(crate) task_id: String,
    pub(crate) state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) creator_did: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) task_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) assignee: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiPersistedEscrowRecord {
    pub(crate) escrow_id: String,
    pub(crate) state: String,
    #[serde(flatten, default)]
    pub(crate) settlement: ServiceApiSettlementMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiPersistedContentRecord {
    pub(crate) content_id: String,
    pub(crate) retention_class: String,
    pub(crate) lifecycle_state: String,
    pub(crate) redaction_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiPersistedBridgeRecord {
    pub(crate) bridge_id: String,
    pub(crate) source_message_id: String,
    pub(crate) bridge_status: String,
    pub(crate) target_message_id: String,
    pub(crate) forward_tx_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiPersistedAgentRecord {
    pub(crate) did: String,
    pub(crate) reputation_score: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) balance: Option<u64>,
    #[serde(default)]
    pub(crate) registered: bool,
    #[serde(default)]
    pub(crate) agent_type: String,
    #[serde(default)]
    pub(crate) model_family: String,
    #[serde(default)]
    pub(crate) capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiPersistedAgentGrantRecord {
    pub(crate) did: String,
    pub(crate) resource: String,
    pub(crate) role: String,
    pub(crate) action: String,
    pub(crate) status: String,
    pub(crate) idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiAuthorizationReceiptRecord {
    pub(crate) receipt_id: String,
    pub(crate) correlation_id: String,
    pub(crate) actor_did: String,
    pub(crate) resource: String,
    pub(crate) action: String,
    pub(crate) role: String,
    pub(crate) decision: String,
    pub(crate) reason_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ServiceApiAgentBalanceBody {
    pub(crate) did: String,
    pub(crate) balance: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ServiceApiAgentRegistrationStoreError {
    Conflict(String),
    Persistence(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiPersistedMessageStoreSnapshot {
    pub(crate) schema_version: String,
    pub(crate) messages: BTreeMap<String, ServiceApiPersistedMessageRecord>,
    pub(crate) channel_messages: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub(crate) auth_nonce_high_watermarks: BTreeMap<String, u64>,
    #[serde(default)]
    pub(crate) tasks: BTreeMap<String, ServiceApiPersistedTaskRecord>,
    #[serde(default)]
    pub(crate) escrows: BTreeMap<String, ServiceApiPersistedEscrowRecord>,
    #[serde(default)]
    pub(crate) contents: BTreeMap<String, ServiceApiPersistedContentRecord>,
    #[serde(default)]
    pub(crate) bridges: BTreeMap<String, ServiceApiPersistedBridgeRecord>,
    #[serde(default)]
    pub(crate) agents: BTreeMap<String, ServiceApiPersistedAgentRecord>,
    #[serde(default)]
    pub(crate) agent_grants: BTreeMap<String, ServiceApiPersistedAgentGrantRecord>,
    #[serde(default)]
    pub(crate) authorization_receipts: Vec<ServiceApiAuthorizationReceiptRecord>,
}

impl Default for ServiceApiPersistedMessageStoreSnapshot {
    fn default() -> Self {
        Self {
            schema_version: "kamn.runtime.service-api-message-store.v3".to_owned(),
            messages: BTreeMap::new(),
            channel_messages: BTreeMap::new(),
            auth_nonce_high_watermarks: BTreeMap::new(),
            tasks: BTreeMap::new(),
            escrows: BTreeMap::new(),
            contents: BTreeMap::new(),
            bridges: BTreeMap::new(),
            agents: BTreeMap::new(),
            agent_grants: BTreeMap::new(),
            authorization_receipts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiMessageStore {
    pub(crate) state_file: Option<String>,
    pub(crate) audit_export_file: Option<String>,
    pub(crate) snapshot: ServiceApiPersistedMessageStoreSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ServiceApiRelayProgressCounts {
    pub(crate) created_message_count: u64,
    pub(crate) relayed_message_count: u64,
    pub(crate) delivered_message_count: u64,
}

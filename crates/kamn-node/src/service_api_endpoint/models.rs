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
pub(crate) struct ServiceApiSnapshot {
    pub(crate) runtime_mode: String,
    pub(crate) role: String,
    pub(crate) chain_id: String,
    pub(crate) chain_version: String,
    pub(crate) cross_store_replay_reason_taxonomy_version: String,
    pub(crate) cross_store_replay_reason_code_count: usize,
    pub(crate) auth_reason_taxonomy_version: String,
    pub(crate) auth_reason_code_count: usize,
    pub(crate) scope_policy_reason_taxonomy_version: String,
    pub(crate) scope_policy_reason_code_count: usize,
    pub(crate) scope_policy_fixture_reason_taxonomy_version: String,
    pub(crate) scope_policy_fixture_reason_code_count: usize,
    pub(crate) scope_policy_fixture_row_count: usize,
    pub(crate) scope_policy_fixture_allow_row_count: usize,
    pub(crate) scope_policy_fixture_deny_row_count: usize,
    pub(crate) scope_policy_fixture_unique_route_count: usize,
    pub(crate) scope_policy_fixture_unique_scope_count: usize,
    pub(crate) scope_policy_fixture_unique_method_count: usize,
    pub(crate) scope_policy_fixture_unique_expected_outcome_count: usize,
    pub(crate) scope_policy_fixture_unique_allow_route_count: usize,
    pub(crate) scope_policy_fixture_unique_deny_route_count: usize,
    pub(crate) scope_policy_fixture_unique_allow_deny_overlap_route_count: usize,
    pub(crate) scope_policy_fixture_unique_allow_only_route_count: usize,
    pub(crate) scope_policy_fixture_unique_deny_only_route_count: usize,
    pub(crate) scope_policy_fixture_unique_allow_scope_count: usize,
    pub(crate) scope_policy_fixture_unique_deny_scope_count: usize,
    pub(crate) scope_policy_fixture_unique_allow_deny_overlap_scope_count: usize,
    pub(crate) scope_policy_fixture_unique_allow_only_scope_count: usize,
    pub(crate) scope_policy_fixture_unique_deny_only_scope_count: usize,
    pub(crate) scope_policy_fixture_unique_allow_deny_overlap_method_count: usize,
    pub(crate) scope_policy_fixture_unique_allow_only_method_count: usize,
    pub(crate) scope_policy_fixture_unique_deny_only_method_count: usize,
    pub(crate) lifecycle_rejection_reason_taxonomy_version: String,
    pub(crate) lifecycle_rejection_reason_code_count: usize,
    pub(crate) route_authz_matrix_schema_version: String,
    pub(crate) route_authz_matrix_total_route_count: usize,
    pub(crate) route_authz_matrix_public_route_count: usize,
    pub(crate) route_authz_matrix_protected_route_count: usize,
    pub(crate) websocket_reason_taxonomy_version: String,
    pub(crate) websocket_reason_code_count: usize,
    pub(crate) observability_source: String,
    pub(crate) observability_latency_p50_ms: u64,
    pub(crate) observability_latency_p99_ms: u64,
    pub(crate) observability_throughput_tps: u64,
    pub(crate) observability_error_rate_bps: u64,
    pub(crate) observability_availability_bps: u64,
    pub(crate) observability_health: String,
    pub(crate) observability_alert_count: usize,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiTaskCreateBody {
    pub(crate) task_id: String,
    pub(crate) state: String,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiEscrowStatusBody {
    pub(crate) escrow_id: String,
    pub(crate) state: String,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiWebsocketStateTransitionBody {
    pub(crate) event: String,
    pub(crate) runtime_mode: String,
    pub(crate) role: String,
    pub(crate) sequence: u64,
}

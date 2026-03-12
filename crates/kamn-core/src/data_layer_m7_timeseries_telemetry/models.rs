use crate::ObservabilitySnapshot;

/// Hourly bucket size in seconds.
pub const DATA_LAYER_M7_HOURLY_BUCKET_SECONDS: u64 = 3_600;
/// Daily bucket size in seconds.
pub const DATA_LAYER_M7_DAILY_BUCKET_SECONDS: u64 = 86_400;
/// Stable reason marker for successful aggregate results.
pub const DATA_LAYER_M7_AGGREGATE_REASON_CODE: &str = "m7_timeseries_aggregate_computed";
/// Stable reason marker for owner-scope authorization failures.
pub const DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE: &str = "m7_timeseries_owner_scope_denied";
/// Stable reason marker for successful billing reconciliation match.
pub const DATA_LAYER_M7_BILLING_RECONCILIATION_MATCH_REASON_CODE: &str =
    "m7_billing_reconciliation_match";
/// Stable reason marker for billing reconciliation mismatch.
pub const DATA_LAYER_M7_BILLING_RECONCILIATION_MISMATCH_REASON_CODE: &str =
    "m7_billing_reconciliation_mismatch";
/// Stable reason marker for invalid projected observability samples.
pub const DATA_LAYER_M7_OBSERVABILITY_SAMPLE_INVALID_REASON_CODE: &str =
    "m7_observability_sample_invalid";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7TelemetryPointInput {
    pub owner_did: String,
    pub agent_did: String,
    pub timestamp_epoch_seconds: u64,
    pub message_count: u64,
    pub bytes_stored: u64,
    pub query_count: u64,
    pub embedding_count: u64,
    pub embedding_anomaly_count: u64,
    pub ingress_latency_ms_p95: u32,
    pub egress_latency_ms_p95: u32,
    pub active_sessions: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7TelemetryPointRecord {
    pub owner_did: String,
    pub agent_did: String,
    pub timestamp_epoch_seconds: u64,
    pub bucket_hour_epoch_seconds: u64,
    pub bucket_day_epoch_seconds: u64,
    pub message_count: u64,
    pub bytes_stored: u64,
    pub query_count: u64,
    pub embedding_count: u64,
    pub embedding_anomaly_count: u64,
    pub ingress_latency_ms_p95: u32,
    pub egress_latency_ms_p95: u32,
    pub active_sessions: u32,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7TelemetryScopeQuery {
    pub requester_owner_did: String,
    pub owner_did: String,
    pub agent_did: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7BillingQuery {
    pub requester_owner_did: String,
    pub owner_did: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7BillingReconciliationInput {
    pub requester_owner_did: String,
    pub owner_did: String,
    pub bucket_day_epoch_seconds: u64,
    pub messages_stored_total: u64,
    pub bytes_stored_total: u64,
    pub queries_executed_total: u64,
    pub embeddings_generated_total: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM7BillingReconciliationDecision {
    Match,
    Mismatch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM7OwnerObservabilityReport {
    pub owner_did: String,
    pub reports: Vec<crate::ObservabilityReport>,
    pub snapshot: ObservabilitySnapshot,
}

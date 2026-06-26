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
/// Public contract model for Data Layer M7 Telemetry Point Input.
pub struct DataLayerM7TelemetryPointInput {
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Agent did carried by this public contract model.
    pub agent_did: String,
    /// Timestamp epoch seconds carried by this public contract model.
    pub timestamp_epoch_seconds: u64,
    /// Message count carried by this public contract model.
    pub message_count: u64,
    /// Bytes stored carried by this public contract model.
    pub bytes_stored: u64,
    /// Query count carried by this public contract model.
    pub query_count: u64,
    /// Embedding count carried by this public contract model.
    pub embedding_count: u64,
    /// Embedding anomaly count carried by this public contract model.
    pub embedding_anomaly_count: u64,
    /// Ingress latency ms p95 carried by this public contract model.
    pub ingress_latency_ms_p95: u32,
    /// Egress latency ms p95 carried by this public contract model.
    pub egress_latency_ms_p95: u32,
    /// Active sessions carried by this public contract model.
    pub active_sessions: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer M7 Telemetry Point Record.
pub struct DataLayerM7TelemetryPointRecord {
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Agent did carried by this public contract model.
    pub agent_did: String,
    /// Timestamp epoch seconds carried by this public contract model.
    pub timestamp_epoch_seconds: u64,
    /// Bucket hour epoch seconds carried by this public contract model.
    pub bucket_hour_epoch_seconds: u64,
    /// Bucket day epoch seconds carried by this public contract model.
    pub bucket_day_epoch_seconds: u64,
    /// Message count carried by this public contract model.
    pub message_count: u64,
    /// Bytes stored carried by this public contract model.
    pub bytes_stored: u64,
    /// Query count carried by this public contract model.
    pub query_count: u64,
    /// Embedding count carried by this public contract model.
    pub embedding_count: u64,
    /// Embedding anomaly count carried by this public contract model.
    pub embedding_anomaly_count: u64,
    /// Ingress latency ms p95 carried by this public contract model.
    pub ingress_latency_ms_p95: u32,
    /// Egress latency ms p95 carried by this public contract model.
    pub egress_latency_ms_p95: u32,
    /// Active sessions carried by this public contract model.
    pub active_sessions: u32,
    /// Sequence carried by this public contract model.
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer M7 Telemetry Scope Query.
pub struct DataLayerM7TelemetryScopeQuery {
    /// Requester owner did carried by this public contract model.
    pub requester_owner_did: String,
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Agent did carried by this public contract model.
    pub agent_did: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer M7 Billing Query.
pub struct DataLayerM7BillingQuery {
    /// Requester owner did carried by this public contract model.
    pub requester_owner_did: String,
    /// Owner did carried by this public contract model.
    pub owner_did: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer M7 Billing Reconciliation Input.
pub struct DataLayerM7BillingReconciliationInput {
    /// Requester owner did carried by this public contract model.
    pub requester_owner_did: String,
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Bucket day epoch seconds carried by this public contract model.
    pub bucket_day_epoch_seconds: u64,
    /// Messages stored total carried by this public contract model.
    pub messages_stored_total: u64,
    /// Bytes stored total carried by this public contract model.
    pub bytes_stored_total: u64,
    /// Queries executed total carried by this public contract model.
    pub queries_executed_total: u64,
    /// Embeddings generated total carried by this public contract model.
    pub embeddings_generated_total: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Public contract enum for Data Layer M7 Billing Reconciliation Decision.
pub enum DataLayerM7BillingReconciliationDecision {
    /// Match variant for this public contract enum.
    Match,
    /// Mismatch variant for this public contract enum.
    Mismatch,
}

#[derive(Debug, Clone, PartialEq)]
/// Public contract model for Data Layer M7 Owner Observability Report.
pub struct DataLayerM7OwnerObservabilityReport {
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Reports carried by this public contract model.
    pub reports: Vec<crate::ObservabilityReport>,
    /// Snapshot carried by this public contract model.
    pub snapshot: ObservabilitySnapshot,
}

/// Deterministic billing reconciliation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7BillingReconciliationReport {
    /// Owner DID scope.
    pub owner_did: String,
    /// Billing day bucket start.
    pub bucket_day_epoch_seconds: u64,
    /// Reconciliation decision.
    pub decision: super::DataLayerM7BillingReconciliationDecision,
    /// Stable decision reason marker.
    pub reason_code: &'static str,
    /// Projected metric: messages stored.
    pub projected_messages_stored_total: u64,
    /// Projected metric: bytes stored.
    pub projected_bytes_stored_total: u64,
    /// Projected metric: queries executed.
    pub projected_queries_executed_total: u64,
    /// Projected metric: embeddings generated.
    pub projected_embeddings_generated_total: u64,
    /// Statement metric: messages stored.
    pub statement_messages_stored_total: u64,
    /// Statement metric: bytes stored.
    pub statement_bytes_stored_total: u64,
    /// Statement metric: queries executed.
    pub statement_queries_executed_total: u64,
    /// Statement metric: embeddings generated.
    pub statement_embeddings_generated_total: u64,
}

/// Hourly aggregate row for one owner+agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7AgentHourlyAggregate {
    pub owner_did: String,
    pub agent_did: String,
    pub bucket_hour_epoch_seconds: u64,
    pub message_count_total: u64,
    pub bytes_stored_total: u64,
    pub query_count_total: u64,
    pub embedding_count_total: u64,
    pub embedding_anomaly_count_total: u64,
    pub reason_code: &'static str,
}

/// Daily aggregate row for one owner+agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7AgentDailyAggregate {
    pub owner_did: String,
    pub agent_did: String,
    pub bucket_day_epoch_seconds: u64,
    pub message_count_total: u64,
    pub bytes_stored_total: u64,
    pub query_count_total: u64,
    pub embedding_count_total: u64,
    pub embedding_anomaly_count_total: u64,
    pub reason_code: &'static str,
}

/// Hourly network summary aggregate row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7NetworkHourlyAggregate {
    pub bucket_hour_epoch_seconds: u64,
    pub message_count_total: u64,
    pub bytes_stored_total: u64,
    pub query_count_total: u64,
    pub embedding_count_total: u64,
    pub embedding_anomaly_count_total: u64,
    pub active_agent_count: u64,
    pub reason_code: &'static str,
}

/// Owner billing daily projection row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7OwnerBillingDailyProjection {
    pub owner_did: String,
    pub bucket_day_epoch_seconds: u64,
    pub messages_stored_total: u64,
    pub bytes_stored_total: u64,
    pub queries_executed_total: u64,
    pub embeddings_generated_total: u64,
    pub reason_code: &'static str,
}

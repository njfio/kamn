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
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Agent did carried by this public contract model.
    pub agent_did: String,
    /// Bucket hour epoch seconds carried by this public contract model.
    pub bucket_hour_epoch_seconds: u64,
    /// Message count total carried by this public contract model.
    pub message_count_total: u64,
    /// Bytes stored total carried by this public contract model.
    pub bytes_stored_total: u64,
    /// Query count total carried by this public contract model.
    pub query_count_total: u64,
    /// Embedding count total carried by this public contract model.
    pub embedding_count_total: u64,
    /// Embedding anomaly count total carried by this public contract model.
    pub embedding_anomaly_count_total: u64,
    /// Reason code carried by this public contract model.
    pub reason_code: &'static str,
}

/// Daily aggregate row for one owner+agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7AgentDailyAggregate {
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Agent did carried by this public contract model.
    pub agent_did: String,
    /// Bucket day epoch seconds carried by this public contract model.
    pub bucket_day_epoch_seconds: u64,
    /// Message count total carried by this public contract model.
    pub message_count_total: u64,
    /// Bytes stored total carried by this public contract model.
    pub bytes_stored_total: u64,
    /// Query count total carried by this public contract model.
    pub query_count_total: u64,
    /// Embedding count total carried by this public contract model.
    pub embedding_count_total: u64,
    /// Embedding anomaly count total carried by this public contract model.
    pub embedding_anomaly_count_total: u64,
    /// Reason code carried by this public contract model.
    pub reason_code: &'static str,
}

/// Hourly network summary aggregate row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7NetworkHourlyAggregate {
    /// Bucket hour epoch seconds carried by this public contract model.
    pub bucket_hour_epoch_seconds: u64,
    /// Message count total carried by this public contract model.
    pub message_count_total: u64,
    /// Bytes stored total carried by this public contract model.
    pub bytes_stored_total: u64,
    /// Query count total carried by this public contract model.
    pub query_count_total: u64,
    /// Embedding count total carried by this public contract model.
    pub embedding_count_total: u64,
    /// Embedding anomaly count total carried by this public contract model.
    pub embedding_anomaly_count_total: u64,
    /// Active agent count carried by this public contract model.
    pub active_agent_count: u64,
    /// Reason code carried by this public contract model.
    pub reason_code: &'static str,
}

/// Owner billing daily projection row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7OwnerBillingDailyProjection {
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
    /// Reason code carried by this public contract model.
    pub reason_code: &'static str,
}

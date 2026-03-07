/// Stable reason marker for successful billing reconciliation match.
pub const DATA_LAYER_M7_BILLING_RECONCILIATION_MATCH_REASON_CODE: &str =
    "m7_billing_reconciliation_match";
/// Stable reason marker for billing reconciliation mismatch.
pub const DATA_LAYER_M7_BILLING_RECONCILIATION_MISMATCH_REASON_CODE: &str =
    "m7_billing_reconciliation_mismatch";

/// Billing projection sample derived from one M7 telemetry point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM7BillingProjectionSampleInput {
    /// Daily bucket start (epoch seconds).
    pub bucket_day_epoch_seconds: u64,
    /// Message count for this sample.
    pub message_count: u64,
    /// Bytes stored for this sample.
    pub bytes_stored: u64,
    /// Query count for this sample.
    pub query_count: u64,
    /// Embedding generation count for this sample.
    pub embedding_count: u64,
}

/// Owner daily billing projection row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7BillingDailyProjection {
    /// Owner DID scope.
    pub owner_did: String,
    /// Daily bucket start (epoch seconds).
    pub bucket_day_epoch_seconds: u64,
    /// Total messages in bucket.
    pub messages_stored_total: u64,
    /// Total bytes in bucket.
    pub bytes_stored_total: u64,
    /// Total queries in bucket.
    pub queries_executed_total: u64,
    /// Total embeddings in bucket.
    pub embeddings_generated_total: u64,
}

/// Billing reconciliation decision output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM7BillingReconciliationDecision {
    /// Statement equals projected owner daily totals.
    Match,
    /// Statement differs from projected owner daily totals.
    Mismatch,
}

/// Owner daily billing statement input used for reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7BillingReconciliationInput {
    /// Owner DID scope.
    pub owner_did: String,
    /// Billing day bucket start (epoch seconds, aligned to day boundary).
    pub bucket_day_epoch_seconds: u64,
    /// Statement metric: messages stored.
    pub messages_stored_total: u64,
    /// Statement metric: bytes stored.
    pub bytes_stored_total: u64,
    /// Statement metric: queries executed.
    pub queries_executed_total: u64,
    /// Statement metric: embeddings generated.
    pub embeddings_generated_total: u64,
}

/// Deterministic billing reconciliation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7BillingReconciliationReport {
    /// Owner DID scope.
    pub owner_did: String,
    /// Billing day bucket start.
    pub bucket_day_epoch_seconds: u64,
    /// Reconciliation decision.
    pub decision: DataLayerM7BillingReconciliationDecision,
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

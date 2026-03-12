/// Hash algorithm label used by M5 deterministic record digests.
pub const DATA_LAYER_M5_HASH_ALGORITHM: &str = "sha256";
/// Genesis marker used by owner-scoped embedding hash chains.
pub const DATA_LAYER_M5_EMBEDDING_HASH_CHAIN_GENESIS: &str = "GENESIS";
/// Distance metric label used by semantic and anomaly contracts.
pub const DATA_LAYER_M5_VECTOR_DISTANCE_METRIC_COSINE: &str = "cosine";
/// Owner-side privacy mode rejects plaintext vector storage.
pub const DATA_LAYER_M5_OWNER_SIDE_PLAINTEXT_STORAGE_NOT_ALLOWED_REASON_CODE: &str =
    "m5_vector_owner_side_plaintext_storage_not_allowed";
/// Server-side plaintext mode requires plaintext vector storage.
pub const DATA_LAYER_M5_SERVER_SIDE_PLAINTEXT_REQUIRED_REASON_CODE: &str =
    "m5_vector_server_side_plaintext_required";
/// Owner-side encrypted mode requires local semantic query execution.
pub const DATA_LAYER_M5_OWNER_SIDE_QUERY_REQUIRES_LOCAL_INDEX_REASON_CODE: &str =
    "m5_vector_owner_side_query_requires_local_index";
/// Semantic query requires a plaintext index for owner scope.
pub const DATA_LAYER_M5_PLAINTEXT_INDEX_MISSING_FOR_OWNER_SCOPE_REASON_CODE: &str =
    "m5_vector_plaintext_index_missing_for_owner_scope";
/// Owner-side encrypted mode requires local anomaly pipeline execution.
pub const DATA_LAYER_M5_OWNER_SIDE_ANOMALY_REQUIRES_LOCAL_PIPELINE_REASON_CODE: &str =
    "m5_vector_owner_side_anomaly_requires_local_pipeline";
/// Candidate exceeded anomaly threshold.
pub const DATA_LAYER_M5_ANOMALY_THRESHOLD_EXCEEDED_REASON_CODE: &str =
    "m5_vector_anomaly_threshold_exceeded";
/// Candidate remained within anomaly threshold.
pub const DATA_LAYER_M5_ANOMALY_WITHIN_THRESHOLD_REASON_CODE: &str =
    "m5_vector_anomaly_within_threshold";
/// Recall drift remained within configured guardrails.
pub const DATA_LAYER_M5_RECALL_DRIFT_STABLE_REASON_CODE: &str = "m5_vector_recall_drift_stable";
/// Recall drift degraded beyond configured guardrails.
pub const DATA_LAYER_M5_RECALL_DRIFT_DEGRADED_REASON_CODE: &str =
    "m5_vector_recall_drift_degraded";
/// Agent DID failed canonical parser validation.
pub const DATA_LAYER_M5_INVALID_AGENT_DID_REASON_CODE: &str = "m5_vector_invalid_agent_did";
/// Retention due projection reason marker.
pub const DATA_LAYER_M5_RETENTION_DUE_REASON_CODE: &str = "m5_vector_retention_due";

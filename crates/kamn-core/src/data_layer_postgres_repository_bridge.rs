//! PostgreSQL repository bridge contracts for data-layer persistence wiring.
//!
//! This module does not execute SQL. It projects validated data-layer inputs
//! into deterministic SQL operation descriptors that runtime adapters can
//! execute later.

use std::fmt;

use crate::{
    data_layer_m2_default_rls_policies, AgentDid, DataLayerM0EnvelopeRecord,
    DataLayerM5EmbeddingRecord, DataLayerM5SemanticQuery, DataLayerM6GraphEdgeRecord,
    DataLayerM6GraphEdgeRelation, DataLayerM6TrustPropagationQuery, DataLayerM7BillingQuery,
    DataLayerM7TelemetryPointRecord, KamnDid, DATA_LAYER_M2_REQUESTER_DID_SETTING,
    DATA_LAYER_M7_DAILY_BUCKET_SECONDS, DATA_LAYER_M7_HOURLY_BUCKET_SECONDS,
};

/// Stable reason marker for invalid requester DID session inputs.
pub const DATA_LAYER_PG_INVALID_REQUESTER_DID_REASON_CODE: &str =
    "data_layer_pg_invalid_requester_did";
/// Stable reason marker for invalid owner DID inputs.
pub const DATA_LAYER_PG_INVALID_OWNER_DID_REASON_CODE: &str = "data_layer_pg_invalid_owner_did";
/// Stable reason marker for pgvector extension unavailability.
pub const DATA_LAYER_PG_PGVECTOR_EXTENSION_UNAVAILABLE_REASON_CODE: &str =
    "data_layer_pg_pgvector_extension_unavailable";
/// Stable reason marker for pgvector dimension mismatch.
pub const DATA_LAYER_PG_PGVECTOR_DIMENSION_MISMATCH_REASON_CODE: &str =
    "data_layer_pg_pgvector_dimension_mismatch";
/// Stable reason marker for AGE extension unavailability.
pub const DATA_LAYER_PG_AGE_EXTENSION_UNAVAILABLE_REASON_CODE: &str =
    "data_layer_pg_age_extension_unavailable";
/// Stable reason marker for unsupported AGE relation projection.
pub const DATA_LAYER_PG_AGE_RELATION_UNSUPPORTED_REASON_CODE: &str =
    "data_layer_pg_age_relation_unsupported";
/// Stable reason marker for Timescale extension unavailability.
pub const DATA_LAYER_PG_TIMESCALE_EXTENSION_UNAVAILABLE_REASON_CODE: &str =
    "data_layer_pg_timescale_extension_unavailable";
/// Stable reason marker for invalid Timescale bucket window inputs.
pub const DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE: &str =
    "data_layer_pg_timescale_invalid_bucket_window";

const DATA_LAYER_PG_MAX_BLIND_INDEX_SEARCH_LIMIT: u32 = 200;
const DATA_LAYER_PG_MAX_VECTOR_SEARCH_LIMIT: usize = 200;
const DATA_LAYER_PG_MAX_AGE_QUERY_LIMIT: usize = 200;
const DATA_LAYER_PG_MAX_TIMESCALE_QUERY_LIMIT: usize = 200;

/// Deterministic operation kind projected by the bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerPgOperationKind {
    /// Insert one message row descriptor.
    InsertMessage,
    /// Select one message by id descriptor.
    SelectMessageById,
    /// Search message rows via blind-index descriptor.
    SearchMessagesByBlindIndex,
    /// Insert one merkle batch row descriptor.
    InsertMerkleBatch,
    /// Assign one message row to merkle batch descriptor.
    AssignMessageMerkleBatch,
    /// Update merkle batch row to submitted status descriptor.
    MarkMerkleBatchSubmitted,
    /// Update merkle batch row to confirmed status descriptor.
    MarkMerkleBatchConfirmed,
    /// Insert one M5 embedding row for pgvector-backed search.
    InsertEmbeddingVector,
    /// Search M5 embedding rows using pgvector distance ordering.
    SearchEmbeddingVectors,
    /// Upsert one M6 graph edge via AGE/openCypher descriptor.
    UpsertGraphEdge,
    /// Query M6 trust-propagation rows via AGE/openCypher descriptor.
    QueryGraphTrustPropagation,
    /// Insert one M7 telemetry row for Timescale-backed ingest paths.
    InsertTelemetryPoint,
    /// Query one owner-scoped telemetry rollup via Timescale query paths.
    QueryTelemetryOwnerRollup,
}

/// Requester session metadata projected into SQL execution context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgRequesterSession {
    /// Session setting key used by RLS policy templates.
    pub setting_key: &'static str,
    /// Validated requester DID value.
    pub requester_did: String,
}

/// Deterministic SQL operation descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgSqlOperation {
    /// Operation kind.
    pub kind: DataLayerPgOperationKind,
    /// SQL statement text.
    pub sql: String,
    /// Stable bind-order markers.
    pub bind_markers: Vec<&'static str>,
    /// RLS requester session metadata.
    pub session: DataLayerPgRequesterSession,
}

/// Blind-index search request projected into SQL operation descriptors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgBlindIndexSearchRequest {
    /// Requester DID for RLS session context.
    pub requester_did: String,
    /// Owner DID used for owner-scope filtering.
    pub owner_did: String,
    /// Blind-index key.
    pub index_key: String,
    /// Blind-index token/hash value.
    pub index_value_hash: String,
    /// Maximum number of rows to return.
    pub limit: u32,
}

/// Deterministic pgvector capability configuration for M5 bridge projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerPgM5PgvectorConfig {
    /// Whether pgvector extension surface is available.
    pub extension_enabled: bool,
    /// Configured vector dimensionality for embeddings and queries.
    pub dimensions: usize,
}

impl DataLayerPgM5PgvectorConfig {
    /// Creates deterministic pgvector configuration.
    pub fn new(
        extension_enabled: bool,
        dimensions: usize,
    ) -> Result<Self, DataLayerPgRepositoryBridgeError> {
        if dimensions == 0 {
            return Err(DataLayerPgRepositoryBridgeError::EmptyField(
                "pgvector_dimensions",
            ));
        }
        Ok(Self {
            extension_enabled,
            dimensions,
        })
    }
}

/// M5 semantic query request projected into pgvector SQL operation descriptors.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerPgM5SimilaritySearchRequest {
    /// Requester DID for RLS session context.
    pub requester_did: String,
    /// Semantic query projected from M5 contract inputs.
    pub query: DataLayerM5SemanticQuery,
}

/// Deterministic AGE capability configuration for M6 bridge projections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgM6AgeConfig {
    /// Whether AGE extension surface is available.
    pub extension_enabled: bool,
    /// Graph namespace marker.
    pub graph_name: String,
}

impl DataLayerPgM6AgeConfig {
    /// Creates deterministic AGE configuration.
    pub fn new(
        extension_enabled: bool,
        graph_name: impl Into<String>,
    ) -> Result<Self, DataLayerPgRepositoryBridgeError> {
        let graph_name = graph_name.into();
        if graph_name.trim().is_empty() {
            return Err(DataLayerPgRepositoryBridgeError::EmptyField(
                "age_graph_name",
            ));
        }
        Ok(Self {
            extension_enabled,
            graph_name,
        })
    }
}

/// M6 trust-propagation query request projected into AGE SQL descriptors.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerPgM6AgeTrustQueryRequest {
    /// Requester DID for RLS/session scope projection.
    pub requester_did: String,
    /// Trust propagation query projected from M6 contract input.
    pub query: DataLayerM6TrustPropagationQuery,
}

/// Deterministic Timescale capability configuration for M7 bridge projections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgM7TimescaleConfig {
    /// Whether Timescale extension surface is available.
    pub extension_enabled: bool,
    /// Telemetry hypertable name.
    pub hypertable_name: String,
}

impl DataLayerPgM7TimescaleConfig {
    /// Creates deterministic Timescale configuration.
    pub fn new(
        extension_enabled: bool,
        hypertable_name: impl Into<String>,
    ) -> Result<Self, DataLayerPgRepositoryBridgeError> {
        let hypertable_name = hypertable_name.into();
        if hypertable_name.trim().is_empty() {
            return Err(DataLayerPgRepositoryBridgeError::EmptyField(
                "timescale_hypertable_name",
            ));
        }
        Ok(Self {
            extension_enabled,
            hypertable_name,
        })
    }
}

/// M7 owner rollup query request projected into Timescale SQL descriptors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgM7TimescaleOwnerRollupRequest {
    /// Requester DID for RLS/session scope projection.
    pub requester_did: String,
    /// Owner-scoped billing/rollup query from M7 contracts.
    pub query: DataLayerM7BillingQuery,
    /// Rollup bucket window in seconds.
    pub bucket_window_seconds: u64,
    /// Optional max rows to return.
    pub limit: Option<usize>,
}

/// RLS SQL statement descriptor projected from M2 templates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgRlsStatement {
    /// Target table name.
    pub table_name: String,
    /// Policy name tied to this statement.
    pub policy_name: String,
    /// SQL statement payload.
    pub sql: String,
}

/// Error taxonomy for PostgreSQL repository bridge projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerPgRepositoryBridgeError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// Requester DID failed validation.
    InvalidRequesterDid {
        /// Invalid input field name.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Parser detail.
        detail: String,
    },
    /// Owner DID failed validation.
    InvalidOwnerDid {
        /// Invalid input field name.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Parser detail.
        detail: String,
    },
    /// Search limit is outside accepted bounds.
    InvalidSearchLimit {
        /// Requested limit.
        requested: u32,
        /// Maximum accepted limit.
        max_allowed: u32,
    },
    /// pgvector extension is unavailable for requested projection.
    PgvectorExtensionUnavailable {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Vector dimensionality does not match pgvector configuration.
    PgvectorDimensionMismatch {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Expected configured dimensionality.
        expected: usize,
        /// Found input dimensionality.
        found: usize,
    },
    /// AGE extension is unavailable for requested projection.
    AgeExtensionUnavailable {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// AGE projection does not support the requested relation.
    AgeUnsupportedRelation {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Relation marker that failed validation.
        relation_marker: &'static str,
    },
    /// Timescale extension is unavailable for requested projection.
    TimescaleExtensionUnavailable {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Timescale bucket window input is invalid.
    InvalidTimescaleBucketWindow {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Invalid bucket window in seconds.
        bucket_window_seconds: u64,
    },
}

impl fmt::Display for DataLayerPgRepositoryBridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(formatter, "{field} must not be empty"),
            Self::InvalidRequesterDid {
                field,
                reason_code,
                detail,
            } => write!(
                formatter,
                "invalid requester did field {field}: {reason_code} ({detail})"
            ),
            Self::InvalidOwnerDid {
                field,
                reason_code,
                detail,
            } => write!(
                formatter,
                "invalid owner did field {field}: {reason_code} ({detail})"
            ),
            Self::InvalidSearchLimit {
                requested,
                max_allowed,
            } => write!(
                formatter,
                "invalid blind-index search limit: requested {requested}, max {max_allowed}"
            ),
            Self::PgvectorExtensionUnavailable { reason_code } => {
                write!(formatter, "pgvector extension unavailable: {reason_code}")
            }
            Self::PgvectorDimensionMismatch {
                reason_code,
                expected,
                found,
            } => write!(
                formatter,
                "pgvector dimension mismatch: {reason_code} (expected {expected}, found {found})"
            ),
            Self::AgeExtensionUnavailable { reason_code } => {
                write!(formatter, "age extension unavailable: {reason_code}")
            }
            Self::AgeUnsupportedRelation {
                reason_code,
                relation_marker,
            } => write!(
                formatter,
                "age relation unsupported: {reason_code} ({relation_marker})"
            ),
            Self::TimescaleExtensionUnavailable { reason_code } => {
                write!(formatter, "timescale extension unavailable: {reason_code}")
            }
            Self::InvalidTimescaleBucketWindow {
                reason_code,
                bucket_window_seconds,
            } => write!(
                formatter,
                "timescale bucket window invalid: {reason_code} ({bucket_window_seconds})"
            ),
        }
    }
}

impl std::error::Error for DataLayerPgRepositoryBridgeError {}

/// Projects a deterministic insert-message SQL operation descriptor.
pub fn data_layer_pg_project_insert_message_operation(
    record: &DataLayerM0EnvelopeRecord,
    owner_did: &str,
    requester_did: &str,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_non_empty(record.message_id.as_str(), "message_id")?;
    validate_non_empty(record.content_hash.as_str(), "content_hash")?;
    validate_non_empty(record.hash_chain_prev.as_str(), "hash_chain_prev")?;
    validate_non_empty(record.envelope_ciphertext.as_str(), "envelope_ciphertext")?;
    if record.recipient_dids.is_empty() {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField(
            "recipient_dids",
        ));
    }
    validate_owner_did(owner_did)?;
    let session = build_requester_session(requester_did)?;

    let sql = "INSERT INTO messages (message_id, owner_did, sender_did, recipient_did, envelope_ciphertext, envelope_nonce, content_hash_sha256, hash_chain_prev, blind_indexes, retention_class) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb, $10);";

    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::InsertMessage,
        sql: sql.to_owned(),
        bind_markers: vec![
            "message_id",
            "owner_did",
            "sender_did",
            "recipient_did",
            "envelope_ciphertext",
            "envelope_nonce",
            "content_hash_sha256",
            "hash_chain_prev",
            "blind_indexes",
            "retention_class",
        ],
        session,
    })
}

/// Projects a deterministic message lookup SQL operation descriptor.
pub fn data_layer_pg_project_select_message_by_id_operation(
    message_id: &str,
    requester_did: &str,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_non_empty(message_id, "message_id")?;
    let session = build_requester_session(requester_did)?;

    let sql = "SELECT message_id, owner_did, sender_did, recipient_did, envelope_ciphertext, content_hash_sha256, hash_chain_prev, blind_indexes, retention_class, shredded_at, created_at FROM messages WHERE message_id = $1;";

    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::SelectMessageById,
        sql: sql.to_owned(),
        bind_markers: vec!["message_id"],
        session,
    })
}

/// Projects a deterministic blind-index search SQL operation descriptor.
pub fn data_layer_pg_project_blind_index_search_operation(
    request: DataLayerPgBlindIndexSearchRequest,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_non_empty(request.owner_did.as_str(), "owner_did")?;
    validate_non_empty(request.index_key.as_str(), "index_key")?;
    validate_non_empty(request.index_value_hash.as_str(), "index_value_hash")?;
    validate_owner_did(request.owner_did.as_str())?;
    if request.limit == 0 || request.limit > DATA_LAYER_PG_MAX_BLIND_INDEX_SEARCH_LIMIT {
        return Err(DataLayerPgRepositoryBridgeError::InvalidSearchLimit {
            requested: request.limit,
            max_allowed: DATA_LAYER_PG_MAX_BLIND_INDEX_SEARCH_LIMIT,
        });
    }
    let session = build_requester_session(request.requester_did.as_str())?;

    let sql = "SELECT message_id, owner_did, sender_did, recipient_did, content_hash_sha256, created_at FROM messages WHERE owner_did = $1 AND blind_indexes ->> $2 = $3 ORDER BY created_at DESC LIMIT $4;";

    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::SearchMessagesByBlindIndex,
        sql: sql.to_owned(),
        bind_markers: vec!["owner_did", "index_key", "index_value_hash", "limit"],
        session,
    })
}

/// Projects a deterministic M5 pgvector embedding-insert SQL operation descriptor.
pub fn data_layer_pg_project_m5_embedding_insert_operation(
    record: &DataLayerM5EmbeddingRecord,
    requester_did: &str,
    config: DataLayerPgM5PgvectorConfig,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_pgvector_extension(config)?;
    validate_non_empty(record.embedding_id.as_str(), "embedding_id")?;
    validate_non_empty(record.message_id.as_str(), "message_id")?;
    validate_non_empty(record.owner_did.as_str(), "owner_did")?;
    validate_non_empty(record.agent_did.as_str(), "agent_did")?;
    validate_non_empty(record.model_id.as_str(), "model_id")?;
    validate_owner_did(record.owner_did.as_str())?;
    let vector = record.vector_plaintext.as_ref().ok_or(
        DataLayerPgRepositoryBridgeError::PgvectorDimensionMismatch {
            reason_code: DATA_LAYER_PG_PGVECTOR_DIMENSION_MISMATCH_REASON_CODE,
            expected: config.dimensions,
            found: 0,
        },
    )?;
    if vector.len() != config.dimensions {
        return Err(
            DataLayerPgRepositoryBridgeError::PgvectorDimensionMismatch {
                reason_code: DATA_LAYER_PG_PGVECTOR_DIMENSION_MISMATCH_REASON_CODE,
                expected: config.dimensions,
                found: vector.len(),
            },
        );
    }
    let session = build_requester_session(requester_did)?;

    let sql = "INSERT INTO embeddings (embedding_id, message_id, owner_did, agent_did, model_id, vector_plaintext, vector_dimensions, created_at) VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6::vector, $7, to_timestamp($8));";

    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::InsertEmbeddingVector,
        sql: sql.to_owned(),
        bind_markers: vec![
            "embedding_id",
            "message_id",
            "owner_did",
            "agent_did",
            "model_id",
            "vector_plaintext",
            "vector_dimensions",
            "created_at_epoch_seconds",
        ],
        session,
    })
}

/// Projects a deterministic M5 pgvector similarity-search SQL operation descriptor.
pub fn data_layer_pg_project_m5_similarity_search_operation(
    request: DataLayerPgM5SimilaritySearchRequest,
    config: DataLayerPgM5PgvectorConfig,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_pgvector_extension(config)?;
    validate_non_empty(request.query.owner_did.as_str(), "owner_did")?;
    validate_owner_did(request.query.owner_did.as_str())?;
    if request.query.query_vector.is_empty() {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField("query_vector"));
    }
    if request.query.query_vector.len() != config.dimensions {
        return Err(
            DataLayerPgRepositoryBridgeError::PgvectorDimensionMismatch {
                reason_code: DATA_LAYER_PG_PGVECTOR_DIMENSION_MISMATCH_REASON_CODE,
                expected: config.dimensions,
                found: request.query.query_vector.len(),
            },
        );
    }
    let limit = request
        .query
        .limit
        .unwrap_or(DATA_LAYER_PG_MAX_VECTOR_SEARCH_LIMIT);
    if limit == 0 || limit > DATA_LAYER_PG_MAX_VECTOR_SEARCH_LIMIT {
        return Err(DataLayerPgRepositoryBridgeError::InvalidSearchLimit {
            requested: limit as u32,
            max_allowed: DATA_LAYER_PG_MAX_VECTOR_SEARCH_LIMIT as u32,
        });
    }
    let session = build_requester_session(request.requester_did.as_str())?;

    let sql = "SELECT embedding_id, message_id, owner_did, agent_did, model_id, vector_dimensions, vector_plaintext <=> $2::vector AS cosine_distance FROM embeddings WHERE owner_did = $1 ORDER BY vector_plaintext <=> $2::vector ASC LIMIT $3;";

    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::SearchEmbeddingVectors,
        sql: sql.to_owned(),
        bind_markers: vec!["owner_did", "query_vector", "limit"],
        session,
    })
}

/// Projects a deterministic M6 AGE graph-edge upsert SQL operation descriptor.
pub fn data_layer_pg_project_m6_age_edge_upsert_operation(
    edge: &DataLayerM6GraphEdgeRecord,
    requester_did: &str,
    config: DataLayerPgM6AgeConfig,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_age_config(&config)?;
    validate_non_empty(edge.owner_did.as_str(), "owner_did")?;
    validate_non_empty(edge.edge_id.as_str(), "edge_id")?;
    validate_non_empty(edge.from_node_id.as_str(), "from_node_id")?;
    validate_non_empty(edge.to_node_id.as_str(), "to_node_id")?;
    validate_owner_did(edge.owner_did.as_str())?;
    let relation_marker = map_age_supported_relation(edge.relation)?;
    let session = build_requester_session(requester_did)?;

    let sql = format!(
        "SELECT * FROM cypher('{}', $$ MERGE (from:Agent {{node_id: $4, owner_did: $1}}) MERGE (to:Agent {{node_id: $5, owner_did: $1}}) MERGE (from)-[r:{} {{edge_id: $2, owner_did: $1}}]->(to) SET r.weight = $6, r.observed_at_epoch_seconds = $7 RETURN r.edge_id $$) AS (edge_id agtype);",
        config.graph_name, relation_marker
    );

    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::UpsertGraphEdge,
        sql,
        bind_markers: vec![
            "owner_did",
            "edge_id",
            "relation_marker",
            "from_node_id",
            "to_node_id",
            "weight",
            "observed_at_epoch_seconds",
        ],
        session,
    })
}

/// Projects a deterministic M6 AGE trust-propagation SQL operation descriptor.
pub fn data_layer_pg_project_m6_age_trust_query_operation(
    request: DataLayerPgM6AgeTrustQueryRequest,
    config: DataLayerPgM6AgeConfig,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_age_config(&config)?;
    validate_non_empty(request.query.owner_did.as_str(), "owner_did")?;
    validate_non_empty(
        request.query.source_agent_node_id.as_str(),
        "source_agent_node_id",
    )?;
    validate_owner_did(request.query.owner_did.as_str())?;
    validate_owner_did(request.query.requester_owner_did.as_str())?;
    if request.query.max_depth == 0 {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField("max_depth"));
    }
    let limit = request
        .query
        .limit
        .unwrap_or(DATA_LAYER_PG_MAX_AGE_QUERY_LIMIT);
    if limit == 0 || limit > DATA_LAYER_PG_MAX_AGE_QUERY_LIMIT {
        return Err(DataLayerPgRepositoryBridgeError::InvalidSearchLimit {
            requested: limit as u32,
            max_allowed: DATA_LAYER_PG_MAX_AGE_QUERY_LIMIT as u32,
        });
    }
    let session = build_requester_session(request.requester_did.as_str())?;

    let sql = format!(
        "SELECT * FROM cypher('{}', $$ MATCH (source:Agent {{node_id: $2, owner_did: $1}})-[:TRUSTS*1..$3]->(target:Agent {{owner_did: $1}}) RETURN target.node_id AS target_agent_node_id $$) AS (target_agent_node_id agtype) LIMIT $4;",
        config.graph_name
    );

    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::QueryGraphTrustPropagation,
        sql,
        bind_markers: vec!["owner_did", "source_agent_node_id", "max_depth", "limit"],
        session,
    })
}

/// Projects a deterministic M7 Timescale telemetry-ingest SQL operation descriptor.
pub fn data_layer_pg_project_m7_timescale_ingest_operation(
    record: &DataLayerM7TelemetryPointRecord,
    requester_did: &str,
    config: DataLayerPgM7TimescaleConfig,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_timescale_config(&config)?;
    validate_owner_did(record.owner_did.as_str())?;
    validate_non_empty(record.agent_did.as_str(), "agent_did")?;
    if record.timestamp_epoch_seconds == 0 {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField(
            "timestamp_epoch_seconds",
        ));
    }
    let expected_hour_bucket = record.timestamp_epoch_seconds
        - (record.timestamp_epoch_seconds % DATA_LAYER_M7_HOURLY_BUCKET_SECONDS);
    let expected_day_bucket = record.timestamp_epoch_seconds
        - (record.timestamp_epoch_seconds % DATA_LAYER_M7_DAILY_BUCKET_SECONDS);
    if record.bucket_hour_epoch_seconds != expected_hour_bucket {
        return Err(
            DataLayerPgRepositoryBridgeError::InvalidTimescaleBucketWindow {
                reason_code: DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE,
                bucket_window_seconds: DATA_LAYER_M7_HOURLY_BUCKET_SECONDS,
            },
        );
    }
    if record.bucket_day_epoch_seconds != expected_day_bucket {
        return Err(
            DataLayerPgRepositoryBridgeError::InvalidTimescaleBucketWindow {
                reason_code: DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE,
                bucket_window_seconds: DATA_LAYER_M7_DAILY_BUCKET_SECONDS,
            },
        );
    }
    let session = build_requester_session(requester_did)?;

    let sql = format!(
        "INSERT INTO {} (owner_did, agent_did, observed_at, bucket_hour_epoch_seconds, bucket_day_epoch_seconds, message_count, bytes_stored, query_count, embedding_count, embedding_anomaly_count, ingress_latency_ms_p95, egress_latency_ms_p95, active_sessions, sequence) VALUES ($1, $2, to_timestamp($3), $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14);",
        config.hypertable_name
    );

    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::InsertTelemetryPoint,
        sql,
        bind_markers: vec![
            "owner_did",
            "agent_did",
            "timestamp_epoch_seconds",
            "bucket_hour_epoch_seconds",
            "bucket_day_epoch_seconds",
            "message_count",
            "bytes_stored",
            "query_count",
            "embedding_count",
            "embedding_anomaly_count",
            "ingress_latency_ms_p95",
            "egress_latency_ms_p95",
            "active_sessions",
            "sequence",
        ],
        session,
    })
}

/// Projects a deterministic M7 Timescale owner-rollup SQL operation descriptor.
pub fn data_layer_pg_project_m7_timescale_owner_rollup_query_operation(
    request: DataLayerPgM7TimescaleOwnerRollupRequest,
    config: DataLayerPgM7TimescaleConfig,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_timescale_config(&config)?;
    validate_owner_did(request.query.owner_did.as_str())?;
    validate_owner_did(request.query.requester_owner_did.as_str())?;
    let interval_marker = match request.bucket_window_seconds {
        DATA_LAYER_M7_HOURLY_BUCKET_SECONDS => "1 hour",
        DATA_LAYER_M7_DAILY_BUCKET_SECONDS => "1 day",
        other => {
            return Err(
                DataLayerPgRepositoryBridgeError::InvalidTimescaleBucketWindow {
                    reason_code: DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE,
                    bucket_window_seconds: other,
                },
            )
        }
    };
    let limit = request
        .limit
        .unwrap_or(DATA_LAYER_PG_MAX_TIMESCALE_QUERY_LIMIT);
    if limit == 0 || limit > DATA_LAYER_PG_MAX_TIMESCALE_QUERY_LIMIT {
        return Err(DataLayerPgRepositoryBridgeError::InvalidSearchLimit {
            requested: limit as u32,
            max_allowed: DATA_LAYER_PG_MAX_TIMESCALE_QUERY_LIMIT as u32,
        });
    }
    let session = build_requester_session(request.requester_did.as_str())?;

    let sql = format!(
        "SELECT time_bucket(INTERVAL '{}', observed_at) AS bucket_start, SUM(message_count) AS message_count_total, SUM(bytes_stored) AS bytes_stored_total, SUM(query_count) AS query_count_total, SUM(embedding_count) AS embedding_count_total FROM {} WHERE owner_did = $1 GROUP BY bucket_start ORDER BY bucket_start DESC LIMIT $2;",
        interval_marker, config.hypertable_name
    );

    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::QueryTelemetryOwnerRollup,
        sql,
        bind_markers: vec!["owner_did", "limit"],
        session,
    })
}

/// Projects default M2 RLS templates into deterministic SQL statement descriptors.
pub fn data_layer_pg_project_default_rls_statements() -> Vec<DataLayerPgRlsStatement> {
    let mut policies = data_layer_m2_default_rls_policies();
    policies.sort_by(|left, right| {
        left.table_name
            .cmp(&right.table_name)
            .then(left.policy_name.cmp(&right.policy_name))
    });

    let mut statements = Vec::with_capacity(policies.len() * 3);
    for policy in policies {
        statements.push(DataLayerPgRlsStatement {
            table_name: policy.table_name.clone(),
            policy_name: policy.policy_name.clone(),
            sql: format!(
                "ALTER TABLE {} ENABLE ROW LEVEL SECURITY;",
                policy.table_name
            ),
        });
        statements.push(DataLayerPgRlsStatement {
            table_name: policy.table_name.clone(),
            policy_name: policy.policy_name.clone(),
            sql: format!(
                "DROP POLICY IF EXISTS {} ON {};",
                policy.policy_name, policy.table_name
            ),
        });
        let mut create_sql = format!(
            "CREATE POLICY {} ON {} USING ({}",
            policy.policy_name, policy.table_name, policy.using_clause
        );
        create_sql.push(')');
        if let Some(with_check_clause) = policy.with_check_clause {
            create_sql.push_str(format!(" WITH CHECK ({with_check_clause})").as_str());
        }
        create_sql.push(';');
        statements.push(DataLayerPgRlsStatement {
            table_name: policy.table_name,
            policy_name: policy.policy_name,
            sql: create_sql,
        });
    }

    statements
}

fn build_requester_session(
    requester_did: &str,
) -> Result<DataLayerPgRequesterSession, DataLayerPgRepositoryBridgeError> {
    let parsed = AgentDid::parse(requester_did).map_err(|error| {
        DataLayerPgRepositoryBridgeError::InvalidRequesterDid {
            field: "requester_did",
            reason_code: DATA_LAYER_PG_INVALID_REQUESTER_DID_REASON_CODE,
            detail: error.to_string(),
        }
    })?;
    Ok(DataLayerPgRequesterSession {
        setting_key: DATA_LAYER_M2_REQUESTER_DID_SETTING,
        requester_did: parsed.as_str().to_owned(),
    })
}

fn validate_owner_did(owner_did: &str) -> Result<(), DataLayerPgRepositoryBridgeError> {
    KamnDid::parse(owner_did).map_err(|error| {
        DataLayerPgRepositoryBridgeError::InvalidOwnerDid {
            field: "owner_did",
            reason_code: DATA_LAYER_PG_INVALID_OWNER_DID_REASON_CODE,
            detail: error.to_string(),
        }
    })?;
    Ok(())
}

fn validate_pgvector_extension(
    config: DataLayerPgM5PgvectorConfig,
) -> Result<(), DataLayerPgRepositoryBridgeError> {
    if !config.extension_enabled {
        return Err(
            DataLayerPgRepositoryBridgeError::PgvectorExtensionUnavailable {
                reason_code: DATA_LAYER_PG_PGVECTOR_EXTENSION_UNAVAILABLE_REASON_CODE,
            },
        );
    }
    if config.dimensions == 0 {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField(
            "pgvector_dimensions",
        ));
    }
    Ok(())
}

fn validate_age_config(
    config: &DataLayerPgM6AgeConfig,
) -> Result<(), DataLayerPgRepositoryBridgeError> {
    if !config.extension_enabled {
        return Err(DataLayerPgRepositoryBridgeError::AgeExtensionUnavailable {
            reason_code: DATA_LAYER_PG_AGE_EXTENSION_UNAVAILABLE_REASON_CODE,
        });
    }
    if config.graph_name.trim().is_empty() {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField(
            "age_graph_name",
        ));
    }
    Ok(())
}

fn validate_timescale_config(
    config: &DataLayerPgM7TimescaleConfig,
) -> Result<(), DataLayerPgRepositoryBridgeError> {
    if !config.extension_enabled {
        return Err(
            DataLayerPgRepositoryBridgeError::TimescaleExtensionUnavailable {
                reason_code: DATA_LAYER_PG_TIMESCALE_EXTENSION_UNAVAILABLE_REASON_CODE,
            },
        );
    }
    if config.hypertable_name.trim().is_empty() {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField(
            "timescale_hypertable_name",
        ));
    }
    Ok(())
}

fn map_age_supported_relation(
    relation: DataLayerM6GraphEdgeRelation,
) -> Result<&'static str, DataLayerPgRepositoryBridgeError> {
    let relation_marker = match relation {
        DataLayerM6GraphEdgeRelation::Messaged => "MESSAGED",
        DataLayerM6GraphEdgeRelation::Trusts => "TRUSTS",
        DataLayerM6GraphEdgeRelation::ParticipatedIn => "PARTICIPATED_IN",
        DataLayerM6GraphEdgeRelation::Owns => "OWNS",
        DataLayerM6GraphEdgeRelation::DelegatedTo => "DELEGATED_TO",
        DataLayerM6GraphEdgeRelation::BelongsToCluster => "BELONGS_TO_CLUSTER",
        DataLayerM6GraphEdgeRelation::ForkedFrom => "FORKED_FROM",
    };
    if relation == DataLayerM6GraphEdgeRelation::Trusts {
        Ok(relation_marker)
    } else {
        Err(DataLayerPgRepositoryBridgeError::AgeUnsupportedRelation {
            reason_code: DATA_LAYER_PG_AGE_RELATION_UNSUPPORTED_REASON_CODE,
            relation_marker,
        })
    }
}

fn validate_non_empty(
    value: &str,
    field: &'static str,
) -> Result<(), DataLayerPgRepositoryBridgeError> {
    if value.trim().is_empty() {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField(field));
    }
    Ok(())
}

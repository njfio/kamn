/// Deterministic operation kind projected by the bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerPgOperationKind {
    /// Insert message variant for this public contract enum.
    InsertMessage,
    /// Select message by id variant for this public contract enum.
    SelectMessageById,
    /// Search messages by blind index variant for this public contract enum.
    SearchMessagesByBlindIndex,
    /// Insert merkle batch variant for this public contract enum.
    InsertMerkleBatch,
    /// Assign message merkle batch variant for this public contract enum.
    AssignMessageMerkleBatch,
    /// Mark merkle batch submitted variant for this public contract enum.
    MarkMerkleBatchSubmitted,
    /// Mark merkle batch confirmed variant for this public contract enum.
    MarkMerkleBatchConfirmed,
    /// Insert embedding vector variant for this public contract enum.
    InsertEmbeddingVector,
    /// Search embedding vectors variant for this public contract enum.
    SearchEmbeddingVectors,
    /// Upsert graph edge variant for this public contract enum.
    UpsertGraphEdge,
    /// Query graph trust propagation variant for this public contract enum.
    QueryGraphTrustPropagation,
    /// Insert telemetry point variant for this public contract enum.
    InsertTelemetryPoint,
    /// Query telemetry owner rollup variant for this public contract enum.
    QueryTelemetryOwnerRollup,
}

/// Requester session metadata projected into SQL execution context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgRequesterSession {
    /// Setting key carried by this public contract model.
    pub setting_key: &'static str,
    /// Requester did carried by this public contract model.
    pub requester_did: String,
}

/// Deterministic SQL operation descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgSqlOperation {
    /// Kind carried by this public contract model.
    pub kind: DataLayerPgOperationKind,
    /// Sql carried by this public contract model.
    pub sql: String,
    /// Bind markers carried by this public contract model.
    pub bind_markers: Vec<&'static str>,
    /// Session carried by this public contract model.
    pub session: DataLayerPgRequesterSession,
}

/// Blind-index search request projected into SQL operation descriptors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgBlindIndexSearchRequest {
    /// Requester did carried by this public contract model.
    pub requester_did: String,
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Index key carried by this public contract model.
    pub index_key: String,
    /// Index value hash carried by this public contract model.
    pub index_value_hash: String,
    /// Limit carried by this public contract model.
    pub limit: u32,
}

/// RLS SQL statement descriptor projected from M2 templates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgRlsStatement {
    /// Table name carried by this public contract model.
    pub table_name: String,
    /// Policy name carried by this public contract model.
    pub policy_name: String,
    /// Sql carried by this public contract model.
    pub sql: String,
}

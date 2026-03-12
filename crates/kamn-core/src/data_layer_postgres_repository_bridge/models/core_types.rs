/// Deterministic operation kind projected by the bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerPgOperationKind {
    InsertMessage,
    SelectMessageById,
    SearchMessagesByBlindIndex,
    InsertMerkleBatch,
    AssignMessageMerkleBatch,
    MarkMerkleBatchSubmitted,
    MarkMerkleBatchConfirmed,
    InsertEmbeddingVector,
    SearchEmbeddingVectors,
    UpsertGraphEdge,
    QueryGraphTrustPropagation,
    InsertTelemetryPoint,
    QueryTelemetryOwnerRollup,
}

/// Requester session metadata projected into SQL execution context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgRequesterSession {
    pub setting_key: &'static str,
    pub requester_did: String,
}

/// Deterministic SQL operation descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgSqlOperation {
    pub kind: DataLayerPgOperationKind,
    pub sql: String,
    pub bind_markers: Vec<&'static str>,
    pub session: DataLayerPgRequesterSession,
}

/// Blind-index search request projected into SQL operation descriptors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgBlindIndexSearchRequest {
    pub requester_did: String,
    pub owner_did: String,
    pub index_key: String,
    pub index_value_hash: String,
    pub limit: u32,
}

/// RLS SQL statement descriptor projected from M2 templates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgRlsStatement {
    pub table_name: String,
    pub policy_name: String,
    pub sql: String,
}

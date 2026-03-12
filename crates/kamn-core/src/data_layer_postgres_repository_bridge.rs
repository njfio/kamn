//! PostgreSQL repository bridge contracts for data-layer persistence wiring.
//!
//! This module does not execute SQL. It projects validated data-layer inputs
//! into deterministic SQL operation descriptors that runtime adapters can
//! execute later.

mod m5_pgvector;
mod m6_age;
mod m7_timescale;
mod models;
mod session;
mod support;

pub use m5_pgvector::{
    data_layer_pg_project_m5_embedding_insert_operation,
    data_layer_pg_project_m5_similarity_search_operation,
};
pub use m6_age::{
    data_layer_pg_project_m6_age_edge_upsert_operation,
    data_layer_pg_project_m6_age_trust_query_operation,
};
pub use m7_timescale::{
    data_layer_pg_project_m7_timescale_ingest_operation,
    data_layer_pg_project_m7_timescale_owner_rollup_query_operation,
};
pub use models::{
    DataLayerPgBlindIndexSearchRequest, DataLayerPgM5PgvectorConfig,
    DataLayerPgM5SimilaritySearchRequest, DataLayerPgM6AgeConfig,
    DataLayerPgM6AgeTrustQueryRequest, DataLayerPgM7TimescaleConfig,
    DataLayerPgM7TimescaleOwnerRollupRequest, DataLayerPgOperationKind,
    DataLayerPgRepositoryBridgeError, DataLayerPgRequesterSession, DataLayerPgRlsStatement,
    DataLayerPgSqlOperation, DATA_LAYER_PG_AGE_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_AGE_RELATION_UNSUPPORTED_REASON_CODE,
    DATA_LAYER_PG_INVALID_OWNER_DID_REASON_CODE, DATA_LAYER_PG_INVALID_REQUESTER_DID_REASON_CODE,
    DATA_LAYER_PG_PGVECTOR_DIMENSION_MISMATCH_REASON_CODE,
    DATA_LAYER_PG_PGVECTOR_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_TIMESCALE_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE,
};
pub use session::{
    data_layer_pg_project_blind_index_search_operation,
    data_layer_pg_project_default_rls_statements, data_layer_pg_project_insert_message_operation,
    data_layer_pg_project_select_message_by_id_operation,
};

pub(crate) use models::{
    map_age_supported_relation, validate_age_config, validate_pgvector_extension,
    validate_timescale_config, DATA_LAYER_PG_MAX_AGE_QUERY_LIMIT,
    DATA_LAYER_PG_MAX_BLIND_INDEX_SEARCH_LIMIT, DATA_LAYER_PG_MAX_TIMESCALE_QUERY_LIMIT,
    DATA_LAYER_PG_MAX_VECTOR_SEARCH_LIMIT,
};
pub(crate) use session::{build_requester_session, validate_owner_did};
pub(crate) use support::validate_non_empty;

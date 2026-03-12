mod config_types;
mod constants;
mod core_types;
mod error;

pub use config_types::{
    DataLayerPgM5PgvectorConfig, DataLayerPgM5SimilaritySearchRequest, DataLayerPgM6AgeConfig,
    DataLayerPgM6AgeTrustQueryRequest, DataLayerPgM7TimescaleConfig,
    DataLayerPgM7TimescaleOwnerRollupRequest,
};
pub use constants::{
    DATA_LAYER_PG_AGE_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_AGE_RELATION_UNSUPPORTED_REASON_CODE,
    DATA_LAYER_PG_INVALID_OWNER_DID_REASON_CODE, DATA_LAYER_PG_INVALID_REQUESTER_DID_REASON_CODE,
    DATA_LAYER_PG_MAX_AGE_QUERY_LIMIT, DATA_LAYER_PG_MAX_BLIND_INDEX_SEARCH_LIMIT,
    DATA_LAYER_PG_MAX_TIMESCALE_QUERY_LIMIT, DATA_LAYER_PG_MAX_VECTOR_SEARCH_LIMIT,
    DATA_LAYER_PG_PGVECTOR_DIMENSION_MISMATCH_REASON_CODE,
    DATA_LAYER_PG_PGVECTOR_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_TIMESCALE_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE,
};
pub use core_types::{
    DataLayerPgBlindIndexSearchRequest, DataLayerPgOperationKind, DataLayerPgRequesterSession,
    DataLayerPgRlsStatement, DataLayerPgSqlOperation,
};
pub use error::DataLayerPgRepositoryBridgeError;

pub(crate) use config_types::{
    map_age_supported_relation, validate_age_config, validate_pgvector_extension,
    validate_timescale_config,
};

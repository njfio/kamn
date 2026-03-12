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

pub const DATA_LAYER_PG_MAX_BLIND_INDEX_SEARCH_LIMIT: u32 = 200;
pub const DATA_LAYER_PG_MAX_VECTOR_SEARCH_LIMIT: usize = 200;
pub const DATA_LAYER_PG_MAX_AGE_QUERY_LIMIT: usize = 200;
pub const DATA_LAYER_PG_MAX_TIMESCALE_QUERY_LIMIT: usize = 200;

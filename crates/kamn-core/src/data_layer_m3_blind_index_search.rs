//! M3 searchable-index contracts for blind-index and metadata lookups.
//!
//! This module models PRD M3 contracts as deterministic in-memory Rust APIs:
//! owner-scoped blind-index token derivation, exact-match blind-index lookups,
//! and metadata filter queries with stable ordering.

#[path = "data_layer_m3_blind_index_search/catalog.rs"]
mod catalog;
#[path = "data_layer_m3_blind_index_search/errors.rs"]
mod errors;
#[path = "data_layer_m3_blind_index_search/hashing.rs"]
mod hashing;
#[path = "data_layer_m3_blind_index_search/models.rs"]
mod models;
#[path = "data_layer_m3_blind_index_search/validation.rs"]
mod validation;

pub use errors::*;
pub use hashing::*;
pub use models::*;
pub(crate) use validation::{
    canonical_field_name, map_content_retrieval_error_to_m3_projection_error,
    normalize_blind_index_value, resolve_limit, sort_results_deterministically,
    validate_blind_index_token, validate_kamn_did, validate_non_empty,
};

#[cfg(test)]
#[path = "data_layer_m3_blind_index_search/tests.rs"]
mod tests;

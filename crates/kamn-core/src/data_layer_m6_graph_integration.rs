//! M6 graph-layer contracts for owner-scoped schema, trust propagation, and portability.
//!
//! This module models PRD M6 behavior as deterministic Rust contracts:
//! owner-scoped graph node/edge registration, bounded trust propagation ranking,
//! and portable edge projection exports suitable for AGE/openCypher adapters.

mod export;
mod models;
mod registry;
mod support;
#[cfg(test)]
mod tests;
mod trust_query;

pub use models::{
    DATA_LAYER_M6_CROSS_OWNER_EDGE_DENIED_REASON_CODE, DATA_LAYER_M6_GRAPH_ENGINE_APACHE_AGE,
    DATA_LAYER_M6_GRAPH_PORTABILITY_PROFILE, DATA_LAYER_M6_OWNER_SCOPE_DENIED_REASON_CODE,
    DATA_LAYER_M6_TRUST_PROPAGATION_REASON_RANKED, DataLayerM6GraphEdgeInput,
    DataLayerM6GraphEdgeRecord, DataLayerM6GraphEdgeRelation, DataLayerM6GraphNodeInput,
    DataLayerM6GraphNodeKind, DataLayerM6GraphNodeRecord, DataLayerM6PortableEdgeProjection,
    DataLayerM6TrustPropagationQuery, DataLayerM6TrustPropagationResult,
};
pub use registry::DataLayerM6GraphRegistry;
pub use support::DataLayerM6GraphIntegrationError;
pub(crate) use support::{resolve_limit, validate_non_empty, validate_weight};

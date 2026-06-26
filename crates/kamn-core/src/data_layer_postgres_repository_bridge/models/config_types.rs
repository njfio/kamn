use crate::{
    DataLayerM5SemanticQuery, DataLayerM6GraphEdgeRelation, DataLayerM6TrustPropagationQuery,
    DataLayerM7BillingQuery,
};

use super::{
    DataLayerPgRepositoryBridgeError, DATA_LAYER_PG_AGE_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_AGE_RELATION_UNSUPPORTED_REASON_CODE,
    DATA_LAYER_PG_PGVECTOR_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_TIMESCALE_EXTENSION_UNAVAILABLE_REASON_CODE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Public contract model for Data Layer Pg M5 Pgvector Config.
pub struct DataLayerPgM5PgvectorConfig {
    /// Extension enabled carried by this public contract model.
    pub extension_enabled: bool,
    /// Dimensions carried by this public contract model.
    pub dimensions: usize,
}

impl DataLayerPgM5PgvectorConfig {
    /// Creates a new value for this public contract type.
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

#[derive(Debug, Clone, PartialEq)]
/// Public contract model for Data Layer Pg M5 Similarity Search Request.
pub struct DataLayerPgM5SimilaritySearchRequest {
    /// Requester did carried by this public contract model.
    pub requester_did: String,
    /// Query carried by this public contract model.
    pub query: DataLayerM5SemanticQuery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer Pg M6 Age Config.
pub struct DataLayerPgM6AgeConfig {
    /// Extension enabled carried by this public contract model.
    pub extension_enabled: bool,
    /// Graph name carried by this public contract model.
    pub graph_name: String,
}

impl DataLayerPgM6AgeConfig {
    /// Creates a new value for this public contract type.
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

#[derive(Debug, Clone, PartialEq)]
/// Public contract model for Data Layer Pg M6 Age Trust Query Request.
pub struct DataLayerPgM6AgeTrustQueryRequest {
    /// Requester did carried by this public contract model.
    pub requester_did: String,
    /// Query carried by this public contract model.
    pub query: DataLayerM6TrustPropagationQuery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer Pg M7 Timescale Config.
pub struct DataLayerPgM7TimescaleConfig {
    /// Extension enabled carried by this public contract model.
    pub extension_enabled: bool,
    /// Hypertable name carried by this public contract model.
    pub hypertable_name: String,
}

impl DataLayerPgM7TimescaleConfig {
    /// Creates a new value for this public contract type.
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

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer Pg M7 Timescale Owner Rollup Request.
pub struct DataLayerPgM7TimescaleOwnerRollupRequest {
    /// Requester did carried by this public contract model.
    pub requester_did: String,
    /// Query carried by this public contract model.
    pub query: DataLayerM7BillingQuery,
    /// Bucket window seconds carried by this public contract model.
    pub bucket_window_seconds: u64,
    /// Limit carried by this public contract model.
    pub limit: Option<usize>,
}

pub(crate) fn validate_pgvector_extension(
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

pub(crate) fn validate_age_config(
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

pub(crate) fn validate_timescale_config(
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

pub(crate) fn map_age_supported_relation(
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

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
pub struct DataLayerPgM5PgvectorConfig {
    pub extension_enabled: bool,
    pub dimensions: usize,
}

impl DataLayerPgM5PgvectorConfig {
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
pub struct DataLayerPgM5SimilaritySearchRequest {
    pub requester_did: String,
    pub query: DataLayerM5SemanticQuery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgM6AgeConfig {
    pub extension_enabled: bool,
    pub graph_name: String,
}

impl DataLayerPgM6AgeConfig {
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
pub struct DataLayerPgM6AgeTrustQueryRequest {
    pub requester_did: String,
    pub query: DataLayerM6TrustPropagationQuery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgM7TimescaleConfig {
    pub extension_enabled: bool,
    pub hypertable_name: String,
}

impl DataLayerPgM7TimescaleConfig {
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
pub struct DataLayerPgM7TimescaleOwnerRollupRequest {
    pub requester_did: String,
    pub query: DataLayerM7BillingQuery,
    pub bucket_window_seconds: u64,
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

use crate::KamnDid;
use std::fmt;

/// Error taxonomy for M6 graph integration contracts.
#[derive(Debug, Clone, PartialEq)]
pub enum DataLayerM6GraphIntegrationError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// DID failed validation.
    InvalidDid(String),
    /// Node id already exists in owner scope.
    DuplicateNodeId {
        /// Owner DID scope.
        owner_did: String,
        /// Duplicate node id.
        node_id: String,
    },
    /// Edge id already exists.
    DuplicateEdgeId(String),
    /// Owner scope was not found.
    OwnerNotFound {
        /// Missing owner DID.
        owner_did: String,
    },
    /// Node not found in owner scope.
    NodeNotFound {
        /// Owner DID scope.
        owner_did: String,
        /// Missing node id.
        node_id: String,
    },
    /// Weight is invalid.
    InvalidWeight(f32),
    /// Query depth is invalid.
    InvalidDepth(u8),
    /// Query limit is invalid.
    InvalidLimit(usize),
    /// Attenuation factor is invalid.
    InvalidAttenuationFactor(f32),
    /// Source node is missing or not an agent.
    InvalidSourceAgentNode(String),
    /// Request violated owner scope isolation.
    OwnerScopeViolation {
        /// Stable reason marker.
        reason_code: &'static str,
    },
}

impl fmt::Display for DataLayerM6GraphIntegrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::DuplicateNodeId { owner_did, node_id } => {
                write!(f, "duplicate node id {node_id} in owner scope {owner_did}")
            }
            Self::DuplicateEdgeId(edge_id) => write!(f, "duplicate edge id: {edge_id}"),
            Self::OwnerNotFound { owner_did } => write!(f, "owner not found: {owner_did}"),
            Self::NodeNotFound { owner_did, node_id } => {
                write!(f, "node not found in owner scope {owner_did}: {node_id}")
            }
            Self::InvalidWeight(weight) => write!(f, "invalid edge weight: {weight}"),
            Self::InvalidDepth(depth) => write!(f, "invalid propagation depth: {depth}"),
            Self::InvalidLimit(limit) => write!(f, "invalid limit: {limit}"),
            Self::InvalidAttenuationFactor(value) => {
                write!(f, "invalid attenuation factor: {value}")
            }
            Self::InvalidSourceAgentNode(node_id) => {
                write!(f, "invalid source agent node: {node_id}")
            }
            Self::OwnerScopeViolation { reason_code } => {
                write!(f, "owner scope violation: {reason_code}")
            }
        }
    }
}

impl std::error::Error for DataLayerM6GraphIntegrationError {}

pub(super) fn parse_kamn_did(value: &str) -> Result<KamnDid, DataLayerM6GraphIntegrationError> {
    KamnDid::parse(value)
        .map_err(|_| DataLayerM6GraphIntegrationError::InvalidDid(value.to_owned()))
}

pub(crate) fn validate_non_empty(
    value: &str,
    field_name: &'static str,
) -> Result<(), DataLayerM6GraphIntegrationError> {
    if value.trim().is_empty() {
        return Err(DataLayerM6GraphIntegrationError::EmptyField(field_name));
    }
    Ok(())
}

pub(crate) fn validate_weight(weight: f32) -> Result<(), DataLayerM6GraphIntegrationError> {
    if !weight.is_finite() || weight <= 0.0 || weight > 1.0 {
        return Err(DataLayerM6GraphIntegrationError::InvalidWeight(weight));
    }
    Ok(())
}

pub(crate) fn resolve_limit(
    limit: Option<usize>,
) -> Result<usize, DataLayerM6GraphIntegrationError> {
    let resolved = limit.unwrap_or(20);
    if resolved == 0 {
        return Err(DataLayerM6GraphIntegrationError::InvalidLimit(resolved));
    }
    Ok(resolved)
}

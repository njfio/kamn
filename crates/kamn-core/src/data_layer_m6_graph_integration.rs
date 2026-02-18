//! M6 graph-layer contracts for owner-scoped schema, trust propagation, and portability.
//!
//! This module models PRD M6 behavior as deterministic Rust contracts:
//! owner-scoped graph node/edge registration, bounded trust propagation ranking,
//! and portable edge projection exports suitable for AGE/openCypher adapters.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Graph engine profile marker for M6 contracts.
pub const DATA_LAYER_M6_GRAPH_ENGINE_APACHE_AGE: &str = "apache-age";
/// Portability profile marker for exported edge projections.
pub const DATA_LAYER_M6_GRAPH_PORTABILITY_PROFILE: &str = "age-open-cypher-portable-v1";
/// Stable reason marker used for successful trust propagation ranking outputs.
pub const DATA_LAYER_M6_TRUST_PROPAGATION_REASON_RANKED: &str = "m6_graph_trust_score_ranked";
/// Stable reason marker for owner-scope authorization denials.
pub const DATA_LAYER_M6_OWNER_SCOPE_DENIED_REASON_CODE: &str = "m6_graph_owner_scope_denied";
/// Stable reason marker for cross-owner edge registration denials.
pub const DATA_LAYER_M6_CROSS_OWNER_EDGE_DENIED_REASON_CODE: &str =
    "m6_graph_cross_owner_edge_denied";

/// Supported node kinds for the M6 graph schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM6GraphNodeKind {
    /// Agent node.
    Agent,
    /// Owner node.
    Owner,
    /// Escrow node.
    Escrow,
    /// Capability node.
    Capability,
    /// Conversation node.
    Conversation,
}

/// Input payload for registering one graph node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM6GraphNodeInput {
    /// Owner DID scope.
    pub owner_did: String,
    /// Stable node identifier within owner scope.
    pub node_id: String,
    /// Node kind marker.
    pub kind: DataLayerM6GraphNodeKind,
    /// Human-readable label.
    pub label: String,
}

/// Stored graph node record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM6GraphNodeRecord {
    /// Owner DID scope.
    pub owner_did: String,
    /// Stable node identifier.
    pub node_id: String,
    /// Node kind marker.
    pub kind: DataLayerM6GraphNodeKind,
    /// Human-readable label.
    pub label: String,
    /// Append-order sequence.
    pub sequence: u64,
}

/// Supported relationship edges for M6 graph schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM6GraphEdgeRelation {
    /// Agent sent messages to another agent.
    Messaged,
    /// Agent trust score relation to another agent.
    Trusts,
    /// Agent participated in escrow.
    ParticipatedIn,
    /// Owner owns agent.
    Owns,
    /// Agent delegated authority to another agent.
    DelegatedTo,
    /// Agent belongs to a cluster.
    BelongsToCluster,
    /// Agent fork provenance relation.
    ForkedFrom,
}

impl DataLayerM6GraphEdgeRelation {
    fn marker(self) -> &'static str {
        match self {
            Self::Messaged => "MESSAGED",
            Self::Trusts => "TRUSTS",
            Self::ParticipatedIn => "PARTICIPATED_IN",
            Self::Owns => "OWNS",
            Self::DelegatedTo => "DELEGATED_TO",
            Self::BelongsToCluster => "BELONGS_TO_CLUSTER",
            Self::ForkedFrom => "FORKED_FROM",
        }
    }
}

/// Input payload for registering one graph edge.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM6GraphEdgeInput {
    /// Owner DID scope.
    pub owner_did: String,
    /// Stable edge identifier.
    pub edge_id: String,
    /// Relationship type.
    pub relation: DataLayerM6GraphEdgeRelation,
    /// Source node identifier.
    pub from_node_id: String,
    /// Target node identifier.
    pub to_node_id: String,
    /// Edge weight in [0.0, 1.0].
    pub weight: f32,
    /// Observation timestamp in epoch seconds.
    pub observed_at_epoch_seconds: u64,
}

/// Stored graph edge record.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM6GraphEdgeRecord {
    /// Owner DID scope.
    pub owner_did: String,
    /// Stable edge identifier.
    pub edge_id: String,
    /// Relationship type.
    pub relation: DataLayerM6GraphEdgeRelation,
    /// Source node identifier.
    pub from_node_id: String,
    /// Target node identifier.
    pub to_node_id: String,
    /// Edge weight in [0.0, 1.0].
    pub weight: f32,
    /// Observation timestamp in epoch seconds.
    pub observed_at_epoch_seconds: u64,
    /// Append-order sequence.
    pub sequence: u64,
}

/// Trust propagation query envelope.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM6TrustPropagationQuery {
    /// Requester owner DID used for scope authorization.
    pub requester_owner_did: String,
    /// Owner DID scope to query.
    pub owner_did: String,
    /// Source agent node id.
    pub source_agent_node_id: String,
    /// Maximum traversal depth.
    pub max_depth: u8,
    /// Hop attenuation factor in (0.0, 1.0].
    pub attenuation_factor: f32,
    /// Optional maximum number of rows to return.
    pub limit: Option<usize>,
}

/// One trust propagation ranking row.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM6TrustPropagationResult {
    /// Target agent node id.
    pub target_agent_node_id: String,
    /// Computed trust score.
    pub trust_score: f32,
    /// Minimum hop distance from source.
    pub hops: u8,
    /// Stable reason marker.
    pub reason_code: &'static str,
}

/// Portable edge projection row for adapter handoff.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM6PortableEdgeProjection {
    /// Graph engine marker.
    pub graph_engine_marker: &'static str,
    /// Portability profile marker.
    pub portability_profile: &'static str,
    /// Owner DID scope.
    pub owner_did: String,
    /// Stable edge identifier.
    pub edge_id: String,
    /// Relationship marker string.
    pub relation_marker: &'static str,
    /// Source node identifier.
    pub from_node_id: String,
    /// Target node identifier.
    pub to_node_id: String,
    /// Edge weight.
    pub weight: f32,
}

/// M6 owner-scoped graph registry and trust propagation service.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DataLayerM6GraphRegistry {
    nodes_by_owner: BTreeMap<String, Vec<DataLayerM6GraphNodeRecord>>,
    edges_by_owner: BTreeMap<String, Vec<DataLayerM6GraphEdgeRecord>>,
    seen_edge_ids: BTreeSet<String>,
}

impl DataLayerM6GraphRegistry {
    /// Creates an empty graph registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one owner-scoped node.
    pub fn register_node(
        &mut self,
        input: DataLayerM6GraphNodeInput,
    ) -> Result<DataLayerM6GraphNodeRecord, DataLayerM6GraphIntegrationError> {
        validate_kamn_did(input.owner_did.as_str())?;
        validate_non_empty(input.node_id.as_str(), "node_id")?;
        validate_non_empty(input.label.as_str(), "label")?;

        let owner_nodes = self
            .nodes_by_owner
            .entry(input.owner_did.clone())
            .or_default();
        if owner_nodes
            .iter()
            .any(|record| record.node_id == input.node_id)
        {
            return Err(DataLayerM6GraphIntegrationError::DuplicateNodeId {
                owner_did: input.owner_did,
                node_id: input.node_id,
            });
        }

        let record = DataLayerM6GraphNodeRecord {
            owner_did: input.owner_did,
            node_id: input.node_id,
            kind: input.kind,
            label: input.label,
            sequence: owner_nodes.len() as u64 + 1,
        };
        owner_nodes.push(record.clone());
        Ok(record)
    }

    /// Registers one owner-scoped edge.
    pub fn register_edge(
        &mut self,
        input: DataLayerM6GraphEdgeInput,
    ) -> Result<DataLayerM6GraphEdgeRecord, DataLayerM6GraphIntegrationError> {
        validate_kamn_did(input.owner_did.as_str())?;
        validate_non_empty(input.edge_id.as_str(), "edge_id")?;
        validate_non_empty(input.from_node_id.as_str(), "from_node_id")?;
        validate_non_empty(input.to_node_id.as_str(), "to_node_id")?;
        validate_weight(input.weight)?;
        if input.observed_at_epoch_seconds == 0 {
            return Err(DataLayerM6GraphIntegrationError::EmptyField(
                "observed_at_epoch_seconds",
            ));
        }

        if self.seen_edge_ids.contains(input.edge_id.as_str()) {
            return Err(DataLayerM6GraphIntegrationError::DuplicateEdgeId(
                input.edge_id,
            ));
        }

        let owner_nodes = self
            .nodes_by_owner
            .get(input.owner_did.as_str())
            .ok_or_else(|| DataLayerM6GraphIntegrationError::OwnerNotFound {
                owner_did: input.owner_did.clone(),
            })?;

        if !owner_nodes
            .iter()
            .any(|record| record.node_id == input.from_node_id)
        {
            if self.node_exists_outside_owner(input.owner_did.as_str(), input.from_node_id.as_str())
            {
                return Err(DataLayerM6GraphIntegrationError::OwnerScopeViolation {
                    reason_code: DATA_LAYER_M6_CROSS_OWNER_EDGE_DENIED_REASON_CODE,
                });
            }
            return Err(DataLayerM6GraphIntegrationError::NodeNotFound {
                owner_did: input.owner_did,
                node_id: input.from_node_id,
            });
        }
        if !owner_nodes
            .iter()
            .any(|record| record.node_id == input.to_node_id)
        {
            if self.node_exists_outside_owner(input.owner_did.as_str(), input.to_node_id.as_str()) {
                return Err(DataLayerM6GraphIntegrationError::OwnerScopeViolation {
                    reason_code: DATA_LAYER_M6_CROSS_OWNER_EDGE_DENIED_REASON_CODE,
                });
            }
            return Err(DataLayerM6GraphIntegrationError::NodeNotFound {
                owner_did: input.owner_did,
                node_id: input.to_node_id,
            });
        }

        let owner_edges = self
            .edges_by_owner
            .entry(input.owner_did.clone())
            .or_default();
        let record = DataLayerM6GraphEdgeRecord {
            owner_did: input.owner_did,
            edge_id: input.edge_id.clone(),
            relation: input.relation,
            from_node_id: input.from_node_id,
            to_node_id: input.to_node_id,
            weight: input.weight,
            observed_at_epoch_seconds: input.observed_at_epoch_seconds,
            sequence: owner_edges.len() as u64 + 1,
        };
        owner_edges.push(record.clone());
        self.seen_edge_ids.insert(input.edge_id);
        Ok(record)
    }

    /// Returns owner-scoped node records.
    pub fn nodes_for_owner(&self, owner_did: &str) -> Option<&[DataLayerM6GraphNodeRecord]> {
        self.nodes_by_owner.get(owner_did).map(Vec::as_slice)
    }

    /// Returns owner-scoped edge records.
    pub fn edges_for_owner(&self, owner_did: &str) -> Option<&[DataLayerM6GraphEdgeRecord]> {
        self.edges_by_owner.get(owner_did).map(Vec::as_slice)
    }

    /// Runs bounded trust propagation scoring for one owner graph.
    pub fn query_trust_propagation(
        &self,
        query: DataLayerM6TrustPropagationQuery,
    ) -> Result<Vec<DataLayerM6TrustPropagationResult>, DataLayerM6GraphIntegrationError> {
        validate_kamn_did(query.requester_owner_did.as_str())?;
        validate_kamn_did(query.owner_did.as_str())?;
        validate_non_empty(query.source_agent_node_id.as_str(), "source_agent_node_id")?;
        if query.requester_owner_did != query.owner_did {
            return Err(DataLayerM6GraphIntegrationError::OwnerScopeViolation {
                reason_code: DATA_LAYER_M6_OWNER_SCOPE_DENIED_REASON_CODE,
            });
        }
        if query.max_depth == 0 {
            return Err(DataLayerM6GraphIntegrationError::InvalidDepth(
                query.max_depth,
            ));
        }
        if !query.attenuation_factor.is_finite()
            || query.attenuation_factor <= 0.0
            || query.attenuation_factor > 1.0
        {
            return Err(DataLayerM6GraphIntegrationError::InvalidAttenuationFactor(
                query.attenuation_factor,
            ));
        }
        let limit = resolve_limit(query.limit)?;

        let owner_nodes = self
            .nodes_by_owner
            .get(query.owner_did.as_str())
            .ok_or_else(|| DataLayerM6GraphIntegrationError::OwnerNotFound {
                owner_did: query.owner_did.clone(),
            })?;
        let source_node = owner_nodes
            .iter()
            .find(|record| record.node_id == query.source_agent_node_id)
            .ok_or_else(|| {
                DataLayerM6GraphIntegrationError::InvalidSourceAgentNode(
                    query.source_agent_node_id.clone(),
                )
            })?;
        if source_node.kind != DataLayerM6GraphNodeKind::Agent {
            return Err(DataLayerM6GraphIntegrationError::InvalidSourceAgentNode(
                query.source_agent_node_id,
            ));
        }

        let owner_edges = self
            .edges_by_owner
            .get(query.owner_did.as_str())
            .map_or(&[] as &[DataLayerM6GraphEdgeRecord], Vec::as_slice);

        let mut frontier = vec![(source_node.node_id.clone(), 1.0_f32, 0_u8)];
        let mut best_scores: BTreeMap<String, (f32, u8)> = BTreeMap::new();
        for depth in 1..=query.max_depth {
            let mut next_frontier: Vec<(String, f32, u8)> = Vec::new();
            for (current_node_id, current_score, _) in frontier {
                for edge in owner_edges.iter().filter(|record| {
                    record.relation == DataLayerM6GraphEdgeRelation::Trusts
                        && record.from_node_id == current_node_id
                }) {
                    let next_score = current_score * edge.weight * query.attenuation_factor;
                    let next_hops = depth;
                    let entry = best_scores
                        .entry(edge.to_node_id.clone())
                        .or_insert((next_score, next_hops));
                    if next_score > entry.0 || (next_score == entry.0 && next_hops < entry.1) {
                        *entry = (next_score, next_hops);
                    }
                    next_frontier.push((edge.to_node_id.clone(), next_score, next_hops));
                }
            }
            frontier = next_frontier;
            if frontier.is_empty() {
                break;
            }
        }

        best_scores.remove(source_node.node_id.as_str());
        let mut results = best_scores
            .into_iter()
            .map(
                |(target_agent_node_id, (trust_score, hops))| DataLayerM6TrustPropagationResult {
                    target_agent_node_id,
                    trust_score,
                    hops,
                    reason_code: DATA_LAYER_M6_TRUST_PROPAGATION_REASON_RANKED,
                },
            )
            .collect::<Vec<_>>();

        results.sort_by(|left, right| {
            right
                .trust_score
                .total_cmp(&left.trust_score)
                .then_with(|| left.target_agent_node_id.cmp(&right.target_agent_node_id))
                .then_with(|| left.hops.cmp(&right.hops))
        });
        if results.len() > limit {
            results.truncate(limit);
        }
        Ok(results)
    }

    /// Exports deterministic portable edge projection rows.
    pub fn export_portable_edge_projection(
        &self,
        owner_did: &str,
    ) -> Result<Vec<DataLayerM6PortableEdgeProjection>, DataLayerM6GraphIntegrationError> {
        validate_kamn_did(owner_did)?;
        let owner_edges = self.edges_by_owner.get(owner_did).ok_or_else(|| {
            DataLayerM6GraphIntegrationError::OwnerNotFound {
                owner_did: owner_did.to_owned(),
            }
        })?;

        let mut projection = owner_edges
            .iter()
            .map(|edge| DataLayerM6PortableEdgeProjection {
                graph_engine_marker: DATA_LAYER_M6_GRAPH_ENGINE_APACHE_AGE,
                portability_profile: DATA_LAYER_M6_GRAPH_PORTABILITY_PROFILE,
                owner_did: edge.owner_did.clone(),
                edge_id: edge.edge_id.clone(),
                relation_marker: edge.relation.marker(),
                from_node_id: edge.from_node_id.clone(),
                to_node_id: edge.to_node_id.clone(),
                weight: edge.weight,
            })
            .collect::<Vec<_>>();
        projection.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));
        Ok(projection)
    }

    /// Exports owner-scoped portable edge projections with requester authorization.
    pub fn export_portable_edge_projection_scoped(
        &self,
        requester_owner_did: &str,
        owner_did: &str,
    ) -> Result<Vec<DataLayerM6PortableEdgeProjection>, DataLayerM6GraphIntegrationError> {
        validate_kamn_did(requester_owner_did)?;
        validate_kamn_did(owner_did)?;
        if requester_owner_did != owner_did {
            return Err(DataLayerM6GraphIntegrationError::OwnerScopeViolation {
                reason_code: DATA_LAYER_M6_OWNER_SCOPE_DENIED_REASON_CODE,
            });
        }
        self.export_portable_edge_projection(owner_did)
    }

    fn node_exists_outside_owner(&self, owner_did: &str, node_id: &str) -> bool {
        self.nodes_by_owner.iter().any(|(scope_owner, nodes)| {
            scope_owner != owner_did && nodes.iter().any(|record| record.node_id == node_id)
        })
    }
}

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

fn validate_kamn_did(value: &str) -> Result<(), DataLayerM6GraphIntegrationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.starts_with("kamn:did:") {
        return Err(DataLayerM6GraphIntegrationError::InvalidDid(
            value.to_owned(),
        ));
    }
    let segments = trimmed.split(':').collect::<Vec<_>>();
    if segments.len() < 4 || segments.iter().any(|segment| segment.is_empty()) {
        return Err(DataLayerM6GraphIntegrationError::InvalidDid(
            value.to_owned(),
        ));
    }
    Ok(())
}

fn validate_non_empty(
    value: &str,
    field_name: &'static str,
) -> Result<(), DataLayerM6GraphIntegrationError> {
    if value.trim().is_empty() {
        return Err(DataLayerM6GraphIntegrationError::EmptyField(field_name));
    }
    Ok(())
}

fn validate_weight(weight: f32) -> Result<(), DataLayerM6GraphIntegrationError> {
    if !weight.is_finite() || weight <= 0.0 || weight > 1.0 {
        return Err(DataLayerM6GraphIntegrationError::InvalidWeight(weight));
    }
    Ok(())
}

fn resolve_limit(limit: Option<usize>) -> Result<usize, DataLayerM6GraphIntegrationError> {
    let resolved = limit.unwrap_or(20);
    if resolved == 0 {
        return Err(DataLayerM6GraphIntegrationError::InvalidLimit(resolved));
    }
    Ok(resolved)
}

//! M6 graph-layer contracts for owner-scoped schema, trust propagation, and portability.
//!
//! This module models PRD M6 behavior as deterministic Rust contracts:
//! owner-scoped graph node/edge registration, bounded trust propagation ranking,
//! and portable edge projection exports suitable for AGE/openCypher adapters.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::KamnDid;

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
        let DataLayerM6GraphNodeInput {
            owner_did,
            node_id,
            kind,
            label,
        } = input;
        let owner_did = parse_kamn_did(owner_did.as_str())?;
        validate_non_empty(node_id.as_str(), "node_id")?;
        validate_non_empty(label.as_str(), "label")?;

        let owner_did_key = owner_did.as_str().to_owned();

        let owner_nodes = self
            .nodes_by_owner
            .entry(owner_did_key.clone())
            .or_default();
        if owner_nodes.iter().any(|record| record.node_id == node_id) {
            return Err(DataLayerM6GraphIntegrationError::DuplicateNodeId {
                owner_did: owner_did_key,
                node_id,
            });
        }

        let record = DataLayerM6GraphNodeRecord {
            owner_did: owner_did.as_str().to_owned(),
            node_id,
            kind,
            label,
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
        let DataLayerM6GraphEdgeInput {
            owner_did,
            edge_id,
            relation,
            from_node_id,
            to_node_id,
            weight,
            observed_at_epoch_seconds,
        } = input;
        let owner_did = parse_kamn_did(owner_did.as_str())?;
        let owner_did_key = owner_did.as_str().to_owned();
        validate_non_empty(edge_id.as_str(), "edge_id")?;
        validate_non_empty(from_node_id.as_str(), "from_node_id")?;
        validate_non_empty(to_node_id.as_str(), "to_node_id")?;
        validate_weight(weight)?;
        if observed_at_epoch_seconds == 0 {
            return Err(DataLayerM6GraphIntegrationError::EmptyField(
                "observed_at_epoch_seconds",
            ));
        }

        if self.seen_edge_ids.contains(edge_id.as_str()) {
            return Err(DataLayerM6GraphIntegrationError::DuplicateEdgeId(edge_id));
        }

        let owner_nodes = self
            .nodes_by_owner
            .get(owner_did_key.as_str())
            .ok_or_else(|| DataLayerM6GraphIntegrationError::OwnerNotFound {
                owner_did: owner_did_key.clone(),
            })?;

        if !owner_nodes
            .iter()
            .any(|record| record.node_id == from_node_id)
        {
            if self.node_exists_outside_owner(owner_did_key.as_str(), from_node_id.as_str()) {
                return Err(DataLayerM6GraphIntegrationError::OwnerScopeViolation {
                    reason_code: DATA_LAYER_M6_CROSS_OWNER_EDGE_DENIED_REASON_CODE,
                });
            }
            return Err(DataLayerM6GraphIntegrationError::NodeNotFound {
                owner_did: owner_did_key.clone(),
                node_id: from_node_id,
            });
        }
        if !owner_nodes
            .iter()
            .any(|record| record.node_id == to_node_id)
        {
            if self.node_exists_outside_owner(owner_did_key.as_str(), to_node_id.as_str()) {
                return Err(DataLayerM6GraphIntegrationError::OwnerScopeViolation {
                    reason_code: DATA_LAYER_M6_CROSS_OWNER_EDGE_DENIED_REASON_CODE,
                });
            }
            return Err(DataLayerM6GraphIntegrationError::NodeNotFound {
                owner_did: owner_did_key.clone(),
                node_id: to_node_id,
            });
        }

        let owner_edges = self
            .edges_by_owner
            .entry(owner_did_key.clone())
            .or_default();
        let record = DataLayerM6GraphEdgeRecord {
            owner_did: owner_did_key,
            edge_id: edge_id.clone(),
            relation,
            from_node_id,
            to_node_id,
            weight,
            observed_at_epoch_seconds,
            sequence: owner_edges.len() as u64 + 1,
        };
        owner_edges.push(record.clone());
        self.seen_edge_ids.insert(edge_id);
        Ok(record)
    }

    /// Returns owner-scoped node records.
    pub fn nodes_for_owner(&self, owner_did: &str) -> Option<&[DataLayerM6GraphNodeRecord]> {
        let owner_did = parse_kamn_did(owner_did).ok()?;
        self.nodes_by_owner
            .get(owner_did.as_str())
            .map(Vec::as_slice)
    }

    /// Returns owner-scoped edge records.
    pub fn edges_for_owner(&self, owner_did: &str) -> Option<&[DataLayerM6GraphEdgeRecord]> {
        let owner_did = parse_kamn_did(owner_did).ok()?;
        self.edges_by_owner
            .get(owner_did.as_str())
            .map(Vec::as_slice)
    }

    /// Runs bounded trust propagation scoring for one owner graph.
    pub fn query_trust_propagation(
        &self,
        query: DataLayerM6TrustPropagationQuery,
    ) -> Result<Vec<DataLayerM6TrustPropagationResult>, DataLayerM6GraphIntegrationError> {
        let DataLayerM6TrustPropagationQuery {
            requester_owner_did,
            owner_did,
            source_agent_node_id,
            max_depth,
            attenuation_factor,
            limit,
        } = query;
        let requester_owner_did = parse_kamn_did(requester_owner_did.as_str())?;
        let owner_did = parse_kamn_did(owner_did.as_str())?;
        validate_non_empty(source_agent_node_id.as_str(), "source_agent_node_id")?;
        if requester_owner_did.as_str() != owner_did.as_str() {
            return Err(DataLayerM6GraphIntegrationError::OwnerScopeViolation {
                reason_code: DATA_LAYER_M6_OWNER_SCOPE_DENIED_REASON_CODE,
            });
        }
        if max_depth == 0 {
            return Err(DataLayerM6GraphIntegrationError::InvalidDepth(max_depth));
        }
        if !attenuation_factor.is_finite() || attenuation_factor <= 0.0 || attenuation_factor > 1.0
        {
            return Err(DataLayerM6GraphIntegrationError::InvalidAttenuationFactor(
                attenuation_factor,
            ));
        }
        let limit = resolve_limit(limit)?;
        let owner_did = owner_did.as_str();

        let owner_nodes = self.nodes_by_owner.get(owner_did).ok_or_else(|| {
            DataLayerM6GraphIntegrationError::OwnerNotFound {
                owner_did: owner_did.to_owned(),
            }
        })?;
        let source_node = owner_nodes
            .iter()
            .find(|record| record.node_id == source_agent_node_id)
            .ok_or_else(|| {
                DataLayerM6GraphIntegrationError::InvalidSourceAgentNode(
                    source_agent_node_id.clone(),
                )
            })?;
        if source_node.kind != DataLayerM6GraphNodeKind::Agent {
            return Err(DataLayerM6GraphIntegrationError::InvalidSourceAgentNode(
                source_agent_node_id,
            ));
        }

        let owner_edges = self
            .edges_by_owner
            .get(owner_did)
            .map_or(&[] as &[DataLayerM6GraphEdgeRecord], Vec::as_slice);

        let mut frontier = vec![(source_node.node_id.clone(), 1.0_f32, 0_u8)];
        let mut best_scores: BTreeMap<String, (f32, u8)> = BTreeMap::new();
        for depth in 1..=max_depth {
            let mut next_frontier: Vec<(String, f32, u8)> = Vec::new();
            for (current_node_id, current_score, _) in frontier {
                for edge in owner_edges.iter().filter(|record| {
                    record.relation == DataLayerM6GraphEdgeRelation::Trusts
                        && record.from_node_id == current_node_id
                }) {
                    let next_score = current_score * edge.weight * attenuation_factor;
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
        let owner_did = parse_kamn_did(owner_did)?;
        let owner_did_key = owner_did.as_str();
        let owner_edges = self.edges_by_owner.get(owner_did_key).ok_or_else(|| {
            DataLayerM6GraphIntegrationError::OwnerNotFound {
                owner_did: owner_did_key.to_owned(),
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
        let requester_owner_did = parse_kamn_did(requester_owner_did)?;
        let owner_did = parse_kamn_did(owner_did)?;
        if requester_owner_did.as_str() != owner_did.as_str() {
            return Err(DataLayerM6GraphIntegrationError::OwnerScopeViolation {
                reason_code: DATA_LAYER_M6_OWNER_SCOPE_DENIED_REASON_CODE,
            });
        }
        self.export_portable_edge_projection(owner_did.as_str())
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

fn parse_kamn_did(value: &str) -> Result<KamnDid, DataLayerM6GraphIntegrationError> {
    KamnDid::parse(value)
        .map_err(|_| DataLayerM6GraphIntegrationError::InvalidDid(value.to_owned()))
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

#[cfg(test)]
mod tests {
    use super::{
        resolve_limit, validate_non_empty, validate_weight, DataLayerM6GraphEdgeInput,
        DataLayerM6GraphEdgeRelation, DataLayerM6GraphIntegrationError, DataLayerM6GraphNodeInput,
        DataLayerM6GraphNodeKind, DataLayerM6GraphRegistry, DataLayerM6TrustPropagationQuery,
        DATA_LAYER_M6_CROSS_OWNER_EDGE_DENIED_REASON_CODE, DATA_LAYER_M6_GRAPH_ENGINE_APACHE_AGE,
        DATA_LAYER_M6_GRAPH_PORTABILITY_PROFILE, DATA_LAYER_M6_OWNER_SCOPE_DENIED_REASON_CODE,
        DATA_LAYER_M6_TRUST_PROPAGATION_REASON_RANKED,
    };

    const OWNER_A: &str = "kamn:did:owner:owner-a-6031";
    const OWNER_B: &str = "kamn:did:owner:owner-b-6031";

    fn register_agent_node(
        registry: &mut DataLayerM6GraphRegistry,
        owner_did: &str,
        node_id: &str,
    ) {
        registry
            .register_node(DataLayerM6GraphNodeInput {
                owner_did: owner_did.to_owned(),
                node_id: node_id.to_owned(),
                kind: DataLayerM6GraphNodeKind::Agent,
                label: format!("label-{node_id}"),
            })
            .expect("fixture node registration must succeed");
    }

    fn edge_input(
        owner_did: &str,
        edge_id: &str,
        relation: DataLayerM6GraphEdgeRelation,
        from_node_id: &str,
        to_node_id: &str,
        weight: f32,
        observed_at_epoch_seconds: u64,
    ) -> DataLayerM6GraphEdgeInput {
        DataLayerM6GraphEdgeInput {
            owner_did: owner_did.to_owned(),
            edge_id: edge_id.to_owned(),
            relation,
            from_node_id: from_node_id.to_owned(),
            to_node_id: to_node_id.to_owned(),
            weight,
            observed_at_epoch_seconds,
        }
    }

    #[test]
    fn unit_validate_weight_rejects_non_finite_and_out_of_range_values() {
        assert_eq!(
            validate_weight(0.0),
            Err(DataLayerM6GraphIntegrationError::InvalidWeight(0.0))
        );
        assert_eq!(
            validate_weight(1.01),
            Err(DataLayerM6GraphIntegrationError::InvalidWeight(1.01))
        );
        assert!(matches!(
            validate_weight(f32::NAN),
            Err(DataLayerM6GraphIntegrationError::InvalidWeight(value)) if value.is_nan()
        ));
        assert!(validate_weight(0.75).is_ok());
    }

    #[test]
    fn unit_resolve_limit_defaults_and_rejects_zero_limit() {
        assert_eq!(resolve_limit(None), Ok(20));
        assert_eq!(resolve_limit(Some(7)), Ok(7));
        assert_eq!(
            resolve_limit(Some(0)),
            Err(DataLayerM6GraphIntegrationError::InvalidLimit(0))
        );
    }

    #[test]
    fn unit_validate_non_empty_rejects_whitespace_only_input() {
        assert_eq!(
            validate_non_empty(" \t", "node_id"),
            Err(DataLayerM6GraphIntegrationError::EmptyField("node_id"))
        );
        assert!(validate_non_empty("node-a", "node_id").is_ok());
    }

    #[test]
    fn unit_m6_graph_registry_registers_deterministic_sequences_and_sorted_projections() {
        let mut registry = DataLayerM6GraphRegistry::new();

        let source = registry
            .register_node(DataLayerM6GraphNodeInput {
                owner_did: OWNER_A.to_owned(),
                node_id: "agent-source".to_owned(),
                kind: DataLayerM6GraphNodeKind::Agent,
                label: "Agent Source".to_owned(),
            })
            .expect("source node should register");
        let target = registry
            .register_node(DataLayerM6GraphNodeInput {
                owner_did: OWNER_A.to_owned(),
                node_id: "agent-target".to_owned(),
                kind: DataLayerM6GraphNodeKind::Agent,
                label: "Agent Target".to_owned(),
            })
            .expect("target node should register");
        assert_eq!(source.sequence, 1);
        assert_eq!(target.sequence, 2);

        let edge_two = registry
            .register_edge(edge_input(
                OWNER_A,
                "edge-2",
                DataLayerM6GraphEdgeRelation::Trusts,
                "agent-source",
                "agent-target",
                0.8,
                1_701_100_001,
            ))
            .expect("first edge should register");
        let edge_one = registry
            .register_edge(edge_input(
                OWNER_A,
                "edge-1",
                DataLayerM6GraphEdgeRelation::Messaged,
                "agent-target",
                "agent-source",
                0.6,
                1_701_100_002,
            ))
            .expect("second edge should register");
        assert_eq!(edge_two.sequence, 1);
        assert_eq!(edge_one.sequence, 2);

        let nodes = registry
            .nodes_for_owner(OWNER_A)
            .expect("owner nodes should be queryable");
        assert_eq!(nodes.len(), 2);
        let edges = registry
            .edges_for_owner(OWNER_A)
            .expect("owner edges should be queryable");
        assert_eq!(edges.len(), 2);

        let projection = registry
            .export_portable_edge_projection(OWNER_A)
            .expect("portable projection should export");
        assert_eq!(
            projection
                .iter()
                .map(|row| row.edge_id.as_str())
                .collect::<Vec<_>>(),
            vec!["edge-1", "edge-2"]
        );
        assert!(projection
            .iter()
            .all(|row| row.graph_engine_marker == DATA_LAYER_M6_GRAPH_ENGINE_APACHE_AGE));
        assert!(projection
            .iter()
            .all(|row| row.portability_profile == DATA_LAYER_M6_GRAPH_PORTABILITY_PROFILE));
        assert_eq!(projection[0].relation_marker, "MESSAGED");
        assert_eq!(projection[1].relation_marker, "TRUSTS");
    }

    #[test]
    fn regression_m6_graph_registry_rejects_cross_owner_and_duplicate_edge_ids() {
        let mut registry = DataLayerM6GraphRegistry::new();
        register_agent_node(&mut registry, OWNER_A, "agent-a1");
        register_agent_node(&mut registry, OWNER_A, "agent-a2");
        register_agent_node(&mut registry, OWNER_B, "agent-b1");

        let cross_owner_edge = registry.register_edge(edge_input(
            OWNER_A,
            "edge-cross-owner",
            DataLayerM6GraphEdgeRelation::Trusts,
            "agent-a1",
            "agent-b1",
            0.9,
            1_701_100_010,
        ));
        assert!(matches!(
            cross_owner_edge,
            Err(DataLayerM6GraphIntegrationError::OwnerScopeViolation {
                reason_code: DATA_LAYER_M6_CROSS_OWNER_EDGE_DENIED_REASON_CODE,
            })
        ));

        registry
            .register_edge(edge_input(
                OWNER_A,
                "edge-dup",
                DataLayerM6GraphEdgeRelation::Trusts,
                "agent-a1",
                "agent-a2",
                0.7,
                1_701_100_020,
            ))
            .expect("first edge-dup registration should succeed");
        assert_eq!(
            registry.register_edge(edge_input(
                OWNER_A,
                "edge-dup",
                DataLayerM6GraphEdgeRelation::Messaged,
                "agent-a2",
                "agent-a1",
                0.65,
                1_701_100_021,
            )),
            Err(DataLayerM6GraphIntegrationError::DuplicateEdgeId(
                "edge-dup".to_owned()
            ))
        );

        assert!(matches!(
            registry.export_portable_edge_projection_scoped(OWNER_B, OWNER_A),
            Err(DataLayerM6GraphIntegrationError::OwnerScopeViolation {
                reason_code: DATA_LAYER_M6_OWNER_SCOPE_DENIED_REASON_CODE,
            })
        ));
    }

    #[test]
    fn unit_m6_graph_registry_trust_propagation_is_ranked_and_limited() {
        let mut registry = DataLayerM6GraphRegistry::new();
        register_agent_node(&mut registry, OWNER_A, "agent-source");
        register_agent_node(&mut registry, OWNER_A, "agent-a");
        register_agent_node(&mut registry, OWNER_A, "agent-b");
        register_agent_node(&mut registry, OWNER_A, "agent-c");

        registry
            .register_edge(edge_input(
                OWNER_A,
                "trust-sa",
                DataLayerM6GraphEdgeRelation::Trusts,
                "agent-source",
                "agent-a",
                0.9,
                1_701_100_100,
            ))
            .expect("source->a trust edge should register");
        registry
            .register_edge(edge_input(
                OWNER_A,
                "trust-sb",
                DataLayerM6GraphEdgeRelation::Trusts,
                "agent-source",
                "agent-b",
                0.7,
                1_701_100_101,
            ))
            .expect("source->b trust edge should register");
        registry
            .register_edge(edge_input(
                OWNER_A,
                "trust-ac",
                DataLayerM6GraphEdgeRelation::Trusts,
                "agent-a",
                "agent-c",
                1.0,
                1_701_100_102,
            ))
            .expect("a->c trust edge should register");

        let results = registry
            .query_trust_propagation(DataLayerM6TrustPropagationQuery {
                requester_owner_did: OWNER_A.to_owned(),
                owner_did: OWNER_A.to_owned(),
                source_agent_node_id: "agent-source".to_owned(),
                max_depth: 3,
                attenuation_factor: 0.8,
                limit: Some(2),
            })
            .expect("trust propagation should succeed for valid graph");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].target_agent_node_id, "agent-a");
        assert!((results[0].trust_score - 0.72).abs() < 0.000_001);
        assert_eq!(results[0].hops, 1);
        assert_eq!(
            results[0].reason_code,
            DATA_LAYER_M6_TRUST_PROPAGATION_REASON_RANKED
        );
        assert_eq!(results[1].target_agent_node_id, "agent-c");
        assert!((results[1].trust_score - 0.576).abs() < 0.000_001);
        assert_eq!(results[1].hops, 2);
        assert_eq!(
            results[1].reason_code,
            DATA_LAYER_M6_TRUST_PROPAGATION_REASON_RANKED
        );
        assert!(results
            .iter()
            .all(|row| row.target_agent_node_id != "agent-b"));
    }
}

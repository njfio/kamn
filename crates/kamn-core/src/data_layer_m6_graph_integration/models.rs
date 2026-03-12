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
    pub(super) fn marker(self) -> &'static str {
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

impl fmt::Display for DataLayerM6GraphEdgeRelation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.marker())
    }
}

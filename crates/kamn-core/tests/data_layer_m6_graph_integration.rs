use kamn_core::{
    DataLayerM6GraphEdgeInput, DataLayerM6GraphEdgeRelation, DataLayerM6GraphIntegrationError,
    DataLayerM6GraphNodeInput, DataLayerM6GraphNodeKind, DataLayerM6GraphRegistry,
    DataLayerM6TrustPropagationQuery, DATA_LAYER_M6_CROSS_OWNER_EDGE_DENIED_REASON_CODE,
    DATA_LAYER_M6_OWNER_SCOPE_DENIED_REASON_CODE,
};

fn node_input(
    owner_did: &str,
    node_id: &str,
    kind: DataLayerM6GraphNodeKind,
) -> DataLayerM6GraphNodeInput {
    DataLayerM6GraphNodeInput {
        owner_did: owner_did.to_owned(),
        node_id: node_id.to_owned(),
        kind,
        label: node_id.to_owned(),
    }
}

fn edge_input(
    owner_did: &str,
    edge_id: &str,
    relation: DataLayerM6GraphEdgeRelation,
    from_node_id: &str,
    to_node_id: &str,
    weight: f32,
) -> DataLayerM6GraphEdgeInput {
    DataLayerM6GraphEdgeInput {
        owner_did: owner_did.to_owned(),
        edge_id: edge_id.to_owned(),
        relation,
        from_node_id: from_node_id.to_owned(),
        to_node_id: to_node_id.to_owned(),
        weight,
        observed_at_epoch_seconds: 1_708_400_000,
    }
}

#[test]
fn spec_c01_graph_registry_accepts_owner_scoped_node_and_edge_contracts() {
    let mut registry = DataLayerM6GraphRegistry::new();
    registry
        .register_node(node_input(
            "kamn:did:owner:alpha",
            "agent-alpha-a",
            DataLayerM6GraphNodeKind::Agent,
        ))
        .expect("agent node A should register");
    registry
        .register_node(node_input(
            "kamn:did:owner:alpha",
            "agent-alpha-b",
            DataLayerM6GraphNodeKind::Agent,
        ))
        .expect("agent node B should register");
    registry
        .register_edge(edge_input(
            "kamn:did:owner:alpha",
            "edge-alpha-trust-1",
            DataLayerM6GraphEdgeRelation::Trusts,
            "agent-alpha-a",
            "agent-alpha-b",
            0.9,
        ))
        .expect("trust edge should register");

    let nodes = registry
        .nodes_for_owner("kamn:did:owner:alpha")
        .expect("owner graph nodes should exist");
    let edges = registry
        .edges_for_owner("kamn:did:owner:alpha")
        .expect("owner graph edges should exist");
    assert_eq!(nodes.len(), 2);
    assert_eq!(edges.len(), 1);
}

#[test]
fn spec_c02_cross_owner_graph_edge_registration_is_denied_fail_closed() {
    let mut registry = DataLayerM6GraphRegistry::new();
    registry
        .register_node(node_input(
            "kamn:did:owner:alpha",
            "agent-alpha-a",
            DataLayerM6GraphNodeKind::Agent,
        ))
        .expect("owner alpha node should register");
    registry
        .register_node(node_input(
            "kamn:did:owner:beta",
            "agent-beta-a",
            DataLayerM6GraphNodeKind::Agent,
        ))
        .expect("owner beta node should register");

    let cross_owner = registry.register_edge(edge_input(
        "kamn:did:owner:alpha",
        "edge-alpha-cross-owner",
        DataLayerM6GraphEdgeRelation::Trusts,
        "agent-alpha-a",
        "agent-beta-a",
        0.7,
    ));
    assert!(matches!(
        cross_owner,
        Err(DataLayerM6GraphIntegrationError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M6_CROSS_OWNER_EDGE_DENIED_REASON_CODE,
        })
    ));
}

#[test]
fn spec_c03_trust_propagation_returns_deterministic_ranked_results() {
    let mut registry = DataLayerM6GraphRegistry::new();
    for node_id in ["agent-a", "agent-b", "agent-c", "agent-d"] {
        registry
            .register_node(node_input(
                "kamn:did:owner:alpha",
                node_id,
                DataLayerM6GraphNodeKind::Agent,
            ))
            .expect("agent node should register");
    }
    registry
        .register_edge(edge_input(
            "kamn:did:owner:alpha",
            "e-ab",
            DataLayerM6GraphEdgeRelation::Trusts,
            "agent-a",
            "agent-b",
            0.9,
        ))
        .expect("edge AB should register");
    registry
        .register_edge(edge_input(
            "kamn:did:owner:alpha",
            "e-bc",
            DataLayerM6GraphEdgeRelation::Trusts,
            "agent-b",
            "agent-c",
            0.8,
        ))
        .expect("edge BC should register");
    registry
        .register_edge(edge_input(
            "kamn:did:owner:alpha",
            "e-ad",
            DataLayerM6GraphEdgeRelation::Trusts,
            "agent-a",
            "agent-d",
            0.5,
        ))
        .expect("edge AD should register");

    let ranked = registry
        .query_trust_propagation(DataLayerM6TrustPropagationQuery {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            source_agent_node_id: "agent-a".to_owned(),
            max_depth: 2,
            attenuation_factor: 0.85,
            limit: Some(3),
        })
        .expect("trust propagation query should succeed");
    assert_eq!(ranked.len(), 3);
    assert_eq!(ranked[0].target_agent_node_id, "agent-b");
    assert_eq!(ranked[1].target_agent_node_id, "agent-c");
    assert_eq!(ranked[2].target_agent_node_id, "agent-d");
}

#[test]
fn spec_c04_portability_projection_is_deterministic_and_complete() {
    let mut registry = DataLayerM6GraphRegistry::new();
    registry
        .register_node(node_input(
            "kamn:did:owner:alpha",
            "agent-a",
            DataLayerM6GraphNodeKind::Agent,
        ))
        .expect("agent A should register");
    registry
        .register_node(node_input(
            "kamn:did:owner:alpha",
            "agent-b",
            DataLayerM6GraphNodeKind::Agent,
        ))
        .expect("agent B should register");
    registry
        .register_edge(edge_input(
            "kamn:did:owner:alpha",
            "edge-1",
            DataLayerM6GraphEdgeRelation::Messaged,
            "agent-a",
            "agent-b",
            1.0,
        ))
        .expect("edge 1 should register");
    registry
        .register_edge(edge_input(
            "kamn:did:owner:alpha",
            "edge-2",
            DataLayerM6GraphEdgeRelation::Trusts,
            "agent-b",
            "agent-a",
            0.7,
        ))
        .expect("edge 2 should register");

    let projection = registry
        .export_portable_edge_projection("kamn:did:owner:alpha")
        .expect("projection should succeed");
    assert_eq!(projection.len(), 2);
    assert_eq!(projection[0].edge_id, "edge-1");
    assert_eq!(projection[1].edge_id, "edge-2");
    assert_eq!(projection[0].relation_marker, "MESSAGED");
    assert_eq!(projection[1].relation_marker, "TRUSTS");
}

#[test]
fn spec_c05_trust_propagation_denies_requester_outside_owner_scope() {
    let mut registry = DataLayerM6GraphRegistry::new();
    registry
        .register_node(node_input(
            "kamn:did:owner:alpha",
            "agent-a",
            DataLayerM6GraphNodeKind::Agent,
        ))
        .expect("agent A should register");
    registry
        .register_node(node_input(
            "kamn:did:owner:alpha",
            "agent-b",
            DataLayerM6GraphNodeKind::Agent,
        ))
        .expect("agent B should register");
    registry
        .register_edge(edge_input(
            "kamn:did:owner:alpha",
            "e-ab",
            DataLayerM6GraphEdgeRelation::Trusts,
            "agent-a",
            "agent-b",
            0.6,
        ))
        .expect("edge AB should register");

    let denied = registry.query_trust_propagation(DataLayerM6TrustPropagationQuery {
        requester_owner_did: "kamn:did:owner:intruder".to_owned(),
        owner_did: "kamn:did:owner:alpha".to_owned(),
        source_agent_node_id: "agent-a".to_owned(),
        max_depth: 2,
        attenuation_factor: 0.85,
        limit: Some(3),
    });
    assert!(matches!(
        denied,
        Err(DataLayerM6GraphIntegrationError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M6_OWNER_SCOPE_DENIED_REASON_CODE,
        })
    ));
}

#[test]
fn spec_c06_scoped_portability_projection_denies_requester_outside_owner_scope() {
    let mut registry = DataLayerM6GraphRegistry::new();
    registry
        .register_node(node_input(
            "kamn:did:owner:alpha",
            "agent-a",
            DataLayerM6GraphNodeKind::Agent,
        ))
        .expect("agent A should register");
    registry
        .register_node(node_input(
            "kamn:did:owner:alpha",
            "agent-b",
            DataLayerM6GraphNodeKind::Agent,
        ))
        .expect("agent B should register");
    registry
        .register_edge(edge_input(
            "kamn:did:owner:alpha",
            "edge-alpha-1",
            DataLayerM6GraphEdgeRelation::Trusts,
            "agent-a",
            "agent-b",
            0.8,
        ))
        .expect("edge should register");

    let denied = registry
        .export_portable_edge_projection_scoped("kamn:did:owner:intruder", "kamn:did:owner:alpha");
    assert!(matches!(
        denied,
        Err(DataLayerM6GraphIntegrationError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M6_OWNER_SCOPE_DENIED_REASON_CODE,
        })
    ));
}

#[test]
fn spec_c07_scoped_portability_projection_matches_authorized_owner_projection() {
    let mut registry = DataLayerM6GraphRegistry::new();
    registry
        .register_node(node_input(
            "kamn:did:owner:alpha",
            "agent-a",
            DataLayerM6GraphNodeKind::Agent,
        ))
        .expect("agent A should register");
    registry
        .register_node(node_input(
            "kamn:did:owner:alpha",
            "agent-b",
            DataLayerM6GraphNodeKind::Agent,
        ))
        .expect("agent B should register");
    registry
        .register_edge(edge_input(
            "kamn:did:owner:alpha",
            "edge-alpha-1",
            DataLayerM6GraphEdgeRelation::Trusts,
            "agent-a",
            "agent-b",
            0.8,
        ))
        .expect("edge should register");
    registry
        .register_edge(edge_input(
            "kamn:did:owner:alpha",
            "edge-alpha-2",
            DataLayerM6GraphEdgeRelation::Messaged,
            "agent-b",
            "agent-a",
            1.0,
        ))
        .expect("edge should register");

    let owner_projection = registry
        .export_portable_edge_projection("kamn:did:owner:alpha")
        .expect("owner projection should succeed");
    let scoped_projection = registry
        .export_portable_edge_projection_scoped("kamn:did:owner:alpha", "kamn:did:owner:alpha")
        .expect("scoped projection should succeed");

    assert_eq!(scoped_projection, owner_projection);
}

use super::*;

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
    assert!(
        projection
            .iter()
            .all(|row| row.graph_engine_marker == DATA_LAYER_M6_GRAPH_ENGINE_APACHE_AGE)
    );
    assert!(
        projection
            .iter()
            .all(|row| row.portability_profile == DATA_LAYER_M6_GRAPH_PORTABILITY_PROFILE)
    );
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

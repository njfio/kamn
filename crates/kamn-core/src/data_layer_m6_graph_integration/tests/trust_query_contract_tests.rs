use super::*;
use crate::DataLayerM6TrustPropagationQuery;

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

use super::{
    resolve_limit, validate_non_empty, validate_weight, DataLayerM6GraphEdgeInput,
    DataLayerM6GraphEdgeRelation, DataLayerM6GraphIntegrationError, DataLayerM6GraphNodeInput,
    DataLayerM6GraphNodeKind, DataLayerM6GraphRegistry,
    DATA_LAYER_M6_CROSS_OWNER_EDGE_DENIED_REASON_CODE, DATA_LAYER_M6_GRAPH_ENGINE_APACHE_AGE,
    DATA_LAYER_M6_GRAPH_PORTABILITY_PROFILE, DATA_LAYER_M6_OWNER_SCOPE_DENIED_REASON_CODE,
    DATA_LAYER_M6_TRUST_PROPAGATION_REASON_RANKED,
};

const OWNER_A: &str = "kamn:did:owner:owner-a-6031";
const OWNER_B: &str = "kamn:did:owner:owner-b-6031";

fn register_agent_node(registry: &mut DataLayerM6GraphRegistry, owner_did: &str, node_id: &str) {
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

mod registry_contract_tests;
mod trust_query_contract_tests;
mod validation_contract_tests;

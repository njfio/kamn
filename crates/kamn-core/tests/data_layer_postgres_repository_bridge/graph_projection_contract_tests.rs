use crate::support::{
    data_layer_pg_project_m6_age_edge_upsert_operation,
    data_layer_pg_project_m6_age_trust_query_operation, fixture_m6_edge_record,
    DataLayerM6GraphEdgeRecord, DataLayerM6GraphEdgeRelation, DataLayerM6TrustPropagationQuery,
    DataLayerPgM6AgeConfig, DataLayerPgM6AgeTrustQueryRequest, DataLayerPgOperationKind,
    DataLayerPgRepositoryBridgeError, DATA_LAYER_PG_AGE_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_AGE_RELATION_UNSUPPORTED_REASON_CODE,
};

macro_rules! assert_upsert_descriptor {
    ($descriptor:expr) => {
        assert_eq!($descriptor.kind, DataLayerPgOperationKind::UpsertGraphEdge);
        assert!($descriptor.sql.contains("SELECT * FROM cypher"));
        assert_upsert_bind_markers(&$descriptor.bind_markers);
    };
}

macro_rules! assert_trust_query_descriptor {
    ($descriptor:expr) => {
        assert_eq!(
            $descriptor.kind,
            DataLayerPgOperationKind::QueryGraphTrustPropagation
        );
        assert!($descriptor
            .sql
            .contains("MATCH (source:Agent {node_id: $2, owner_did: $1})"));
        assert_eq!(
            $descriptor.bind_markers,
            vec!["owner_did", "source_agent_node_id", "max_depth", "limit"]
        );
    };
}

#[test]
fn spec_c07_m6_age_projection_is_deterministic_for_edge_upsert_and_trust_query() {
    let edge = fixture_m6_edge_record();
    let upsert_descriptor = data_layer_pg_project_m6_age_edge_upsert_operation(
        &edge,
        "kamn:did:agent:agent-1",
        enabled_age_config(),
    )
    .expect("valid m6 edge should project AGE upsert descriptor");
    assert_upsert_descriptor!(upsert_descriptor);

    let trust_query_descriptor = data_layer_pg_project_m6_age_trust_query_operation(
        trust_query_request(),
        enabled_age_config(),
    )
    .expect("valid trust query should project AGE query descriptor");
    assert_trust_query_descriptor!(trust_query_descriptor);
}

#[test]
fn spec_c08_m6_age_projection_fails_closed_for_extension_and_relation_mismatch() {
    let edge = fixture_m6_edge_record();
    let extension_error = data_layer_pg_project_m6_age_edge_upsert_operation(
        &edge,
        "kamn:did:agent:agent-1",
        disabled_age_config(),
    )
    .expect_err("disabled AGE extension should fail closed");
    assert_age_extension_error(extension_error);

    let unsupported_relation_error = data_layer_pg_project_m6_age_edge_upsert_operation(
        &unsupported_relation_edge(edge),
        "kamn:did:agent:agent-1",
        enabled_age_config(),
    )
    .expect_err("unsupported relation projection should fail closed");
    assert_unsupported_relation_error(unsupported_relation_error);
}

fn trust_query_request() -> DataLayerPgM6AgeTrustQueryRequest {
    DataLayerPgM6AgeTrustQueryRequest {
        requester_did: "kamn:did:agent:agent-1".to_owned(),
        query: DataLayerM6TrustPropagationQuery {
            requester_owner_did: "kamn:did:owner:owner-1".to_owned(),
            owner_did: "kamn:did:owner:owner-1".to_owned(),
            source_agent_node_id: "agent-a".to_owned(),
            max_depth: 2,
            attenuation_factor: 0.85,
            limit: Some(10),
        },
    }
}

fn assert_upsert_bind_markers(markers: &[&str]) {
    assert_eq!(
        markers,
        [
            "owner_did",
            "edge_id",
            "relation_marker",
            "from_node_id",
            "to_node_id",
            "weight",
            "observed_at_epoch_seconds",
        ]
    );
}

fn unsupported_relation_edge(edge: DataLayerM6GraphEdgeRecord) -> DataLayerM6GraphEdgeRecord {
    DataLayerM6GraphEdgeRecord {
        relation: DataLayerM6GraphEdgeRelation::Messaged,
        ..edge
    }
}

fn disabled_age_config() -> DataLayerPgM6AgeConfig {
    DataLayerPgM6AgeConfig::new(false, "kamn_graph").expect("disabled AGE config should construct")
}

fn enabled_age_config() -> DataLayerPgM6AgeConfig {
    DataLayerPgM6AgeConfig::new(true, "kamn_graph").expect("enabled AGE config should construct")
}

fn assert_age_extension_error(error: DataLayerPgRepositoryBridgeError) {
    match error {
        DataLayerPgRepositoryBridgeError::AgeExtensionUnavailable { reason_code } => {
            assert_eq!(
                reason_code,
                DATA_LAYER_PG_AGE_EXTENSION_UNAVAILABLE_REASON_CODE
            );
        }
        other => panic!("unexpected extension error variant: {other:?}"),
    }
}

fn assert_unsupported_relation_error(error: DataLayerPgRepositoryBridgeError) {
    match error {
        DataLayerPgRepositoryBridgeError::AgeUnsupportedRelation { reason_code, .. } => {
            assert_eq!(
                reason_code,
                DATA_LAYER_PG_AGE_RELATION_UNSUPPORTED_REASON_CODE
            );
        }
        other => panic!("unexpected relation error variant: {other:?}"),
    }
}

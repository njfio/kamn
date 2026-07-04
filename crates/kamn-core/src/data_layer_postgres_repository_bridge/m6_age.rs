use crate::{DataLayerM6GraphEdgeRecord, DataLayerPgRepositoryBridgeError};

use super::{
    build_requester_session, map_age_supported_relation, validate_age_config, validate_non_empty,
    validate_owner_did, DataLayerPgM6AgeConfig, DataLayerPgM6AgeTrustQueryRequest,
    DataLayerPgOperationKind, DataLayerPgSqlOperation, DATA_LAYER_PG_MAX_AGE_QUERY_LIMIT,
};

/// Runs the data layer pg project m6 age edge upsert operation contract helper.
pub fn data_layer_pg_project_m6_age_edge_upsert_operation(
    edge: &DataLayerM6GraphEdgeRecord,
    requester_did: &str,
    config: DataLayerPgM6AgeConfig,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_age_config(&config)?;
    validate_edge(edge)?;
    build_edge_upsert_operation(
        requester_did,
        config.graph_name,
        map_age_supported_relation(edge.relation)?,
    )
}

/// Runs the data layer pg project m6 age trust query operation contract helper.
pub fn data_layer_pg_project_m6_age_trust_query_operation(
    request: DataLayerPgM6AgeTrustQueryRequest,
    config: DataLayerPgM6AgeConfig,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_age_config(&config)?;
    validate_trust_query(&request)?;
    let limit = resolve_age_limit(request.query.limit)?;
    build_trust_query_operation(request.requester_did.as_str(), config.graph_name, limit)
}

fn validate_edge(
    edge: &DataLayerM6GraphEdgeRecord,
) -> Result<(), DataLayerPgRepositoryBridgeError> {
    validate_non_empty(edge.owner_did.as_str(), "owner_did")?;
    validate_non_empty(edge.edge_id.as_str(), "edge_id")?;
    validate_non_empty(edge.from_node_id.as_str(), "from_node_id")?;
    validate_non_empty(edge.to_node_id.as_str(), "to_node_id")?;
    validate_owner_did(edge.owner_did.as_str())
}

fn build_edge_upsert_operation(
    requester_did: &str,
    graph_name: String,
    relation_marker: &'static str,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::UpsertGraphEdge,
        sql: edge_upsert_sql(&graph_name, relation_marker),
        bind_markers: edge_upsert_bind_markers(),
        session: build_requester_session(requester_did)?,
    })
}

fn edge_upsert_sql(graph_name: &str, relation_marker: &str) -> String {
    format!(
        "SELECT * FROM cypher('{graph_name}', $$ MERGE (from:Agent {{node_id: $4, owner_did: $1}}) MERGE (to:Agent {{node_id: $5, owner_did: $1}}) MERGE (from)-[r:{relation_marker} {{edge_id: $2, owner_did: $1}}]->(to) SET r.weight = $6, r.observed_at_epoch_seconds = $7 RETURN r.edge_id $$) AS (edge_id agtype);"
    )
}

fn edge_upsert_bind_markers() -> Vec<&'static str> {
    vec![
        "owner_did",
        "edge_id",
        "relation_marker",
        "from_node_id",
        "to_node_id",
        "weight",
        "observed_at_epoch_seconds",
    ]
}

fn validate_trust_query(
    request: &DataLayerPgM6AgeTrustQueryRequest,
) -> Result<(), DataLayerPgRepositoryBridgeError> {
    validate_non_empty(request.query.owner_did.as_str(), "owner_did")?;
    validate_non_empty(
        request.query.source_agent_node_id.as_str(),
        "source_agent_node_id",
    )?;
    validate_owner_did(request.query.owner_did.as_str())?;
    validate_owner_did(request.query.requester_owner_did.as_str())?;
    if request.query.max_depth == 0 {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField("max_depth"));
    }
    Ok(())
}

fn resolve_age_limit(limit: Option<usize>) -> Result<usize, DataLayerPgRepositoryBridgeError> {
    let limit = limit.unwrap_or(DATA_LAYER_PG_MAX_AGE_QUERY_LIMIT);
    if limit == 0 || limit > DATA_LAYER_PG_MAX_AGE_QUERY_LIMIT {
        return Err(DataLayerPgRepositoryBridgeError::InvalidSearchLimit {
            requested: limit as u32,
            max_allowed: DATA_LAYER_PG_MAX_AGE_QUERY_LIMIT as u32,
        });
    }
    Ok(limit)
}

fn build_trust_query_operation(
    requester_did: &str,
    graph_name: String,
    _limit: usize,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::QueryGraphTrustPropagation,
        sql: trust_query_sql(&graph_name),
        bind_markers: vec!["owner_did", "source_agent_node_id", "max_depth", "limit"],
        session: build_requester_session(requester_did)?,
    })
}

fn trust_query_sql(graph_name: &str) -> String {
    format!(
        "SELECT * FROM cypher('{graph_name}', $$ MATCH (source:Agent {{node_id: $2, owner_did: $1}})-[:TRUSTS*1..$3]->(target:Agent {{owner_did: $1}}) RETURN target.node_id AS target_agent_node_id $$) AS (target_agent_node_id agtype) LIMIT $4;"
    )
}

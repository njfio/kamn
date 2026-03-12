use super::{
    DATA_LAYER_M6_OWNER_SCOPE_DENIED_REASON_CODE, DATA_LAYER_M6_TRUST_PROPAGATION_REASON_RANKED,
    DataLayerM6GraphEdgeRecord, DataLayerM6GraphEdgeRelation, DataLayerM6GraphIntegrationError,
    DataLayerM6GraphNodeKind, DataLayerM6GraphRegistry, DataLayerM6TrustPropagationQuery,
    DataLayerM6TrustPropagationResult,
    support::{parse_kamn_did, resolve_limit, validate_non_empty},
};
use std::collections::BTreeMap;

impl DataLayerM6GraphRegistry {
    /// Runs bounded trust propagation scoring for one owner graph.
    pub fn query_trust_propagation(
        &self,
        query: DataLayerM6TrustPropagationQuery,
    ) -> Result<Vec<DataLayerM6TrustPropagationResult>, DataLayerM6GraphIntegrationError> {
        let query = validate_query(query)?;
        let owner_did = query.owner_did.as_str();
        let source_node_id =
            self.source_agent_node(owner_did, query.source_agent_node_id.as_str())?;
        let mut best_scores = propagate_trust_scores(
            self.owner_edges(owner_did),
            source_node_id.as_str(),
            query.max_depth,
            query.attenuation_factor,
        );
        best_scores.remove(source_node_id.as_str());
        Ok(sorted_results(best_scores, query.limit.unwrap_or(20)))
    }

    fn owner_nodes(
        &self,
        owner_did: &str,
    ) -> Result<&[super::DataLayerM6GraphNodeRecord], DataLayerM6GraphIntegrationError> {
        self.nodes_by_owner
            .get(owner_did)
            .map(Vec::as_slice)
            .ok_or_else(|| DataLayerM6GraphIntegrationError::OwnerNotFound {
                owner_did: owner_did.to_owned(),
            })
    }

    fn owner_edges(&self, owner_did: &str) -> &[DataLayerM6GraphEdgeRecord] {
        self.edges_by_owner
            .get(owner_did)
            .map_or(&[] as &[DataLayerM6GraphEdgeRecord], Vec::as_slice)
    }

    fn source_agent_node(
        &self,
        owner_did: &str,
        source_agent_node_id: &str,
    ) -> Result<String, DataLayerM6GraphIntegrationError> {
        let source_node = self
            .owner_nodes(owner_did)?
            .iter()
            .find(|record| record.node_id == source_agent_node_id)
            .ok_or_else(|| {
                DataLayerM6GraphIntegrationError::InvalidSourceAgentNode(
                    source_agent_node_id.to_owned(),
                )
            })?;
        if source_node.kind != DataLayerM6GraphNodeKind::Agent {
            return Err(DataLayerM6GraphIntegrationError::InvalidSourceAgentNode(
                source_agent_node_id.to_owned(),
            ));
        }
        Ok(source_node.node_id.clone())
    }
}

fn validate_query(
    query: DataLayerM6TrustPropagationQuery,
) -> Result<DataLayerM6TrustPropagationQuery, DataLayerM6GraphIntegrationError> {
    let requester_owner_did = parse_kamn_did(query.requester_owner_did.as_str())?;
    let owner_did = parse_kamn_did(query.owner_did.as_str())?;
    validate_non_empty(query.source_agent_node_id.as_str(), "source_agent_node_id")?;
    if requester_owner_did.as_str() != owner_did.as_str() {
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
    Ok(DataLayerM6TrustPropagationQuery {
        requester_owner_did: requester_owner_did.as_str().to_owned(),
        owner_did: owner_did.as_str().to_owned(),
        source_agent_node_id: query.source_agent_node_id,
        max_depth: query.max_depth,
        attenuation_factor: query.attenuation_factor,
        limit: Some(resolve_limit(query.limit)?),
    })
}

fn propagate_trust_scores(
    owner_edges: &[DataLayerM6GraphEdgeRecord],
    source_node_id: &str,
    max_depth: u8,
    attenuation_factor: f32,
) -> BTreeMap<String, (f32, u8)> {
    let mut frontier = vec![(source_node_id.to_owned(), 1.0_f32, 0_u8)];
    let mut best_scores: BTreeMap<String, (f32, u8)> = BTreeMap::new();
    for depth in 1..=max_depth {
        let mut next_frontier = Vec::new();
        for (current_node_id, current_score, _) in frontier {
            for edge in trust_edges_from(owner_edges, current_node_id.as_str()) {
                update_best_scores(
                    &mut best_scores,
                    &mut next_frontier,
                    edge,
                    current_score,
                    attenuation_factor,
                    depth,
                );
            }
        }
        if next_frontier.is_empty() {
            break;
        }
        frontier = next_frontier;
    }
    best_scores
}

fn trust_edges_from<'a>(
    owner_edges: &'a [DataLayerM6GraphEdgeRecord],
    current_node_id: &str,
) -> Vec<&'a DataLayerM6GraphEdgeRecord> {
    owner_edges
        .iter()
        .filter(|record| {
            record.relation == DataLayerM6GraphEdgeRelation::Trusts
                && record.from_node_id == current_node_id
        })
        .collect()
}

fn update_best_scores(
    best_scores: &mut BTreeMap<String, (f32, u8)>,
    next_frontier: &mut Vec<(String, f32, u8)>,
    edge: &DataLayerM6GraphEdgeRecord,
    current_score: f32,
    attenuation_factor: f32,
    depth: u8,
) {
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

fn sorted_results(
    best_scores: BTreeMap<String, (f32, u8)>,
    limit: usize,
) -> Vec<DataLayerM6TrustPropagationResult> {
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
    results
}

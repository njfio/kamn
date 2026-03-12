use super::{
    DATA_LAYER_M6_GRAPH_ENGINE_APACHE_AGE, DATA_LAYER_M6_GRAPH_PORTABILITY_PROFILE,
    DATA_LAYER_M6_OWNER_SCOPE_DENIED_REASON_CODE, DataLayerM6GraphIntegrationError,
    DataLayerM6GraphRegistry, DataLayerM6PortableEdgeProjection, support::parse_kamn_did,
};

impl DataLayerM6GraphRegistry {
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
}

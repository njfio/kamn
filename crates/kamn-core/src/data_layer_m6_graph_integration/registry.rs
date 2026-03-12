mod edge_registration;

use super::{
    DATA_LAYER_M6_CROSS_OWNER_EDGE_DENIED_REASON_CODE, DataLayerM6GraphEdgeInput,
    DataLayerM6GraphEdgeRecord, DataLayerM6GraphIntegrationError, DataLayerM6GraphNodeInput,
    DataLayerM6GraphNodeRecord,
    support::{parse_kamn_did, validate_non_empty},
};
use std::collections::{BTreeMap, BTreeSet};

use self::edge_registration::normalize_edge_input;

/// M6 owner-scoped graph registry and trust propagation service.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DataLayerM6GraphRegistry {
    pub(super) nodes_by_owner: BTreeMap<String, Vec<DataLayerM6GraphNodeRecord>>,
    pub(super) edges_by_owner: BTreeMap<String, Vec<DataLayerM6GraphEdgeRecord>>,
    pub(super) seen_edge_ids: BTreeSet<String>,
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
        let edge = self.validate_edge_input(input)?;
        let owner_nodes = self.owner_nodes_for_edge(edge.owner_did.as_str())?;
        self.ensure_owner_edge_nodes(owner_nodes, &edge)?;
        Ok(self.store_edge_record(edge))
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

    pub(super) fn node_exists_outside_owner(&self, owner_did: &str, node_id: &str) -> bool {
        self.nodes_by_owner.iter().any(|(scope_owner, nodes)| {
            scope_owner != owner_did && nodes.iter().any(|record| record.node_id == node_id)
        })
    }

    fn ensure_owner_scoped_node(
        &self,
        owner_did: &str,
        owner_nodes: &[DataLayerM6GraphNodeRecord],
        node_id: &str,
    ) -> Result<(), DataLayerM6GraphIntegrationError> {
        if owner_nodes.iter().any(|record| record.node_id == node_id) {
            return Ok(());
        }
        if self.node_exists_outside_owner(owner_did, node_id) {
            return Err(DataLayerM6GraphIntegrationError::OwnerScopeViolation {
                reason_code: DATA_LAYER_M6_CROSS_OWNER_EDGE_DENIED_REASON_CODE,
            });
        }
        Err(DataLayerM6GraphIntegrationError::NodeNotFound {
            owner_did: owner_did.to_owned(),
            node_id: node_id.to_owned(),
        })
    }

    fn validate_edge_input(
        &self,
        input: DataLayerM6GraphEdgeInput,
    ) -> Result<DataLayerM6GraphEdgeInput, DataLayerM6GraphIntegrationError> {
        let edge = normalize_edge_input(input)?;
        if self.seen_edge_ids.contains(edge.edge_id.as_str()) {
            return Err(DataLayerM6GraphIntegrationError::DuplicateEdgeId(
                edge.edge_id,
            ));
        }
        Ok(edge)
    }

    fn owner_nodes_for_edge(
        &self,
        owner_did: &str,
    ) -> Result<&[DataLayerM6GraphNodeRecord], DataLayerM6GraphIntegrationError> {
        self.nodes_by_owner
            .get(owner_did)
            .map(Vec::as_slice)
            .ok_or_else(|| DataLayerM6GraphIntegrationError::OwnerNotFound {
                owner_did: owner_did.to_owned(),
            })
    }

    fn ensure_owner_edge_nodes(
        &self,
        owner_nodes: &[DataLayerM6GraphNodeRecord],
        edge: &DataLayerM6GraphEdgeInput,
    ) -> Result<(), DataLayerM6GraphIntegrationError> {
        self.ensure_owner_scoped_node(
            edge.owner_did.as_str(),
            owner_nodes,
            edge.from_node_id.as_str(),
        )?;
        self.ensure_owner_scoped_node(
            edge.owner_did.as_str(),
            owner_nodes,
            edge.to_node_id.as_str(),
        )
    }

    fn store_edge_record(&mut self, edge: DataLayerM6GraphEdgeInput) -> DataLayerM6GraphEdgeRecord {
        let owner_edges = self
            .edges_by_owner
            .entry(edge.owner_did.clone())
            .or_default();
        let record = DataLayerM6GraphEdgeRecord {
            owner_did: edge.owner_did.clone(),
            edge_id: edge.edge_id.clone(),
            relation: edge.relation,
            from_node_id: edge.from_node_id,
            to_node_id: edge.to_node_id,
            weight: edge.weight,
            observed_at_epoch_seconds: edge.observed_at_epoch_seconds,
            sequence: owner_edges.len() as u64 + 1,
        };
        owner_edges.push(record.clone());
        self.seen_edge_ids.insert(edge.edge_id);
        record
    }
}

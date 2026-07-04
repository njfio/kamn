use crate::data_layer_m6_graph_integration::{
    support::{parse_kamn_did, validate_non_empty, validate_weight},
    DataLayerM6GraphEdgeInput, DataLayerM6GraphIntegrationError,
};

pub(super) fn normalize_edge_input(
    input: DataLayerM6GraphEdgeInput,
) -> Result<DataLayerM6GraphEdgeInput, DataLayerM6GraphIntegrationError> {
    let DataLayerM6GraphEdgeInput {
        owner_did,
        edge_id,
        relation,
        from_node_id,
        to_node_id,
        weight,
        observed_at_epoch_seconds,
    } = input;
    let owner_did = parse_kamn_did(owner_did.as_str())?;
    validate_non_empty(edge_id.as_str(), "edge_id")?;
    validate_non_empty(from_node_id.as_str(), "from_node_id")?;
    validate_non_empty(to_node_id.as_str(), "to_node_id")?;
    validate_weight(weight)?;
    if observed_at_epoch_seconds == 0 {
        return Err(DataLayerM6GraphIntegrationError::EmptyField(
            "observed_at_epoch_seconds",
        ));
    }
    Ok(DataLayerM6GraphEdgeInput {
        owner_did: owner_did.as_str().to_owned(),
        edge_id,
        relation,
        from_node_id,
        to_node_id,
        weight,
        observed_at_epoch_seconds,
    })
}

use super::super::models::*;
use crate::{AgentDid, AgentDidError, KamnDid};

pub(crate) fn validate_non_empty(
    value: &str,
    field_name: &'static str,
) -> Result<(), DataLayerM5VectorIntegrationError> {
    if value.trim().is_empty() {
        return Err(DataLayerM5VectorIntegrationError::EmptyField(field_name));
    }
    Ok(())
}

pub(crate) fn parse_kamn_did(value: &str) -> Result<KamnDid, DataLayerM5VectorIntegrationError> {
    KamnDid::parse(value)
        .map_err(|_| DataLayerM5VectorIntegrationError::InvalidDid(value.to_owned()))
}

pub(crate) fn validate_agent_did(
    value: &str,
) -> Result<AgentDid, DataLayerM5VectorIntegrationError> {
    AgentDid::parse(value).map_err(map_agent_did_error_to_m5)
}

fn map_agent_did_error_to_m5(error: AgentDidError) -> DataLayerM5VectorIntegrationError {
    DataLayerM5VectorIntegrationError::InvalidAgentDid {
        reason_code: DATA_LAYER_M5_INVALID_AGENT_DID_REASON_CODE,
        detail: error.to_string(),
    }
}

pub(crate) fn validate_vector(
    vector: Vec<f32>,
    field_name: &'static str,
) -> Result<Vec<f32>, DataLayerM5VectorIntegrationError> {
    if vector.is_empty() {
        return Err(DataLayerM5VectorIntegrationError::EmptyField(field_name));
    }
    if vector.iter().any(|value| !value.is_finite()) {
        return Err(DataLayerM5VectorIntegrationError::InvalidVectorValue(
            field_name,
        ));
    }
    Ok(vector)
}

pub(crate) fn resolve_limit(
    limit: Option<usize>,
) -> Result<usize, DataLayerM5VectorIntegrationError> {
    let resolved = limit.unwrap_or(20);
    if resolved == 0 {
        return Err(DataLayerM5VectorIntegrationError::InvalidLimit(resolved));
    }
    Ok(resolved)
}

pub(crate) fn resolve_lookback_window(
    lookback_window: Option<usize>,
) -> Result<usize, DataLayerM5VectorIntegrationError> {
    let resolved = lookback_window.unwrap_or(500);
    if resolved == 0 {
        return Err(DataLayerM5VectorIntegrationError::InvalidLookbackWindow(
            resolved,
        ));
    }
    Ok(resolved)
}

pub(crate) fn owner_vector_dimensions(records: &[DataLayerM5EmbeddingRecord]) -> Option<usize> {
    records
        .iter()
        .find_map(|record| record.vector_plaintext.as_ref().map(Vec::len))
}

use crate::data_layer_m3_blind_index_search::{
    DataLayerM3MessageMetadataRecord, DataLayerM3SearchError,
};
use crate::ContentRetrievalError;

pub(crate) fn validate_kamn_did(value: &str) -> Result<(), DataLayerM3SearchError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.starts_with("kamn:did:") {
        return Err(DataLayerM3SearchError::InvalidDid(value.to_owned()));
    }
    let segments = trimmed.split(':').collect::<Vec<_>>();
    if segments.len() < 4 || segments.iter().any(|segment| segment.is_empty()) {
        return Err(DataLayerM3SearchError::InvalidDid(value.to_owned()));
    }
    Ok(())
}

pub(crate) fn validate_non_empty(
    value: &str,
    field_name: &'static str,
) -> Result<(), DataLayerM3SearchError> {
    if value.trim().is_empty() {
        return Err(DataLayerM3SearchError::EmptyField(field_name));
    }
    Ok(())
}

pub(crate) fn canonical_field_name(field_name: &str) -> Result<String, DataLayerM3SearchError> {
    let trimmed = field_name.trim();
    if trimmed.is_empty() {
        return Err(DataLayerM3SearchError::EmptyField("field_name"));
    }
    let canonical = trimmed.to_ascii_lowercase();
    if canonical
        .chars()
        .any(|character| !(character.is_ascii_alphanumeric() || character == '_'))
    {
        return Err(DataLayerM3SearchError::EmptyField("field_name"));
    }
    Ok(canonical)
}

pub(crate) fn validate_blind_index_token(
    field_name: &str,
    token: &str,
) -> Result<(), DataLayerM3SearchError> {
    let trimmed = token.trim();
    if trimmed.is_empty() || !trimmed.starts_with("sha256:") {
        return Err(DataLayerM3SearchError::InvalidBlindIndexToken {
            field_name: field_name.to_owned(),
        });
    }
    Ok(())
}

pub(crate) fn resolve_limit(limit: Option<usize>) -> Result<usize, DataLayerM3SearchError> {
    match limit {
        Some(0) => Err(DataLayerM3SearchError::InvalidLimit(0)),
        Some(value) => Ok(value),
        None => Ok(usize::MAX),
    }
}

pub(crate) fn map_content_retrieval_error_to_m3_projection_error(
    error: ContentRetrievalError,
) -> DataLayerM3SearchError {
    DataLayerM3SearchError::InvalidRetrievalRequestProjection {
        reason: error.to_string(),
    }
}

pub(crate) fn normalize_blind_index_value(value: &str) -> Result<String, DataLayerM3SearchError> {
    let normalized = value
        .split_whitespace()
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join(" ");
    if normalized.is_empty() {
        return Err(DataLayerM3SearchError::EmptyField("value"));
    }
    Ok(normalized)
}

pub(crate) fn sort_results_deterministically(results: &mut [DataLayerM3MessageMetadataRecord]) {
    results.sort_by(|left, right| {
        right
            .created_at_epoch_seconds
            .cmp(&left.created_at_epoch_seconds)
            .then(left.message_id.cmp(&right.message_id))
    });
}

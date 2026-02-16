use super::BlockPipelineError;

pub(super) fn extract_error_reason_marker(detail: &str) -> Option<String> {
    let trimmed = detail.trim();
    let open_index = trimmed.rfind('(')?;
    let close_index = trimmed.rfind(')')?;
    if close_index != trimmed.len().saturating_sub(1) || close_index <= open_index + 1 {
        return None;
    }
    let marker = &trimmed[open_index + 1..close_index];
    if marker.chars().all(|character| {
        character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || character == '_'
            || character == '-'
            || character == ':'
    }) {
        return Some(marker.to_owned());
    }
    None
}

pub(super) fn validate_transport_payload_field_value(
    field: &str,
    value: &str,
) -> Result<(), BlockPipelineError> {
    if value.trim().is_empty() {
        return Err(BlockPipelineError::TransportFeed(format!(
            "transport candidate field is empty: {field} (transport_candidate_field_empty)"
        )));
    }
    if value.contains('\n') || value.contains('\r') {
        return Err(BlockPipelineError::TransportFeed(format!(
            "transport candidate field contains line break: {field} (transport_candidate_field_line_break)"
        )));
    }
    Ok(())
}

pub(super) fn validate_canonical_commit_store_field(
    field: &str,
    value: &str,
) -> Result<(), BlockPipelineError> {
    if value.trim().is_empty() {
        return Err(BlockPipelineError::CommitStore(format!(
            "canonical commit field is empty: {field} (canonical_commit_store_field_empty)"
        )));
    }
    if value.contains('|') {
        return Err(BlockPipelineError::CommitStore(format!(
            "canonical commit field contains reserved separator '|': {field} (canonical_commit_store_field_separator_invalid)"
        )));
    }
    if value.contains('\n') || value.contains('\r') {
        return Err(BlockPipelineError::CommitStore(format!(
            "canonical commit field contains line break: {field} (canonical_commit_store_field_line_break)"
        )));
    }
    Ok(())
}

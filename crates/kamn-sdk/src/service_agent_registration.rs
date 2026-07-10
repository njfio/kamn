use crate::{AgentMetadata, SdkError};

/// Builds the canonical JSON body for service-backed agent registration.
pub fn service_agent_registration_payload(metadata: &AgentMetadata) -> Result<String, SdkError> {
    validate_agent_metadata(metadata)?;
    Ok(serde_json::json!({
        "agent_type": metadata.agent_type,
        "model_family": metadata.model_family,
        "capabilities": metadata.capabilities,
    })
    .to_string())
}

fn validate_agent_metadata(metadata: &AgentMetadata) -> Result<(), SdkError> {
    validate_non_empty("agent_type", metadata.agent_type.as_str())?;
    validate_non_empty("model_family", metadata.model_family.as_str())?;
    if metadata.capabilities.is_empty() {
        return Err(invalid(
            "capabilities",
            "must include at least one capability",
        ));
    }
    if metadata
        .capabilities
        .iter()
        .any(|value| value.trim().is_empty())
    {
        return Err(invalid(
            "capabilities",
            "must not include empty capability entries",
        ));
    }
    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), SdkError> {
    if value.trim().is_empty() {
        return Err(invalid(field, "must not be empty"));
    }
    Ok(())
}

fn invalid(field: &'static str, reason: &'static str) -> SdkError {
    SdkError::InvalidInput { field, reason }
}

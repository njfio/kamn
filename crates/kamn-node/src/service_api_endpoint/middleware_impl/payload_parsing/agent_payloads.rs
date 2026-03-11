use super::*;

pub(super) fn invalid_agent_registration_json(error: serde_json::Error) -> ServiceApiReasonedError {
    agent_registration_error(format!(
        "agent registration payload must be valid json: {error}"
    ))
}

pub(super) fn invalid_agent_search_json(error: serde_json::Error) -> ServiceApiReasonedError {
    ServiceApiReasonedError::new(
        REASON_CODE_AGENT_SEARCH_PAYLOAD_INVALID,
        format!("agent search payload must be valid json: {error}"),
    )
}

pub(super) fn validate_agent_registration_fields(
    parsed: &ServiceApiAgentRegisterRequestBody,
) -> Result<(), ServiceApiReasonedError> {
    ensure_non_empty(
        &parsed.agent_type,
        "agent registration payload missing non-empty agent_type",
    )?;
    ensure_non_empty(
        &parsed.model_family,
        "agent registration payload missing non-empty model_family",
    )
}

pub(super) fn validate_agent_registration_capabilities(
    parsed: &ServiceApiAgentRegisterRequestBody,
) -> Result<(), ServiceApiReasonedError> {
    if parsed.capabilities.is_empty() {
        return Err(agent_registration_error(
            "agent registration payload missing non-empty capabilities",
        ));
    }
    if parsed
        .capabilities
        .iter()
        .any(|value| value.trim().is_empty())
    {
        return Err(agent_registration_error(
            "agent registration payload capabilities must not contain empty entries",
        ));
    }
    Ok(())
}

pub(super) fn ensure_optional_search_value(
    value: Option<&str>,
    field_name: &str,
) -> Result<(), ServiceApiReasonedError> {
    if value.is_some_and(|value| value.trim().is_empty()) {
        return Err(ServiceApiReasonedError::new(
            REASON_CODE_AGENT_SEARCH_PAYLOAD_INVALID,
            format!("agent search payload {field_name} must not be empty when provided"),
        ));
    }
    Ok(())
}

fn ensure_non_empty(value: &str, message: &str) -> Result<(), ServiceApiReasonedError> {
    if value.trim().is_empty() {
        return Err(agent_registration_error(message));
    }
    Ok(())
}

pub(super) fn agent_registration_error(message: impl Into<String>) -> ServiceApiReasonedError {
    ServiceApiReasonedError::new(REASON_CODE_AGENT_REGISTRATION_PAYLOAD_INVALID, message)
}

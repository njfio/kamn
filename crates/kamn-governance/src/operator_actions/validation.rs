use super::error::OperatorActionServiceError;

pub fn validate_request(
    config_key: &str,
    config_value: &str,
    requested_at_unix: u64,
) -> Result<(), OperatorActionServiceError> {
    require_non_empty("config_key", config_key)?;
    require_non_empty("config_value", config_value)?;
    require_requested_at(requested_at_unix)
}

pub fn require_requested_at(requested_at_unix: u64) -> Result<(), OperatorActionServiceError> {
    if requested_at_unix == 0 {
        return Err(OperatorActionServiceError::EmptyField("requested_at_unix"));
    }
    Ok(())
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), OperatorActionServiceError> {
    if value.trim().is_empty() {
        return Err(OperatorActionServiceError::EmptyField(field));
    }
    Ok(())
}

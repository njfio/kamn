use super::*;
use serde_json::Value;

mod agent_payloads;

use agent_payloads::{
    ensure_optional_search_value, invalid_agent_registration_json, invalid_agent_search_json,
    validate_agent_registration_capabilities, validate_agent_registration_fields,
};

pub(super) struct ServiceApiRelayIngestPayload {
    pub(super) message_id: String,
    pub(super) sender_did: Option<String>,
    pub(super) recipient_did: String,
    pub(super) body: String,
}

pub(super) fn extract_channel_id_from_payload(payload: &str) -> Option<String> {
    let parsed = serde_json::from_str::<Value>(payload).ok()?;
    let channel_id = parsed.get("channel_id")?.as_str()?.trim();
    (!channel_id.is_empty()).then(|| channel_id.to_owned())
}

pub(super) fn extract_canonical_recipient_did_from_payload(
    payload: &str,
) -> Result<Option<String>, ServiceApiReasonedError> {
    let Ok(parsed) = serde_json::from_str::<Value>(payload) else {
        return Ok(None);
    };
    for key in ["recipient_did", "to", "to_did"] {
        let Some(raw_value) = parsed.get(key).and_then(Value::as_str) else {
            continue;
        };
        let recipient_did = raw_value.trim();
        if recipient_did.is_empty() {
            continue;
        }
        AgentDid::parse(recipient_did).map_err(invalid_recipient_did)?;
        return Ok(Some(recipient_did.to_owned()));
    }
    Ok(None)
}

pub(super) fn parse_relay_ingest_payload(
    payload: &str,
) -> Result<ServiceApiRelayIngestPayload, ServiceApiReasonedError> {
    let parsed = serde_json::from_str::<Value>(payload).map_err(invalid_relay_json)?;
    Ok(ServiceApiRelayIngestPayload {
        message_id: required_string_field(
            &parsed,
            "message_id",
            "relay payload missing non-empty message_id",
        )?,
        sender_did: optional_valid_did(&parsed, "sender_did")?,
        recipient_did: required_valid_did(
            &parsed,
            "recipient_did",
            "relay payload missing non-empty recipient_did",
        )?,
        body: required_string_field(&parsed, "body", "relay payload missing string body")?,
    })
}

pub(super) fn parse_agent_registration_payload(
    payload: &str,
) -> Result<ServiceApiAgentRegisterRequestBody, ServiceApiReasonedError> {
    let parsed = serde_json::from_str::<ServiceApiAgentRegisterRequestBody>(payload)
        .map_err(invalid_agent_registration_json)?;
    validate_agent_registration_fields(&parsed)?;
    validate_agent_registration_capabilities(&parsed)?;
    Ok(parsed)
}

pub(super) fn parse_agent_search_payload(
    payload: &str,
) -> Result<ServiceApiAgentSearchRequestBody, ServiceApiReasonedError> {
    let parsed = serde_json::from_str::<ServiceApiAgentSearchRequestBody>(payload)
        .map_err(invalid_agent_search_json)?;
    ensure_optional_search_value(parsed.model_family.as_deref(), "model_family")?;
    ensure_optional_search_value(parsed.capability.as_deref(), "capability")?;
    Ok(ServiceApiAgentSearchRequestBody {
        capability: parsed.capability.map(|value| value.trim().to_owned()),
        model_family: parsed.model_family.map(|value| value.trim().to_owned()),
    })
}

fn invalid_recipient_did(error: impl std::fmt::Display) -> ServiceApiReasonedError {
    ServiceApiReasonedError::new(
        REASON_CODE_MESSAGE_RECIPIENT_DID_INVALID,
        format!("invalid recipient did: {error}"),
    )
}

fn invalid_relay_json(error: serde_json::Error) -> ServiceApiReasonedError {
    ServiceApiReasonedError::new(
        REASON_CODE_RELAY_PAYLOAD_INVALID,
        format!("relay payload must be valid json: {error}"),
    )
}

fn required_string_field(
    parsed: &Value,
    key: &str,
    missing_message: &str,
) -> Result<String, ServiceApiReasonedError> {
    parsed
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            ServiceApiReasonedError::new(REASON_CODE_RELAY_PAYLOAD_INVALID, missing_message)
        })
}

fn required_valid_did(
    parsed: &Value,
    key: &str,
    missing_message: &str,
) -> Result<String, ServiceApiReasonedError> {
    let did = required_string_field(parsed, key, missing_message)?;
    validate_relay_agent_did(key.trim_end_matches("_did"), did.as_str())?;
    Ok(did)
}

fn optional_valid_did(
    parsed: &Value,
    key: &str,
) -> Result<Option<String>, ServiceApiReasonedError> {
    let Some(value) = parsed.get(key).and_then(Value::as_str).map(str::trim) else {
        return Ok(None);
    };
    if value.is_empty() {
        return Ok(None);
    }
    validate_relay_agent_did(key.trim_end_matches("_did"), value)?;
    Ok(Some(value.to_owned()))
}

fn validate_relay_agent_did(role: &str, did: &str) -> Result<(), ServiceApiReasonedError> {
    AgentDid::parse(did).map_err(|error| {
        ServiceApiReasonedError::new(
            REASON_CODE_RELAY_DID_INVALID,
            format!("invalid relay {role} did: {error}"),
        )
    })?;
    Ok(())
}

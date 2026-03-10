use super::{
    default_endpoint, kolme_endpoint, validate_non_empty, DEFAULT_S12_AGENT_NAME,
    DEFAULT_S12_REGISTER_CONTENT_PAYLOAD,
};
use crate::drivers::sdk_direct::live_probe_tranche_three::live_probe_support::validate_content_state;

pub(super) fn run_live_s12_retention_deletion_probe() -> Result<(), String> {
    let settings = s12_settings();
    let registered = register_content(&settings)?;
    expire_content(&settings, registered.content_id.as_str())?;
    let tombstoned = tombstone_content(&settings, registered.content_id.as_str())?;
    query_content(&settings, registered.content_id.as_str(), &tombstoned)
}

struct S12Settings {
    endpoint: String,
    kolme_endpoint: String,
    base_agent_name: String,
    register_payload: String,
}

struct S12ContentState {
    content_id: String,
    lifecycle_state: String,
    redaction_status: String,
}

fn s12_settings() -> S12Settings {
    S12Settings {
        endpoint: default_endpoint(),
        kolme_endpoint: kolme_endpoint(),
        base_agent_name: super::super::env_var_or_default(
            "KAMN_E2E_S12_AGENT_NAME",
            DEFAULT_S12_AGENT_NAME,
        ),
        register_payload: super::super::env_var_or_default(
            "KAMN_E2E_S12_REGISTER_CONTENT_PAYLOAD",
            DEFAULT_S12_REGISTER_CONTENT_PAYLOAD,
        ),
    }
}

fn register_content(settings: &S12Settings) -> Result<S12ContentState, String> {
    let handle = connect_content_agent(
        settings,
        "register",
        "sdk-direct live s12 register connect failed",
    )?;
    let registration = handle
        .register_content(settings.register_payload.as_str())
        .map_err(|error| format!("sdk-direct live s12 register-content failed: {error}"))?;
    validate_registration(
        registration.content_id.as_str(),
        registration.retention_class.as_str(),
        registration.lifecycle_state.as_str(),
        registration.redaction_status.as_str(),
    )?;
    Ok(S12ContentState {
        content_id: registration.content_id,
        lifecycle_state: registration.lifecycle_state,
        redaction_status: registration.redaction_status,
    })
}

fn expire_content(settings: &S12Settings, content_id: &str) -> Result<(), String> {
    let handle = connect_content_agent(
        settings,
        "expire",
        "sdk-direct live s12 expire connect failed",
    )?;
    let expired = handle
        .expire_content(content_id)
        .map_err(|error| format!("sdk-direct live s12 expire-content failed: {error}"))?;
    super::super::validate_s12_content_id_match(
        content_id,
        expired.content_id.as_str(),
        "sdk-direct live s12 expire-content",
    )?;
    validate_content_state(
        expired.lifecycle_state.as_str(),
        expired.redaction_status.as_str(),
        "sdk-direct live s12 expire-content",
    )
}

fn tombstone_content(settings: &S12Settings, content_id: &str) -> Result<S12ContentState, String> {
    let handle = connect_content_agent(
        settings,
        "tombstone",
        "sdk-direct live s12 tombstone connect failed",
    )?;
    let tombstoned = handle
        .tombstone_content(content_id)
        .map_err(|error| format!("sdk-direct live s12 tombstone-content failed: {error}"))?;
    super::super::validate_s12_content_id_match(
        content_id,
        tombstoned.content_id.as_str(),
        "sdk-direct live s12 tombstone-content",
    )?;
    validate_content_state(
        tombstoned.lifecycle_state.as_str(),
        tombstoned.redaction_status.as_str(),
        "sdk-direct live s12 tombstone-content",
    )?;
    Ok(S12ContentState {
        content_id: tombstoned.content_id,
        lifecycle_state: tombstoned.lifecycle_state,
        redaction_status: tombstoned.redaction_status,
    })
}

fn query_content(
    settings: &S12Settings,
    content_id: &str,
    expected: &S12ContentState,
) -> Result<(), String> {
    let handle = connect_content_agent(
        settings,
        "query",
        "sdk-direct live s12 query connect failed",
    )?;
    let queried = handle
        .query_content(content_id)
        .map_err(|error| format!("sdk-direct live s12 query-content failed: {error}"))?;
    validate_query_state(
        content_id,
        expected,
        queried.content_id.as_str(),
        queried.lifecycle_state.as_str(),
        queried.redaction_status.as_str(),
    )
}

fn connect_content_agent(
    settings: &S12Settings,
    suffix: &str,
    context: &str,
) -> Result<super::KamnAgentHandle, String> {
    super::connect_agent(
        settings.endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-{suffix}", settings.base_agent_name).as_str(),
        context,
    )
}

fn validate_registration(
    content_id: &str,
    retention_class: &str,
    lifecycle_state: &str,
    redaction_status: &str,
) -> Result<(), String> {
    validate_non_empty(
        content_id,
        "sdk-direct live s12 register-content returned empty content_id",
    )?;
    validate_non_empty(
        retention_class,
        "sdk-direct live s12 register-content returned empty retention_class",
    )?;
    validate_content_state(
        lifecycle_state,
        redaction_status,
        "sdk-direct live s12 register-content",
    )
}

fn validate_query_state(
    content_id: &str,
    expected: &S12ContentState,
    observed_id: &str,
    observed_lifecycle: &str,
    observed_redaction: &str,
) -> Result<(), String> {
    super::super::validate_s12_content_id_match(
        content_id,
        observed_id,
        "sdk-direct live s12 query-content",
    )?;
    validate_query_field(
        expected.lifecycle_state.as_str(),
        observed_lifecycle,
        "lifecycle_state",
    )?;
    validate_query_field(
        expected.redaction_status.as_str(),
        observed_redaction,
        "redaction_status",
    )
}

fn validate_query_field(expected: &str, observed: &str, field: &str) -> Result<(), String> {
    super::super::validate_s12_content_field_coherence(
        expected,
        observed,
        field,
        "sdk-direct live s12 query-content",
    )
}

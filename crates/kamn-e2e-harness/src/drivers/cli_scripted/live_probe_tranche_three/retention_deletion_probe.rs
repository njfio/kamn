use super::{
    default_endpoint, env_payload, env_value, validate_non_empty, DEFAULT_S12_AGENT_NAME,
    DEFAULT_S12_REGISTER_CONTENT_PAYLOAD,
};
use crate::drivers::cli_scripted::live_probe_tranche_three::live_probe_support::validate_content_state;
use crate::drivers::shared_helpers::{
    validate_s12_content_field_coherence, validate_s12_content_id_match,
};

pub(super) fn run_live_s12_cli_retention_deletion_probe() -> Result<(), String> {
    let settings = s12_settings();
    let registered = register_content(&settings)?;
    expire_content(&settings, registered.content_id.as_str())?;
    let tombstoned = tombstone_content(&settings, registered.content_id.as_str())?;
    query_content(&settings, registered.content_id.as_str(), &tombstoned)
}

struct S12Settings {
    endpoint: String,
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
        base_agent_name: env_value("KAMN_E2E_S12_AGENT_NAME", DEFAULT_S12_AGENT_NAME),
        register_payload: env_payload(
            "KAMN_E2E_S12_REGISTER_CONTENT_PAYLOAD",
            DEFAULT_S12_REGISTER_CONTENT_PAYLOAD,
        ),
    }
}

fn register_content(settings: &S12Settings) -> Result<S12ContentState, String> {
    let output = run_content_command(
        settings,
        "register-content",
        settings.register_payload.as_str(),
        "register",
    )?;
    let content_id = require_content_id(output.as_str(), "cli live s12 register-content")?;
    validate_non_empty(
        require_field(
            output.as_str(),
            "retention_class",
            "cli live s12 register-content",
        )?,
        "cli live s12 register-content returned empty retention_class",
    )?;
    let state = content_state(output.as_str(), "cli live s12 register-content")?;
    Ok(S12ContentState {
        content_id,
        ..state
    })
}

fn expire_content(settings: &S12Settings, content_id: &str) -> Result<(), String> {
    let output = run_content_command(settings, "expire-content", content_id, "expire")?;
    validate_s12_content_id_match(
        content_id,
        require_field(output.as_str(), "content_id", "cli live s12 expire-content")?,
        "cli live s12 expire-content",
    )?;
    let state = content_state(output.as_str(), "cli live s12 expire-content")?;
    validate_content_state(
        state.lifecycle_state.as_str(),
        state.redaction_status.as_str(),
        "cli live s12 expire-content",
    )
}

fn tombstone_content(settings: &S12Settings, content_id: &str) -> Result<S12ContentState, String> {
    let output = run_content_command(settings, "tombstone-content", content_id, "tombstone")?;
    validate_s12_content_id_match(
        content_id,
        require_field(
            output.as_str(),
            "content_id",
            "cli live s12 tombstone-content",
        )?,
        "cli live s12 tombstone-content",
    )?;
    let state = content_state(output.as_str(), "cli live s12 tombstone-content")?;
    validate_content_state(
        state.lifecycle_state.as_str(),
        state.redaction_status.as_str(),
        "cli live s12 tombstone-content",
    )?;
    Ok(S12ContentState {
        content_id: content_id.to_owned(),
        ..state
    })
}

fn query_content(
    settings: &S12Settings,
    content_id: &str,
    expected: &S12ContentState,
) -> Result<(), String> {
    let output = run_content_command(settings, "query-content", content_id, "query")?;
    validate_s12_content_id_match(
        content_id,
        require_field(output.as_str(), "content_id", "cli live s12 query-content")?,
        "cli live s12 query-content",
    )?;
    let observed = content_state(output.as_str(), "cli live s12 query-content")?;
    validate_query_field(
        expected.lifecycle_state.as_str(),
        observed.lifecycle_state.as_str(),
        "lifecycle_state",
    )?;
    validate_query_field(
        expected.redaction_status.as_str(),
        observed.redaction_status.as_str(),
        "redaction_status",
    )
}

fn run_content_command(
    settings: &S12Settings,
    command: &str,
    value: &str,
    suffix: &str,
) -> Result<String, String> {
    super::super::run_cli_command_capture_stdout_with_agent_name(
        super::cli_binary().as_str(),
        &[
            command,
            "--endpoint",
            settings.endpoint.as_str(),
            "--format",
            "text",
            value,
        ],
        &format!("cli live s12 {command}"),
        format!("{}-{suffix}", settings.base_agent_name).as_str(),
    )
}

fn require_content_id(output: &str, step: &str) -> Result<String, String> {
    let content_id = require_field(output, "content_id", step)?;
    validate_non_empty(content_id, &format!("{step} returned empty content_id"))?;
    Ok(content_id.to_owned())
}

fn content_state(output: &str, step: &str) -> Result<S12ContentState, String> {
    Ok(S12ContentState {
        content_id: require_field(output, "content_id", step)?.to_owned(),
        lifecycle_state: require_field(output, "lifecycle_state", step)?.to_owned(),
        redaction_status: require_field(output, "redaction_status", step)?.to_owned(),
    })
}

fn validate_query_field(expected: &str, observed: &str, field: &str) -> Result<(), String> {
    validate_s12_content_field_coherence(expected, observed, field, "cli live s12 query-content")
}

fn require_field<'a>(output: &'a str, key: &str, step: &str) -> Result<&'a str, String> {
    super::super::parse_text_output_field(output, key)
        .ok_or_else(|| format!("{step} response missing {key} field: {output}"))
}

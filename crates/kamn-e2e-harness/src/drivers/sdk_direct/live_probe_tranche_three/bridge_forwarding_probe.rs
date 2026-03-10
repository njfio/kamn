use super::{
    default_endpoint, kolme_endpoint, validate_non_empty, DEFAULT_S13_AGENT_NAME,
    DEFAULT_S13_SUBMIT_BRIDGE_PAYLOAD,
};
use crate::drivers::sdk_direct::live_probe_tranche_three::live_probe_support::{
    validate_bridge_forward_fields, validate_queried_bridge_field,
};

pub(super) fn run_live_s13_bridge_forwarding_probe() -> Result<(), String> {
    let settings = s13_settings();
    let bridge_id = submit_bridge_message(&settings)?;
    let forwarded = forward_bridge_message(&settings, bridge_id.as_str())?;
    query_bridge_message(&settings, bridge_id.as_str(), &forwarded)
}

struct S13Settings {
    endpoint: String,
    kolme_endpoint: String,
    base_agent_name: String,
    submit_payload: String,
}

struct S13ForwardedState {
    bridge_status: String,
    target_message_id: String,
    forward_tx_hash: String,
}

fn s13_settings() -> S13Settings {
    S13Settings {
        endpoint: default_endpoint(),
        kolme_endpoint: kolme_endpoint(),
        base_agent_name: super::super::env_var_or_default(
            "KAMN_E2E_S13_AGENT_NAME",
            DEFAULT_S13_AGENT_NAME,
        ),
        submit_payload: super::super::env_var_or_default(
            "KAMN_E2E_S13_SUBMIT_BRIDGE_PAYLOAD",
            DEFAULT_S13_SUBMIT_BRIDGE_PAYLOAD,
        ),
    }
}

fn submit_bridge_message(settings: &S13Settings) -> Result<String, String> {
    let handle = connect_bridge_agent(
        settings,
        "submit",
        "sdk-direct live s13 submit connect failed",
    )?;
    let submitted = handle
        .submit_bridge_message(settings.submit_payload.as_str())
        .map_err(|error| format!("sdk-direct live s13 submit-bridge-message failed: {error}"))?;
    validate_non_empty(
        submitted.bridge_id.as_str(),
        "sdk-direct live s13 submit-bridge-message returned empty bridge_id",
    )?;
    validate_non_empty(
        submitted.source_message_id.as_str(),
        "sdk-direct live s13 submit-bridge-message returned empty source_message_id",
    )?;
    validate_non_empty(
        submitted.bridge_status.as_str(),
        "sdk-direct live s13 submit-bridge-message returned empty bridge_status",
    )?;
    Ok(submitted.bridge_id)
}

fn forward_bridge_message(
    settings: &S13Settings,
    bridge_id: &str,
) -> Result<S13ForwardedState, String> {
    let handle = connect_bridge_agent(
        settings,
        "forward",
        "sdk-direct live s13 forward connect failed",
    )?;
    let forwarded = handle
        .forward_bridge_message(bridge_id)
        .map_err(|error| format!("sdk-direct live s13 forward-bridge-message failed: {error}"))?;
    validate_forwarded_bridge_state(
        bridge_id,
        forwarded.bridge_id.as_str(),
        forwarded.bridge_status.as_str(),
        forwarded.target_message_id.as_str(),
        forwarded.forward_tx_hash.as_str(),
    )?;
    Ok(S13ForwardedState {
        bridge_status: forwarded.bridge_status,
        target_message_id: forwarded.target_message_id,
        forward_tx_hash: forwarded.forward_tx_hash,
    })
}

fn query_bridge_message(
    settings: &S13Settings,
    bridge_id: &str,
    forwarded: &S13ForwardedState,
) -> Result<(), String> {
    let handle = connect_bridge_agent(
        settings,
        "query",
        "sdk-direct live s13 query connect failed",
    )?;
    let queried = handle
        .query_bridge_message(bridge_id)
        .map_err(|error| format!("sdk-direct live s13 query-bridge-message failed: {error}"))?;
    validate_queried_bridge_state(
        bridge_id,
        forwarded,
        (
            queried.bridge_id.as_str(),
            queried.bridge_status.as_str(),
            queried.target_message_id.as_str(),
            queried.forward_tx_hash.as_str(),
        ),
    )
}

fn connect_bridge_agent(
    settings: &S13Settings,
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

fn validate_forwarded_bridge_state(
    bridge_id: &str,
    observed_bridge_id: &str,
    observed_bridge_status: &str,
    observed_target_message_id: &str,
    observed_forward_tx_hash: &str,
) -> Result<(), String> {
    super::super::validate_s13_bridge_id_match(
        bridge_id,
        observed_bridge_id,
        "sdk-direct live s13 forward-bridge-message",
    )?;
    validate_bridge_forward_fields(
        observed_bridge_status,
        observed_target_message_id,
        observed_forward_tx_hash,
        "sdk-direct live s13 forward-bridge-message",
    )
}

fn validate_queried_bridge_state(
    bridge_id: &str,
    forwarded: &S13ForwardedState,
    observed_fields: (&str, &str, &str, &str),
) -> Result<(), String> {
    validate_queried_bridge_id(bridge_id, observed_fields.0)?;
    validate_queried_bridge_fields(forwarded, observed_fields)
}

fn validate_queried_bridge_id(bridge_id: &str, observed_bridge_id: &str) -> Result<(), String> {
    super::super::validate_s13_bridge_id_match(
        bridge_id,
        observed_bridge_id,
        "sdk-direct live s13 query-bridge-message",
    )
}

fn validate_queried_bridge_fields(
    forwarded: &S13ForwardedState,
    observed_fields: (&str, &str, &str, &str),
) -> Result<(), String> {
    [observed_fields.1, observed_fields.2, observed_fields.3]
        .into_iter()
        .zip(queried_bridge_expected_fields(forwarded))
        .zip(["bridge_status", "target_message_id", "forward_tx_hash"])
        .try_for_each(|((observed, expected), field)| {
            validate_queried_bridge_field(expected, observed, field)
        })
}

fn queried_bridge_expected_fields(forwarded: &S13ForwardedState) -> [&str; 3] {
    [
        forwarded.bridge_status.as_str(),
        forwarded.target_message_id.as_str(),
        forwarded.forward_tx_hash.as_str(),
    ]
}

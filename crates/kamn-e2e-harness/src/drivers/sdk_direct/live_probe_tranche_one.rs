use super::{env_var_or_default, KamnAgentHandle, DEFAULT_AGENT_NAME, DEFAULT_KOLME_ENDPOINT};

#[path = "live_probe_tranche_one/channel_task_probes.rs"]
mod channel_task_probes;
#[path = "live_probe_tranche_one/discovery_direct_message_probes.rs"]
mod discovery_direct_message_probes;
#[path = "live_probe_tranche_one/escrow_probe_support.rs"]
mod escrow_probe_support;

const DEFAULT_S02_MESSAGE_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s02"}"#;
const DEFAULT_S02_REPLY_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s02-reply"}"#;
const DEFAULT_S03_CHANNEL_PAYLOAD: &str =
    r#"{"name":"sdk-direct-live-s03","members":["alice","bob","carol"]}"#;
const DEFAULT_S03_MESSAGE_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s03-channel-message"}"#;
const DEFAULT_S04_CREATE_TASK_PAYLOAD: &str =
    r#"{"title":"sdk-direct-live-s04","description":"live task lifecycle probe"}"#;
const DEFAULT_S04_ESCROW_AMOUNT: u64 = 1;
const DEFAULT_S05_FUND_ESCROW_PAYLOAD: &str = r#"{"task_id":"sdk-direct-live-s05","amount":1}"#;

pub(super) fn base_agent_name() -> String {
    env_var_or_default("KAMN_AGENT_NAME", DEFAULT_AGENT_NAME)
}

pub(super) fn connect_agent(agent_name: &str, context: &str) -> Result<KamnAgentHandle, String> {
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    KamnAgentHandle::connect(endpoint.as_str(), kolme_endpoint.as_str(), agent_name)
        .map_err(|error| format!("{context}: {error}"))
}

pub(super) fn validate_non_empty(value: &str, error: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(error.to_owned());
    }
    Ok(())
}

pub(super) fn run_live_s01_discovery_probe() -> Result<(), String> {
    discovery_direct_message_probes::run_live_s01_discovery_probe()
}

pub(super) fn run_live_s02_direct_message_probe() -> Result<(), String> {
    discovery_direct_message_probes::run_live_s02_direct_message_probe()
}

pub(super) fn run_live_s03_group_channel_probe() -> Result<(), String> {
    channel_task_probes::run_live_s03_group_channel_probe()
}

pub(super) fn run_live_s04_task_lifecycle_probe() -> Result<(), String> {
    channel_task_probes::run_live_s04_task_lifecycle_probe()
}

pub(super) fn run_live_s05_escrow_settlement_probe() -> Result<(), String> {
    escrow_probe_support::run_live_s05_escrow_settlement_probe()
}

#[cfg(test)]
pub(super) fn validate_live_s03_query_message_response(
    expected_message_id: &str,
    queried_message_id: &str,
    queried_status: &str,
) -> Result<(), String> {
    channel_task_probes::validate_live_s03_query_message_response(
        expected_message_id,
        queried_message_id,
        queried_status,
    )
}

#[cfg(test)]
pub(super) fn validate_live_s03_list_messages_response(
    expected_channel_id: &str,
    listed_channel_id: &str,
) -> Result<(), String> {
    channel_task_probes::validate_live_s03_list_messages_response(
        expected_channel_id,
        listed_channel_id,
    )
}

#[cfg(test)]
pub(super) fn validate_live_s05_release_escrow_receipt(
    expected_escrow_id: &str,
    released_escrow_id: &str,
    released_state: &str,
) -> Result<(), String> {
    escrow_probe_support::validate_live_s05_release_escrow_receipt(
        expected_escrow_id,
        released_escrow_id,
        released_state,
    )
}

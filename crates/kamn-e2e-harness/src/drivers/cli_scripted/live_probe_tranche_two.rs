use std::env;

#[path = "live_probe_tranche_two/message_query_support.rs"]
mod message_query_support;
#[path = "live_probe_tranche_two/proof_replay_probes.rs"]
mod proof_replay_probes;
#[path = "live_probe_tranche_two/recovery_failover_probes.rs"]
mod recovery_failover_probes;
#[path = "live_probe_tranche_two/topology_coherence_probe.rs"]
mod topology_coherence_probe;

const CLI_BINARY_ENV: &str = super::CLI_BINARY_ENV;
const DEFAULT_CLI_BINARY: &str = super::DEFAULT_CLI_BINARY;
const DEFAULT_S07_AGENT_NAME: &str = super::DEFAULT_S07_AGENT_NAME;
const DEFAULT_S07_MESSAGE_PAYLOAD: &str = super::DEFAULT_S07_MESSAGE_PAYLOAD;
const DEFAULT_S08_AGENT_NAME: &str = super::DEFAULT_S08_AGENT_NAME;
const DEFAULT_S08_PRE_MESSAGE_PAYLOAD: &str = super::DEFAULT_S08_PRE_MESSAGE_PAYLOAD;
const DEFAULT_S08_POST_MESSAGE_PAYLOAD: &str = super::DEFAULT_S08_POST_MESSAGE_PAYLOAD;
const DEFAULT_S09_AGENT_NAME: &str = super::DEFAULT_S09_AGENT_NAME;
const DEFAULT_S09_PRE_MESSAGE_PAYLOAD: &str = super::DEFAULT_S09_PRE_MESSAGE_PAYLOAD;
const DEFAULT_S09_POST_MESSAGE_PAYLOAD: &str = super::DEFAULT_S09_POST_MESSAGE_PAYLOAD;
const DEFAULT_S10_AGENT_NAME: &str = super::DEFAULT_S10_AGENT_NAME;
const DEFAULT_S10_MESSAGE_PAYLOAD: &str = super::DEFAULT_S10_MESSAGE_PAYLOAD;
const DEFAULT_S06_MESSAGE_ID: &str = super::DEFAULT_S06_MESSAGE_ID;
const DEFAULT_S06_TX_HASH: &str = super::DEFAULT_S06_TX_HASH;
const DEFAULT_S06_BLOCK_HEIGHT: u64 = super::DEFAULT_S06_BLOCK_HEIGHT;
const DEFAULT_S06_FINALITY: &str = super::DEFAULT_S06_FINALITY;

pub(super) use message_query_support::{
    validate_s08_distinct_message_ids, validate_s08_message_receipt_fields,
    validate_s08_query_message_response,
};

pub(super) fn cli_binary() -> String {
    super::env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY)
}

pub(super) fn default_endpoint() -> String {
    super::env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080")
}

pub(super) fn env_payload(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}

pub(super) fn env_value(name: &str, default: &str) -> String {
    super::env_var_or_default(name, default)
}

pub(super) fn agent_name(name: &str, default: &str) -> String {
    env_value(name, default)
}

pub(super) fn validate_non_empty(value: &str, error: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(error.to_owned());
    }
    Ok(())
}

pub(super) fn run_live_s06_cli_proof_verification_probe() -> Result<(), String> {
    proof_replay_probes::run_live_s06_cli_proof_verification_probe()
}

pub(super) fn run_live_s07_cli_replay_protection_probe() -> Result<(), String> {
    proof_replay_probes::run_live_s07_cli_replay_protection_probe()
}

pub(super) fn run_live_s08_cli_crash_recovery_probe() -> Result<(), String> {
    recovery_failover_probes::run_live_s08_cli_crash_recovery_probe()
}

pub(super) fn run_live_s09_cli_transport_failover_probe() -> Result<(), String> {
    recovery_failover_probes::run_live_s09_cli_transport_failover_probe()
}

pub(super) fn run_live_s10_cli_topology_coherence_probe() -> Result<(), String> {
    topology_coherence_probe::run_live_s10_cli_topology_coherence_probe()
}

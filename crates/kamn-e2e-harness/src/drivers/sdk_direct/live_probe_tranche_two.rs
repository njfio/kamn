use super::{KamnAgentHandle, KolmeProofReceipt, DEFAULT_AGENT_NAME, DEFAULT_KOLME_ENDPOINT};

#[path = "live_probe_tranche_two/message_query_support.rs"]
mod message_query_support;
#[path = "live_probe_tranche_two/proof_replay_probes.rs"]
mod proof_replay_probes;
#[path = "live_probe_tranche_two/recovery_failover_probes.rs"]
mod recovery_failover_probes;
#[path = "live_probe_tranche_two/topology_coherence_probe.rs"]
mod topology_coherence_probe;

const DEFAULT_S07_AGENT_NAME: &str = "kamn-e2e-sdk-s07";
const DEFAULT_S07_MESSAGE_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s07-replay"}"#;
const DEFAULT_S08_AGENT_NAME: &str = "kamn-e2e-sdk-s08";
const DEFAULT_S08_PRE_MESSAGE_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s08-pre"}"#;
const DEFAULT_S08_POST_MESSAGE_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s08-post"}"#;
const DEFAULT_S09_AGENT_NAME: &str = "kamn-e2e-sdk-s09";
const DEFAULT_S09_PRE_MESSAGE_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s09-pre"}"#;
const DEFAULT_S09_POST_MESSAGE_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s09-post"}"#;
const DEFAULT_S10_AGENT_NAME: &str = "kamn-e2e-sdk-s10";
const DEFAULT_S10_MESSAGE_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s10-topology"}"#;
const DEFAULT_S06_MESSAGE_ID: &str = "s06-live-proof";
const DEFAULT_S06_TX_HASH: &str = "sha256:s06-live-proof";
const DEFAULT_S06_BLOCK_HEIGHT: u64 = 1;
const DEFAULT_S06_FINALITY: &str = "final";

pub(super) use message_query_support::{
    validate_s08_distinct_message_ids, validate_s08_message_receipt_fields,
    validate_s08_query_message_response,
};

pub(super) fn default_agent_name() -> String {
    super::env_var_or_default("KAMN_AGENT_NAME", DEFAULT_AGENT_NAME)
}

pub(super) fn default_endpoint() -> String {
    super::env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080")
}

pub(super) fn kolme_endpoint() -> String {
    super::env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT)
}

pub(super) fn connect_agent(
    endpoint: &str,
    kolme_endpoint: &str,
    agent_name: &str,
    context: &str,
) -> Result<KamnAgentHandle, String> {
    KamnAgentHandle::connect(endpoint, kolme_endpoint, agent_name)
        .map_err(|error| format!("{context}: {error}"))
}

pub(super) fn validate_non_empty(value: &str, error: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(error.to_owned());
    }
    Ok(())
}

pub(super) fn run_live_s06_proof_verification_probe() -> Result<(), String> {
    proof_replay_probes::run_live_s06_proof_verification_probe()
}

pub(super) fn run_live_s07_replay_protection_probe() -> Result<(), String> {
    proof_replay_probes::run_live_s07_replay_protection_probe()
}

pub(super) fn run_live_s08_crash_recovery_probe() -> Result<(), String> {
    recovery_failover_probes::run_live_s08_crash_recovery_probe()
}

pub(super) fn run_live_s09_transport_failover_probe() -> Result<(), String> {
    recovery_failover_probes::run_live_s09_transport_failover_probe()
}

pub(super) fn run_live_s10_topology_coherence_probe() -> Result<(), String> {
    topology_coherence_probe::run_live_s10_topology_coherence_probe()
}

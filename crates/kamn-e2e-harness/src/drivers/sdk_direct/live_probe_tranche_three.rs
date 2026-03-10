use super::{KamnAgentHandle, KolmeProofReceipt, DEFAULT_KOLME_ENDPOINT};

#[path = "live_probe_tranche_three/batch_merkle_probe.rs"]
mod batch_merkle_probe;
#[path = "live_probe_tranche_three/bridge_forwarding_probe.rs"]
mod bridge_forwarding_probe;
#[path = "live_probe_tranche_three/live_probe_support.rs"]
mod live_probe_support;
#[path = "live_probe_tranche_three/performance_smoke_probe.rs"]
mod performance_smoke_probe;
#[path = "live_probe_tranche_three/retention_deletion_probe.rs"]
mod retention_deletion_probe;
#[path = "live_probe_tranche_three/signer_rotation_probe.rs"]
mod signer_rotation_probe;

const DEFAULT_S11_PRIMARY_AGENT_NAME: &str = "kamn-e2e-sdk-s11-primary";
const DEFAULT_S11_ROTATED_MESSAGE_PAYLOAD: &str =
    r#"{\"message\":\"sdk-direct-live-s11-rotated\"}"#;
const DEFAULT_S11_STALE_MESSAGE_PAYLOAD: &str = r#"{\"message\":\"sdk-direct-live-s11-stale\"}"#;
const DEFAULT_S11_MESSAGE_PAYLOAD: &str = r#"{\"message\":\"sdk-direct-live-s11-primary\"}"#;
const DEFAULT_S12_AGENT_NAME: &str = "kamn-e2e-sdk-s12";
const DEFAULT_S12_REGISTER_CONTENT_PAYLOAD: &str =
    r#"{\"content\":\"sdk-direct-live-s12\",\"retention_class\":\"standard\"}"#;
const DEFAULT_S13_AGENT_NAME: &str = "kamn-e2e-sdk-s13";
const DEFAULT_S13_SUBMIT_BRIDGE_PAYLOAD: &str =
    r#"{\"source_message_id\":\"sdk-direct-live-s13\",\"target_network\":\"testnet\"}"#;
const DEFAULT_S14_AGENT_NAME: &str = "kamn-e2e-sdk-s14";
const DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_A: &str =
    r#"{\"message\":\"sdk-direct-live-s14-batch-a\"}"#;
const DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_B: &str =
    r#"{\"message\":\"sdk-direct-live-s14-batch-b\"}"#;
const DEFAULT_S14_BLOCK_HEIGHT: u64 = 1;
const DEFAULT_S14_FINALITY: &str = "final";
const DEFAULT_S15_AGENT_NAME: &str = "kamn-e2e-sdk-s15";
const DEFAULT_S15_MESSAGE_PAYLOAD: &str = r#"{\"message\":\"sdk-direct-live-s15-performance\"}"#;
const DEFAULT_S15_ITERATIONS: u64 = 3;
const DEFAULT_S15_MAX_TOTAL_MILLIS: u128 = 5_000;
const DEFAULT_S15_MAX_P50_MILLIS: u128 = 2_500;
const DEFAULT_S15_MAX_P99_MILLIS: u128 = 5_000;

pub(super) use live_probe_support::validate_s14_proof_response;

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

pub(super) fn run_live_s11_signer_rotation_probe() -> Result<(), String> {
    signer_rotation_probe::run_live_s11_signer_rotation_probe()
}

pub(super) fn run_live_s12_retention_deletion_probe() -> Result<(), String> {
    retention_deletion_probe::run_live_s12_retention_deletion_probe()
}

pub(super) fn run_live_s13_bridge_forwarding_probe() -> Result<(), String> {
    bridge_forwarding_probe::run_live_s13_bridge_forwarding_probe()
}

pub(super) fn run_live_s14_batch_merkle_probe() -> Result<(), String> {
    batch_merkle_probe::run_live_s14_batch_merkle_probe()
}

pub(super) fn run_live_s15_performance_smoke_probe() -> Result<(), String> {
    performance_smoke_probe::run_live_s15_performance_smoke_probe()
}

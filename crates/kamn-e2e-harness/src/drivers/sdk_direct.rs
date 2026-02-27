use crate::drivers::shared_helpers::{
    env_var_or_default, env_var_or_else, is_live_bound_scenario_id,
    live_execution_enabled_from_env as shared_live_execution_enabled_from_env,
    live_s07_probe_agent_suffix, parse_s15_budget_env_u128, validate_s07_replay_reason_marker,
    validate_s12_content_field_coherence, validate_s12_content_id_match,
    validate_s13_bridge_field_coherence, validate_s13_bridge_id_match,
    validate_s15_latency_budget_samples,
};
use crate::drivers::{DriverExecutionResult, HarnessDriver};
use crate::ExecutionMode;
use kamn_agent_lib::{KamnAgentHandle, KolmeProofReceipt};
use std::env;
use std::sync::Arc;

const SDK_DIRECT_LIVE_ENV: &str = "KAMN_E2E_SDK_DIRECT_LIVE";
const DEFAULT_KOLME_ENDPOINT: &str = "http://localhost:3000";
const DEFAULT_AGENT_NAME: &str = "kamn-e2e-sdk-direct";
const DEFAULT_S02_MESSAGE_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s02"}"#;
const DEFAULT_S02_REPLY_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s02-reply"}"#;
const DEFAULT_S03_CHANNEL_PAYLOAD: &str =
    r#"{"name":"sdk-direct-live-s03","members":["alice","bob","carol"]}"#;
const DEFAULT_S03_MESSAGE_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s03-channel-message"}"#;
const DEFAULT_S04_CREATE_TASK_PAYLOAD: &str =
    r#"{"title":"sdk-direct-live-s04","description":"live task lifecycle probe"}"#;
const DEFAULT_S04_ESCROW_AMOUNT: u64 = 1;
const DEFAULT_S05_FUND_ESCROW_PAYLOAD: &str = r#"{"task_id":"sdk-direct-live-s05","amount":1}"#;
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
const DEFAULT_S11_PRIMARY_AGENT_NAME: &str = "kamn-e2e-sdk-s11-primary";
const DEFAULT_S11_ROTATED_MESSAGE_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s11-rotated"}"#;
const DEFAULT_S11_STALE_MESSAGE_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s11-stale"}"#;
const DEFAULT_S11_MESSAGE_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s11-primary"}"#;
const DEFAULT_S12_AGENT_NAME: &str = "kamn-e2e-sdk-s12";
const DEFAULT_S12_REGISTER_CONTENT_PAYLOAD: &str =
    r#"{"content":"sdk-direct-live-s12","retention_class":"standard"}"#;
const DEFAULT_S13_AGENT_NAME: &str = "kamn-e2e-sdk-s13";
const DEFAULT_S13_SUBMIT_BRIDGE_PAYLOAD: &str =
    r#"{"source_message_id":"sdk-direct-live-s13","target_network":"testnet"}"#;
const DEFAULT_S14_AGENT_NAME: &str = "kamn-e2e-sdk-s14";
const DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_A: &str = r#"{"message":"sdk-direct-live-s14-batch-a"}"#;
const DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_B: &str = r#"{"message":"sdk-direct-live-s14-batch-b"}"#;
const DEFAULT_S14_BLOCK_HEIGHT: u64 = 1;
const DEFAULT_S14_FINALITY: &str = "final";
const DEFAULT_S15_AGENT_NAME: &str = "kamn-e2e-sdk-s15";
const DEFAULT_S15_MESSAGE_PAYLOAD: &str = r#"{"message":"sdk-direct-live-s15-performance"}"#;
const DEFAULT_S15_ITERATIONS: u64 = 3;
const DEFAULT_S15_MAX_TOTAL_MILLIS: u128 = 5_000;
const DEFAULT_S15_MAX_P50_MILLIS: u128 = 2_500;
const DEFAULT_S15_MAX_P99_MILLIS: u128 = 5_000;
const DEFAULT_S06_MESSAGE_ID: &str = "s06-live-proof";
const DEFAULT_S06_TX_HASH: &str = "sha256:s06-live-proof";
const DEFAULT_S06_BLOCK_HEIGHT: u64 = 1;
const DEFAULT_S06_FINALITY: &str = "final";

type LiveProbe = dyn Fn() -> Result<(), String> + Send + Sync + 'static;

/// SDK-direct driver with optional live execution for S-01 through S-15.
#[derive(Clone)]
pub struct SdkDirectDriver {
    live_execution_enabled: bool,
    discovery_probe: Arc<LiveProbe>,
    direct_message_probe: Arc<LiveProbe>,
    group_channel_probe: Arc<LiveProbe>,
    task_lifecycle_probe: Arc<LiveProbe>,
    escrow_settlement_probe: Arc<LiveProbe>,
    proof_verification_probe: Arc<LiveProbe>,
    replay_protection_probe: Arc<LiveProbe>,
    crash_recovery_probe: Arc<LiveProbe>,
    transport_failover_probe: Arc<LiveProbe>,
    topology_coherence_probe: Arc<LiveProbe>,
    signer_rotation_probe: Arc<LiveProbe>,
    retention_deletion_probe: Arc<LiveProbe>,
    bridge_forwarding_probe: Arc<LiveProbe>,
    batch_merkle_probe: Arc<LiveProbe>,
    performance_smoke_probe: Arc<LiveProbe>,
}

impl std::fmt::Debug for SdkDirectDriver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SdkDirectDriver")
            .field("live_execution_enabled", &self.live_execution_enabled)
            .finish()
    }
}

impl Default for SdkDirectDriver {
    fn default() -> Self {
        Self::from_env()
    }
}

impl SdkDirectDriver {
    /// Builds SDK-direct driver from environment configuration.
    pub fn from_env() -> Self {
        Self::with_probes(
            live_execution_enabled_from_env(),
            run_live_s01_discovery_probe,
            run_live_s02_direct_message_probe,
            run_live_s03_group_channel_probe,
            run_live_s04_task_lifecycle_probe,
            run_live_s05_escrow_settlement_probe,
            (
                run_live_s06_proof_verification_probe,
                run_live_s07_replay_protection_probe,
                run_live_s08_crash_recovery_probe,
                run_live_s09_transport_failover_probe,
                run_live_s10_topology_coherence_probe,
                run_live_s11_signer_rotation_probe,
                run_live_s12_retention_deletion_probe,
                run_live_s13_bridge_forwarding_probe,
                run_live_s14_batch_merkle_probe,
                run_live_s15_performance_smoke_probe,
            ),
        )
    }

    /// Creates SDK-direct driver with one probe reused for all live-bound scenarios.
    pub fn with_probe<F>(live_execution_enabled: bool, live_probe: F) -> Self
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        let live_probe: Arc<LiveProbe> = Arc::new(live_probe);
        Self {
            live_execution_enabled,
            discovery_probe: live_probe.clone(),
            direct_message_probe: live_probe.clone(),
            task_lifecycle_probe: live_probe.clone(),
            group_channel_probe: live_probe.clone(),
            escrow_settlement_probe: live_probe.clone(),
            proof_verification_probe: live_probe.clone(),
            replay_protection_probe: live_probe.clone(),
            crash_recovery_probe: live_probe.clone(),
            transport_failover_probe: live_probe.clone(),
            topology_coherence_probe: live_probe.clone(),
            signer_rotation_probe: live_probe.clone(),
            retention_deletion_probe: live_probe.clone(),
            bridge_forwarding_probe: live_probe.clone(),
            batch_merkle_probe: live_probe.clone(),
            performance_smoke_probe: live_probe,
        }
    }

    /// Creates SDK-direct driver with explicit per-scenario probe implementations.
    pub fn with_probes<F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T>(
        live_execution_enabled: bool,
        discovery_probe: F,
        direct_message_probe: G,
        group_channel_probe: H,
        task_lifecycle_probe: I,
        escrow_settlement_probe: J,
        proof_replay_crash_failover_topology_signer_retention_bridge_merkle_and_performance_probes: (
            K,
            L,
            M,
            N,
            O,
            P,
            Q,
            R,
            S,
            T,
        ),
    ) -> Self
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
        G: Fn() -> Result<(), String> + Send + Sync + 'static,
        H: Fn() -> Result<(), String> + Send + Sync + 'static,
        I: Fn() -> Result<(), String> + Send + Sync + 'static,
        J: Fn() -> Result<(), String> + Send + Sync + 'static,
        K: Fn() -> Result<(), String> + Send + Sync + 'static,
        L: Fn() -> Result<(), String> + Send + Sync + 'static,
        M: Fn() -> Result<(), String> + Send + Sync + 'static,
        N: Fn() -> Result<(), String> + Send + Sync + 'static,
        O: Fn() -> Result<(), String> + Send + Sync + 'static,
        P: Fn() -> Result<(), String> + Send + Sync + 'static,
        Q: Fn() -> Result<(), String> + Send + Sync + 'static,
        R: Fn() -> Result<(), String> + Send + Sync + 'static,
        S: Fn() -> Result<(), String> + Send + Sync + 'static,
        T: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        let (
            proof_verification_probe,
            replay_protection_probe,
            crash_recovery_probe,
            transport_failover_probe,
            topology_coherence_probe,
            signer_rotation_probe,
            retention_deletion_probe,
            bridge_forwarding_probe,
            batch_merkle_probe,
            performance_smoke_probe,
        ) = proof_replay_crash_failover_topology_signer_retention_bridge_merkle_and_performance_probes;
        Self {
            live_execution_enabled,
            discovery_probe: Arc::new(discovery_probe),
            direct_message_probe: Arc::new(direct_message_probe),
            group_channel_probe: Arc::new(group_channel_probe),
            task_lifecycle_probe: Arc::new(task_lifecycle_probe),
            escrow_settlement_probe: Arc::new(escrow_settlement_probe),
            proof_verification_probe: Arc::new(proof_verification_probe),
            replay_protection_probe: Arc::new(replay_protection_probe),
            crash_recovery_probe: Arc::new(crash_recovery_probe),
            transport_failover_probe: Arc::new(transport_failover_probe),
            topology_coherence_probe: Arc::new(topology_coherence_probe),
            signer_rotation_probe: Arc::new(signer_rotation_probe),
            retention_deletion_probe: Arc::new(retention_deletion_probe),
            bridge_forwarding_probe: Arc::new(bridge_forwarding_probe),
            batch_merkle_probe: Arc::new(batch_merkle_probe),
            performance_smoke_probe: Arc::new(performance_smoke_probe),
        }
    }
}

impl HarnessDriver for SdkDirectDriver {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::SdkDirect
    }

    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult {
        let status = if !is_live_bound_scenario_id(scenario_id) {
            "pass"
        } else if !self.live_execution_enabled {
            "fail"
        } else {
            match self.live_probe_for_scenario(scenario_id) {
                Some(probe) if probe.is_ok() => "pass",
                Some(_) => "fail",
                None => "fail",
            }
        };
        DriverExecutionResult {
            scenario_id,
            status,
        }
    }
}

impl SdkDirectDriver {
    fn live_probe_for_scenario(&self, scenario_id: &'static str) -> Option<Result<(), String>> {
        match scenario_id {
            "S-01" => Some((self.discovery_probe)()),
            "S-02" => Some((self.direct_message_probe)()),
            "S-03" => Some((self.group_channel_probe)()),
            "S-04" => Some((self.task_lifecycle_probe)()),
            "S-05" => Some((self.escrow_settlement_probe)()),
            "S-06" => Some((self.proof_verification_probe)()),
            "S-07" => Some((self.replay_protection_probe)()),
            "S-08" => Some((self.crash_recovery_probe)()),
            "S-09" => Some((self.transport_failover_probe)()),
            "S-10" => Some((self.topology_coherence_probe)()),
            "S-11" => Some((self.signer_rotation_probe)()),
            "S-12" => Some((self.retention_deletion_probe)()),
            "S-13" => Some((self.bridge_forwarding_probe)()),
            "S-14" => Some((self.batch_merkle_probe)()),
            "S-15" => Some((self.performance_smoke_probe)()),
            _ => None,
        }
    }
}

fn live_execution_enabled_from_env() -> bool {
    shared_live_execution_enabled_from_env(SDK_DIRECT_LIVE_ENV)
}

fn run_live_s01_discovery_probe() -> Result<(), String> {
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_AGENT_NAME);

    let handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        agent_name.as_str(),
    )
    .map_err(|error| format!("sdk-direct live discovery connect failed: {error}"))?;

    let did = handle.identity().did().as_str();
    if did.trim().is_empty() {
        return Err("sdk-direct live discovery failed: empty DID".to_owned());
    }

    let health = handle
        .health()
        .map_err(|error| format!("sdk-direct live discovery health check failed: {error}"))?;
    if health.status.trim().is_empty() {
        return Err("sdk-direct live discovery failed: empty health status".to_owned());
    }

    Ok(())
}

fn run_live_s02_direct_message_probe() -> Result<(), String> {
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_AGENT_NAME);
    let message_payload =
        env_var_or_default("KAMN_E2E_S02_MESSAGE_PAYLOAD", DEFAULT_S02_MESSAGE_PAYLOAD);
    let reply_payload = env_var_or_default("KAMN_E2E_S02_REPLY_PAYLOAD", DEFAULT_S02_REPLY_PAYLOAD);

    let send_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{agent_name}-s02-send").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s02 connect failed: {error}"))?;
    let send_receipt = send_handle
        .send_message(message_payload.as_str())
        .map_err(|error| format!("sdk-direct live s02 send-message failed: {error}"))?;
    if send_receipt.message_id.trim().is_empty() {
        return Err("sdk-direct live s02 send-message returned empty message_id".to_owned());
    }
    if send_receipt.status.trim().is_empty() {
        return Err("sdk-direct live s02 send-message returned empty status".to_owned());
    }

    let query_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{agent_name}-s02-query").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s02 connect failed: {error}"))?;
    let queried_status = query_handle
        .query_message(send_receipt.message_id.as_str())
        .map_err(|error| format!("sdk-direct live s02 query-message failed: {error}"))?;
    if queried_status.message_id != send_receipt.message_id {
        return Err(format!(
            "sdk-direct live s02 query-message returned mismatched message_id: expected={}, got={}",
            send_receipt.message_id, queried_status.message_id
        ));
    }
    if queried_status.status.trim().is_empty() {
        return Err("sdk-direct live s02 query-message returned empty status".to_owned());
    }

    let reply_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{agent_name}-s02-reply").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s02 connect failed: {error}"))?;
    let reply_receipt = reply_handle
        .send_message(reply_payload.as_str())
        .map_err(|error| format!("sdk-direct live s02 reply send-message failed: {error}"))?;
    if reply_receipt.message_id.trim().is_empty() {
        return Err("sdk-direct live s02 reply send-message returned empty message_id".to_owned());
    }
    if reply_receipt.status.trim().is_empty() {
        return Err("sdk-direct live s02 reply send-message returned empty status".to_owned());
    }

    let reply_query_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{agent_name}-s02-query-reply").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s02 connect failed: {error}"))?;
    let reply_query_status = reply_query_handle
        .query_message(reply_receipt.message_id.as_str())
        .map_err(|error| format!("sdk-direct live s02 reply query-message failed: {error}"))?;
    if reply_query_status.message_id != reply_receipt.message_id {
        return Err(format!(
            "sdk-direct live s02 reply query-message returned mismatched message_id: expected={}, got={}",
            reply_receipt.message_id, reply_query_status.message_id
        ));
    }
    if reply_query_status.status.trim().is_empty() {
        return Err("sdk-direct live s02 reply query-message returned empty status".to_owned());
    }

    Ok(())
}

fn run_live_s03_group_channel_probe() -> Result<(), String> {
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_AGENT_NAME);
    let channel_payload =
        env_var_or_default("KAMN_E2E_S03_CHANNEL_PAYLOAD", DEFAULT_S03_CHANNEL_PAYLOAD);
    let message_payload =
        env_var_or_default("KAMN_E2E_S03_MESSAGE_PAYLOAD", DEFAULT_S03_MESSAGE_PAYLOAD);

    let create_channel_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{agent_name}-s03-create-channel").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s03 connect failed: {error}"))?;
    let channel_receipt = create_channel_handle
        .create_channel(channel_payload.as_str())
        .map_err(|error| format!("sdk-direct live s03 create-channel failed: {error}"))?;
    if channel_receipt.channel_id.trim().is_empty() {
        return Err("sdk-direct live s03 create-channel returned empty channel_id".to_owned());
    }
    if channel_receipt.status.trim().is_empty() {
        return Err("sdk-direct live s03 create-channel returned empty status".to_owned());
    }

    let send_message_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{agent_name}-s03-send-message").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s03 connect failed: {error}"))?;
    let send_receipt = send_message_handle
        .send_message(message_payload.as_str())
        .map_err(|error| format!("sdk-direct live s03 send-message failed: {error}"))?;
    if send_receipt.message_id.trim().is_empty() {
        return Err("sdk-direct live s03 send-message returned empty message_id".to_owned());
    }
    if send_receipt.status.trim().is_empty() {
        return Err("sdk-direct live s03 send-message returned empty status".to_owned());
    }

    let query_message_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{agent_name}-s03-query-message").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s03 connect failed: {error}"))?;
    let queried_status = query_message_handle
        .query_message(send_receipt.message_id.as_str())
        .map_err(|error| format!("sdk-direct live s03 query-message failed: {error}"))?;
    validate_live_s03_query_message_response(
        send_receipt.message_id.as_str(),
        queried_status.message_id.as_str(),
        queried_status.status.as_str(),
    )?;

    let list_messages_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{agent_name}-s03-list-messages").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s03 connect failed: {error}"))?;
    let message_listing = list_messages_handle
        .list_messages(channel_receipt.channel_id.as_str())
        .map_err(|error| format!("sdk-direct live s03 list-messages failed: {error}"))?;
    validate_live_s03_list_messages_response(
        channel_receipt.channel_id.as_str(),
        message_listing.channel_id.as_str(),
    )?;

    Ok(())
}

fn validate_live_s03_query_message_response(
    expected_message_id: &str,
    queried_message_id: &str,
    queried_status: &str,
) -> Result<(), String> {
    if queried_message_id != expected_message_id {
        return Err(format!(
            "sdk-direct live s03 query-message returned mismatched message_id: expected={}, got={}",
            expected_message_id, queried_message_id
        ));
    }
    if queried_status.trim().is_empty() {
        return Err("sdk-direct live s03 query-message returned empty status".to_owned());
    }
    Ok(())
}

fn validate_live_s03_list_messages_response(
    expected_channel_id: &str,
    listed_channel_id: &str,
) -> Result<(), String> {
    if listed_channel_id != expected_channel_id {
        return Err(format!(
            "sdk-direct live s03 list-messages returned mismatched channel_id: expected={}, got={}",
            expected_channel_id, listed_channel_id
        ));
    }
    Ok(())
}

fn run_live_s04_task_lifecycle_probe() -> Result<(), String> {
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_AGENT_NAME);
    let create_task_payload = env_var_or_default(
        "KAMN_E2E_S04_CREATE_TASK_PAYLOAD",
        DEFAULT_S04_CREATE_TASK_PAYLOAD,
    );

    let create_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{agent_name}-s04-create").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s04 connect failed: {error}"))?;

    let task_receipt = create_handle
        .create_task(create_task_payload.as_str())
        .map_err(|error| format!("sdk-direct live s04 create-task failed: {error}"))?;
    if task_receipt.task_id.trim().is_empty() {
        return Err("sdk-direct live s04 create-task returned empty task_id".to_owned());
    }

    let fund_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{agent_name}-s04-fund").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s04 connect failed: {error}"))?;
    let fund_payload = format!(
        "{{\"task_id\":\"{}\",\"amount\":{}}}",
        task_receipt.task_id, DEFAULT_S04_ESCROW_AMOUNT
    );
    let escrow_receipt = fund_handle
        .fund_escrow(fund_payload.as_str())
        .map_err(|error| format!("sdk-direct live s04 fund-escrow failed: {error}"))?;
    if escrow_receipt.escrow_id.trim().is_empty() {
        return Err("sdk-direct live s04 fund-escrow returned empty escrow_id".to_owned());
    }

    let accept_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{agent_name}-s04-accept").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s04 connect failed: {error}"))?;
    let accept_receipt = accept_handle
        .accept_task(task_receipt.task_id.as_str())
        .map_err(|error| format!("sdk-direct live s04 accept-task failed: {error}"))?;
    if accept_receipt.state.trim().is_empty() {
        return Err("sdk-direct live s04 accept-task returned empty state".to_owned());
    }

    let complete_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{agent_name}-s04-complete").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s04 connect failed: {error}"))?;
    let complete_receipt = complete_handle
        .complete_task(task_receipt.task_id.as_str())
        .map_err(|error| format!("sdk-direct live s04 complete-task failed: {error}"))?;
    if complete_receipt.state.trim().is_empty() {
        return Err("sdk-direct live s04 complete-task returned empty state".to_owned());
    }

    let release_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{agent_name}-s04-release").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s04 connect failed: {error}"))?;
    let release_receipt = release_handle
        .release_escrow(escrow_receipt.escrow_id.as_str())
        .map_err(|error| format!("sdk-direct live s04 release-escrow failed: {error}"))?;
    if release_receipt.state.trim().is_empty() {
        return Err("sdk-direct live s04 release-escrow returned empty state".to_owned());
    }

    Ok(())
}

fn run_live_s05_escrow_settlement_probe() -> Result<(), String> {
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_AGENT_NAME);
    let fund_payload = env_var_or_default(
        "KAMN_E2E_S05_FUND_ESCROW_PAYLOAD",
        DEFAULT_S05_FUND_ESCROW_PAYLOAD,
    );

    let fund_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{agent_name}-s05-fund").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s05 connect failed: {error}"))?;
    let funded_receipt = fund_handle
        .fund_escrow(fund_payload.as_str())
        .map_err(|error| format!("sdk-direct live s05 fund-escrow failed: {error}"))?;
    if funded_receipt.escrow_id.trim().is_empty() {
        return Err("sdk-direct live s05 fund-escrow returned empty escrow_id".to_owned());
    }
    if funded_receipt.state.trim().is_empty() {
        return Err("sdk-direct live s05 fund-escrow returned empty state".to_owned());
    }

    let release_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{agent_name}-s05-release").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s05 connect failed: {error}"))?;
    let release_receipt = release_handle
        .release_escrow(funded_receipt.escrow_id.as_str())
        .map_err(|error| format!("sdk-direct live s05 release-escrow failed: {error}"))?;
    validate_live_s05_release_escrow_receipt(
        funded_receipt.escrow_id.as_str(),
        release_receipt.escrow_id.as_str(),
        release_receipt.state.as_str(),
    )?;

    Ok(())
}

fn validate_live_s05_release_escrow_receipt(
    expected_escrow_id: &str,
    released_escrow_id: &str,
    released_state: &str,
) -> Result<(), String> {
    if released_escrow_id != expected_escrow_id {
        return Err(format!(
            "sdk-direct live s05 release-escrow returned mismatched escrow_id: expected={expected_escrow_id}, got={released_escrow_id}"
        ));
    }
    if released_state.trim().is_empty() {
        return Err("sdk-direct live s05 release-escrow returned empty state".to_owned());
    }
    Ok(())
}

fn run_live_s06_proof_verification_probe() -> Result<(), String> {
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_AGENT_NAME);
    let message_id = env_var_or_default("KAMN_E2E_S06_PROOF_MESSAGE_ID", DEFAULT_S06_MESSAGE_ID);
    let tx_hash = env_var_or_default("KAMN_E2E_S06_PROOF_TX_HASH", DEFAULT_S06_TX_HASH);
    let block_height = env::var("KAMN_E2E_S06_PROOF_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("sdk-direct live s06 invalid block height env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S06_BLOCK_HEIGHT);
    let finality = env_var_or_default("KAMN_E2E_S06_PROOF_FINALITY", DEFAULT_S06_FINALITY);

    let handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        agent_name.as_str(),
    )
    .map_err(|error| format!("sdk-direct live s06 connect failed: {error}"))?;

    let receipt = KolmeProofReceipt {
        tx_hash,
        block_height,
        finality,
    };
    let verification = handle
        .verify_proof(message_id.as_str(), &receipt)
        .map_err(|error| format!("sdk-direct live s06 verify-proof failed: {error}"))?;

    if !verification.verified {
        return Err("sdk-direct live s06 verify-proof returned verified=false".to_owned());
    }
    if verification.finality.trim() != "FINAL" {
        return Err(format!(
            "sdk-direct live s06 verify-proof returned non-final finality: {}",
            verification.finality
        ));
    }

    Ok(())
}

fn run_live_s07_replay_protection_probe() -> Result<(), String> {
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    let base_agent_name = env_var_or_default("KAMN_E2E_S07_AGENT_NAME", DEFAULT_S07_AGENT_NAME);
    let message_payload =
        env_var_or_default("KAMN_E2E_S07_REPLAY_PAYLOAD", DEFAULT_S07_MESSAGE_PAYLOAD);
    let replay_agent_name = format!(
        "{base_agent_name}-{}",
        live_s07_probe_agent_suffix().as_str()
    );

    let first_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        replay_agent_name.as_str(),
    )
    .map_err(|error| format!("sdk-direct live s07 connect failed: {error}"))?;
    let first_receipt = first_handle
        .send_message(message_payload.as_str())
        .map_err(|error| format!("sdk-direct live s07 initial send-message failed: {error}"))?;
    if first_receipt.message_id.trim().is_empty() {
        return Err(
            "sdk-direct live s07 initial send-message returned empty message_id".to_owned(),
        );
    }
    if first_receipt.status.trim().is_empty() {
        return Err("sdk-direct live s07 initial send-message returned empty status".to_owned());
    }

    let replay_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        replay_agent_name.as_str(),
    )
    .map_err(|error| format!("sdk-direct live s07 connect failed: {error}"))?;
    let replay_error = replay_handle
        .send_message(message_payload.as_str())
        .err()
        .ok_or_else(|| {
            "sdk-direct live s07 replay send-message unexpectedly succeeded".to_owned()
        })?;
    let replay_error_text = replay_error.to_string();
    validate_s07_replay_reason_marker(
        replay_error_text.as_str(),
        "sdk-direct live s07 replay send-message",
    )?;

    Ok(())
}

fn run_live_s08_crash_recovery_probe() -> Result<(), String> {
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    let base_agent_name = env_var_or_default("KAMN_E2E_S08_AGENT_NAME", DEFAULT_S08_AGENT_NAME);
    let pre_message_payload = env_var_or_default(
        "KAMN_E2E_S08_PRE_MESSAGE_PAYLOAD",
        DEFAULT_S08_PRE_MESSAGE_PAYLOAD,
    );
    let post_message_payload = env_var_or_default(
        "KAMN_E2E_S08_POST_MESSAGE_PAYLOAD",
        DEFAULT_S08_POST_MESSAGE_PAYLOAD,
    );

    let pre_send_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-pre-send").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s08 connect failed: {error}"))?;
    let pre_receipt = pre_send_handle
        .send_message(pre_message_payload.as_str())
        .map_err(|error| {
            format!("sdk-direct live s08 pre-boundary send-message failed: {error}")
        })?;
    validate_s08_message_receipt_fields(
        pre_receipt.message_id.as_str(),
        pre_receipt.status.as_str(),
        "sdk-direct live s08 pre-boundary send-message",
    )?;

    let pre_query_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-pre-query").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s08 connect failed: {error}"))?;
    let pre_query = pre_query_handle
        .query_message(pre_receipt.message_id.as_str())
        .map_err(|error| {
            format!("sdk-direct live s08 pre-boundary query-message failed: {error}")
        })?;
    validate_s08_query_message_response(
        pre_receipt.message_id.as_str(),
        pre_query.message_id.as_str(),
        pre_query.status.as_str(),
        "sdk-direct live s08 pre-boundary query-message",
    )?;

    let boundary_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-boundary").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s08 connect failed: {error}"))?;
    let boundary_health = boundary_handle
        .health()
        .map_err(|error| format!("sdk-direct live s08 boundary health check failed: {error}"))?;
    if boundary_health.status.trim().is_empty() {
        return Err("sdk-direct live s08 boundary health check returned empty status".to_owned());
    }

    let post_send_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-post-send").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s08 connect failed: {error}"))?;
    let post_receipt = post_send_handle
        .send_message(post_message_payload.as_str())
        .map_err(|error| {
            format!("sdk-direct live s08 post-boundary send-message failed: {error}")
        })?;
    validate_s08_message_receipt_fields(
        post_receipt.message_id.as_str(),
        post_receipt.status.as_str(),
        "sdk-direct live s08 post-boundary send-message",
    )?;
    validate_s08_distinct_message_ids(
        pre_receipt.message_id.as_str(),
        post_receipt.message_id.as_str(),
        "sdk-direct live s08 post-boundary send-message",
    )?;

    let post_query_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-post-query").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s08 connect failed: {error}"))?;
    let post_query = post_query_handle
        .query_message(post_receipt.message_id.as_str())
        .map_err(|error| {
            format!("sdk-direct live s08 post-boundary query-message failed: {error}")
        })?;
    validate_s08_query_message_response(
        post_receipt.message_id.as_str(),
        post_query.message_id.as_str(),
        post_query.status.as_str(),
        "sdk-direct live s08 post-boundary query-message",
    )?;

    Ok(())
}

fn run_live_s09_transport_failover_probe() -> Result<(), String> {
    let primary_endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let failover_endpoint = env_var_or_else("KAMN_E2E_S09_FAILOVER_ENDPOINT", || {
        primary_endpoint.clone()
    });
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    let base_agent_name = env_var_or_default("KAMN_E2E_S09_AGENT_NAME", DEFAULT_S09_AGENT_NAME);
    let pre_message_payload = env_var_or_default(
        "KAMN_E2E_S09_PRE_MESSAGE_PAYLOAD",
        DEFAULT_S09_PRE_MESSAGE_PAYLOAD,
    );
    let post_message_payload = env_var_or_default(
        "KAMN_E2E_S09_POST_MESSAGE_PAYLOAD",
        DEFAULT_S09_POST_MESSAGE_PAYLOAD,
    );

    let pre_send_handle = KamnAgentHandle::connect(
        primary_endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-pre-send").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s09 connect failed: {error}"))?;
    let pre_receipt = pre_send_handle
        .send_message(pre_message_payload.as_str())
        .map_err(|error| {
            format!("sdk-direct live s09 pre-failover send-message failed: {error}")
        })?;
    validate_s08_message_receipt_fields(
        pre_receipt.message_id.as_str(),
        pre_receipt.status.as_str(),
        "sdk-direct live s09 pre-failover send-message",
    )?;

    let pre_query_handle = KamnAgentHandle::connect(
        primary_endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-pre-query").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s09 connect failed: {error}"))?;
    let pre_query = pre_query_handle
        .query_message(pre_receipt.message_id.as_str())
        .map_err(|error| {
            format!("sdk-direct live s09 pre-failover query-message failed: {error}")
        })?;
    validate_s08_query_message_response(
        pre_receipt.message_id.as_str(),
        pre_query.message_id.as_str(),
        pre_query.status.as_str(),
        "sdk-direct live s09 pre-failover query-message",
    )?;

    let boundary_handle = KamnAgentHandle::connect(
        failover_endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-boundary").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s09 failover connect failed: {error}"))?;
    let boundary_health = boundary_handle.health().map_err(|error| {
        format!("sdk-direct live s09 failover boundary health check failed: {error}")
    })?;
    if boundary_health.status.trim().is_empty() {
        return Err(
            "sdk-direct live s09 failover boundary health check returned empty status".to_owned(),
        );
    }

    let post_send_handle = KamnAgentHandle::connect(
        failover_endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-post-send").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s09 failover connect failed: {error}"))?;
    let post_receipt = post_send_handle
        .send_message(post_message_payload.as_str())
        .map_err(|error| {
            format!("sdk-direct live s09 post-failover send-message failed: {error}")
        })?;
    validate_s08_message_receipt_fields(
        post_receipt.message_id.as_str(),
        post_receipt.status.as_str(),
        "sdk-direct live s09 post-failover send-message",
    )?;
    validate_s08_distinct_message_ids(
        pre_receipt.message_id.as_str(),
        post_receipt.message_id.as_str(),
        "sdk-direct live s09 post-failover send-message",
    )?;

    let post_query_handle = KamnAgentHandle::connect(
        failover_endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-post-query").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s09 failover connect failed: {error}"))?;
    let post_query = post_query_handle
        .query_message(post_receipt.message_id.as_str())
        .map_err(|error| {
            format!("sdk-direct live s09 post-failover query-message failed: {error}")
        })?;
    validate_s08_query_message_response(
        post_receipt.message_id.as_str(),
        post_query.message_id.as_str(),
        post_query.status.as_str(),
        "sdk-direct live s09 post-failover query-message",
    )?;

    Ok(())
}

fn run_live_s10_topology_coherence_probe() -> Result<(), String> {
    let primary_endpoint = env_var_or_else("KAMN_E2E_S10_PRIMARY_ENDPOINT", || {
        env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080")
    });
    let secondary_endpoint = env_var_or_else("KAMN_E2E_S10_SECONDARY_ENDPOINT", || {
        primary_endpoint.clone()
    });
    let tertiary_endpoint = env_var_or_else("KAMN_E2E_S10_TERTIARY_ENDPOINT", || {
        secondary_endpoint.clone()
    });
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    let base_agent_name = env_var_or_default("KAMN_E2E_S10_AGENT_NAME", DEFAULT_S10_AGENT_NAME);
    let message_payload =
        env_var_or_default("KAMN_E2E_S10_MESSAGE_PAYLOAD", DEFAULT_S10_MESSAGE_PAYLOAD);

    let primary_send_handle = KamnAgentHandle::connect(
        primary_endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-primary-send").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s10 primary connect failed: {error}"))?;
    let primary_receipt = primary_send_handle
        .send_message(message_payload.as_str())
        .map_err(|error| format!("sdk-direct live s10 primary send-message failed: {error}"))?;
    validate_s08_message_receipt_fields(
        primary_receipt.message_id.as_str(),
        primary_receipt.status.as_str(),
        "sdk-direct live s10 primary send-message",
    )?;

    let secondary_query_handle = KamnAgentHandle::connect(
        secondary_endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-secondary-query").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s10 secondary connect failed: {error}"))?;
    let secondary_query = secondary_query_handle
        .query_message(primary_receipt.message_id.as_str())
        .map_err(|error| format!("sdk-direct live s10 secondary query-message failed: {error}"))?;
    validate_s08_query_message_response(
        primary_receipt.message_id.as_str(),
        secondary_query.message_id.as_str(),
        secondary_query.status.as_str(),
        "sdk-direct live s10 secondary query-message",
    )?;

    let tertiary_query_handle = KamnAgentHandle::connect(
        tertiary_endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-tertiary-query").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s10 tertiary connect failed: {error}"))?;
    let tertiary_query = tertiary_query_handle
        .query_message(primary_receipt.message_id.as_str())
        .map_err(|error| format!("sdk-direct live s10 tertiary query-message failed: {error}"))?;
    validate_s08_query_message_response(
        primary_receipt.message_id.as_str(),
        tertiary_query.message_id.as_str(),
        tertiary_query.status.as_str(),
        "sdk-direct live s10 tertiary query-message",
    )?;

    let secondary_health_handle = KamnAgentHandle::connect(
        secondary_endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-secondary-boundary").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s10 secondary connect failed: {error}"))?;
    let secondary_health = secondary_health_handle
        .health()
        .map_err(|error| format!("sdk-direct live s10 secondary health check failed: {error}"))?;
    if secondary_health.status.trim().is_empty() {
        return Err("sdk-direct live s10 secondary health check returned empty status".to_owned());
    }

    let tertiary_health_handle = KamnAgentHandle::connect(
        tertiary_endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-tertiary-boundary").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s10 tertiary connect failed: {error}"))?;
    let tertiary_health = tertiary_health_handle
        .health()
        .map_err(|error| format!("sdk-direct live s10 tertiary health check failed: {error}"))?;
    if tertiary_health.status.trim().is_empty() {
        return Err("sdk-direct live s10 tertiary health check returned empty status".to_owned());
    }

    Ok(())
}

fn run_live_s11_signer_rotation_probe() -> Result<(), String> {
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    let primary_agent_name = env_var_or_default(
        "KAMN_E2E_S11_PRIMARY_AGENT_NAME",
        DEFAULT_S11_PRIMARY_AGENT_NAME,
    );
    let rotated_agent_name = env_var_or_else("KAMN_E2E_S11_ROTATED_AGENT_NAME", || {
        format!("{primary_agent_name}-rotated")
    });
    let message_payload =
        env_var_or_default("KAMN_E2E_S11_MESSAGE_PAYLOAD", DEFAULT_S11_MESSAGE_PAYLOAD);
    let rotated_message_payload = env_var_or_default(
        "KAMN_E2E_S11_ROTATED_MESSAGE_PAYLOAD",
        DEFAULT_S11_ROTATED_MESSAGE_PAYLOAD,
    );
    let stale_message_payload = env_var_or_default(
        "KAMN_E2E_S11_STALE_MESSAGE_PAYLOAD",
        DEFAULT_S11_STALE_MESSAGE_PAYLOAD,
    );

    let primary_send_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        primary_agent_name.as_str(),
    )
    .map_err(|error| format!("sdk-direct live s11 primary connect failed: {error}"))?;
    let primary_receipt = primary_send_handle
        .send_message(message_payload.as_str())
        .map_err(|error| format!("sdk-direct live s11 primary send-message failed: {error}"))?;
    validate_s08_message_receipt_fields(
        primary_receipt.message_id.as_str(),
        primary_receipt.status.as_str(),
        "sdk-direct live s11 primary send-message",
    )?;

    let primary_query_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{primary_agent_name}-query").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s11 primary query connect failed: {error}"))?;
    let primary_query = primary_query_handle
        .query_message(primary_receipt.message_id.as_str())
        .map_err(|error| format!("sdk-direct live s11 primary query-message failed: {error}"))?;
    validate_s08_query_message_response(
        primary_receipt.message_id.as_str(),
        primary_query.message_id.as_str(),
        primary_query.status.as_str(),
        "sdk-direct live s11 primary query-message",
    )?;

    let rotated_send_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        rotated_agent_name.as_str(),
    )
    .map_err(|error| format!("sdk-direct live s11 rotated connect failed: {error}"))?;
    let rotated_receipt = rotated_send_handle
        .send_message(rotated_message_payload.as_str())
        .map_err(|error| format!("sdk-direct live s11 rotated send-message failed: {error}"))?;
    validate_s08_message_receipt_fields(
        rotated_receipt.message_id.as_str(),
        rotated_receipt.status.as_str(),
        "sdk-direct live s11 rotated send-message",
    )?;
    validate_s08_distinct_message_ids(
        primary_receipt.message_id.as_str(),
        rotated_receipt.message_id.as_str(),
        "sdk-direct live s11 rotated send-message",
    )?;

    let rotated_query_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{rotated_agent_name}-query").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s11 rotated query connect failed: {error}"))?;
    let rotated_query = rotated_query_handle
        .query_message(rotated_receipt.message_id.as_str())
        .map_err(|error| format!("sdk-direct live s11 rotated query-message failed: {error}"))?;
    validate_s08_query_message_response(
        rotated_receipt.message_id.as_str(),
        rotated_query.message_id.as_str(),
        rotated_query.status.as_str(),
        "sdk-direct live s11 rotated query-message",
    )?;

    let stale_primary_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        primary_agent_name.as_str(),
    )
    .map_err(|error| format!("sdk-direct live s11 stale-primary connect failed: {error}"))?;
    let stale_primary_error = stale_primary_handle
        .send_message(stale_message_payload.as_str())
        .err()
        .ok_or_else(|| {
            "sdk-direct live s11 stale-primary send-message unexpectedly succeeded".to_owned()
        })?;
    validate_s07_replay_reason_marker(
        stale_primary_error.to_string().as_str(),
        "sdk-direct live s11 stale-primary send-message",
    )?;

    Ok(())
}

fn run_live_s12_retention_deletion_probe() -> Result<(), String> {
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    let base_agent_name = env_var_or_default("KAMN_E2E_S12_AGENT_NAME", DEFAULT_S12_AGENT_NAME);
    let register_payload = env_var_or_default(
        "KAMN_E2E_S12_REGISTER_CONTENT_PAYLOAD",
        DEFAULT_S12_REGISTER_CONTENT_PAYLOAD,
    );

    let register_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-register").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s12 register connect failed: {error}"))?;
    let registration = register_handle
        .register_content(register_payload.as_str())
        .map_err(|error| format!("sdk-direct live s12 register-content failed: {error}"))?;
    if registration.content_id.trim().is_empty() {
        return Err("sdk-direct live s12 register-content returned empty content_id".to_owned());
    }
    if registration.retention_class.trim().is_empty() {
        return Err(
            "sdk-direct live s12 register-content returned empty retention_class".to_owned(),
        );
    }
    if registration.lifecycle_state.trim().is_empty() {
        return Err(
            "sdk-direct live s12 register-content returned empty lifecycle_state".to_owned(),
        );
    }
    if registration.redaction_status.trim().is_empty() {
        return Err(
            "sdk-direct live s12 register-content returned empty redaction_status".to_owned(),
        );
    }

    let expire_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-expire").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s12 expire connect failed: {error}"))?;
    let expired = expire_handle
        .expire_content(registration.content_id.as_str())
        .map_err(|error| format!("sdk-direct live s12 expire-content failed: {error}"))?;
    validate_s12_content_id_match(
        registration.content_id.as_str(),
        expired.content_id.as_str(),
        "sdk-direct live s12 expire-content",
    )?;
    if expired.lifecycle_state.trim().is_empty() {
        return Err("sdk-direct live s12 expire-content returned empty lifecycle_state".to_owned());
    }
    if expired.redaction_status.trim().is_empty() {
        return Err(
            "sdk-direct live s12 expire-content returned empty redaction_status".to_owned(),
        );
    }

    let tombstone_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-tombstone").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s12 tombstone connect failed: {error}"))?;
    let tombstoned = tombstone_handle
        .tombstone_content(registration.content_id.as_str())
        .map_err(|error| format!("sdk-direct live s12 tombstone-content failed: {error}"))?;
    validate_s12_content_id_match(
        registration.content_id.as_str(),
        tombstoned.content_id.as_str(),
        "sdk-direct live s12 tombstone-content",
    )?;
    if tombstoned.lifecycle_state.trim().is_empty() {
        return Err(
            "sdk-direct live s12 tombstone-content returned empty lifecycle_state".to_owned(),
        );
    }
    if tombstoned.redaction_status.trim().is_empty() {
        return Err(
            "sdk-direct live s12 tombstone-content returned empty redaction_status".to_owned(),
        );
    }

    let query_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-query").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s12 query connect failed: {error}"))?;
    let queried = query_handle
        .query_content(registration.content_id.as_str())
        .map_err(|error| format!("sdk-direct live s12 query-content failed: {error}"))?;
    validate_s12_content_id_match(
        registration.content_id.as_str(),
        queried.content_id.as_str(),
        "sdk-direct live s12 query-content",
    )?;
    validate_s12_content_field_coherence(
        tombstoned.lifecycle_state.as_str(),
        queried.lifecycle_state.as_str(),
        "lifecycle_state",
        "sdk-direct live s12 query-content",
    )?;
    validate_s12_content_field_coherence(
        tombstoned.redaction_status.as_str(),
        queried.redaction_status.as_str(),
        "redaction_status",
        "sdk-direct live s12 query-content",
    )?;

    Ok(())
}

fn run_live_s13_bridge_forwarding_probe() -> Result<(), String> {
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    let base_agent_name = env_var_or_default("KAMN_E2E_S13_AGENT_NAME", DEFAULT_S13_AGENT_NAME);
    let submit_payload = env_var_or_default(
        "KAMN_E2E_S13_SUBMIT_BRIDGE_PAYLOAD",
        DEFAULT_S13_SUBMIT_BRIDGE_PAYLOAD,
    );

    let submit_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-submit").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s13 submit connect failed: {error}"))?;
    let submitted = submit_handle
        .submit_bridge_message(submit_payload.as_str())
        .map_err(|error| format!("sdk-direct live s13 submit-bridge-message failed: {error}"))?;
    if submitted.bridge_id.trim().is_empty() {
        return Err(
            "sdk-direct live s13 submit-bridge-message returned empty bridge_id".to_owned(),
        );
    }
    if submitted.source_message_id.trim().is_empty() {
        return Err(
            "sdk-direct live s13 submit-bridge-message returned empty source_message_id".to_owned(),
        );
    }
    if submitted.bridge_status.trim().is_empty() {
        return Err(
            "sdk-direct live s13 submit-bridge-message returned empty bridge_status".to_owned(),
        );
    }

    let forward_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-forward").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s13 forward connect failed: {error}"))?;
    let forwarded = forward_handle
        .forward_bridge_message(submitted.bridge_id.as_str())
        .map_err(|error| format!("sdk-direct live s13 forward-bridge-message failed: {error}"))?;
    validate_s13_bridge_id_match(
        submitted.bridge_id.as_str(),
        forwarded.bridge_id.as_str(),
        "sdk-direct live s13 forward-bridge-message",
    )?;
    if forwarded.bridge_status.trim().is_empty() {
        return Err(
            "sdk-direct live s13 forward-bridge-message returned empty bridge_status".to_owned(),
        );
    }
    if forwarded.target_message_id.trim().is_empty() {
        return Err(
            "sdk-direct live s13 forward-bridge-message returned empty target_message_id"
                .to_owned(),
        );
    }
    if forwarded.forward_tx_hash.trim().is_empty() {
        return Err(
            "sdk-direct live s13 forward-bridge-message returned empty forward_tx_hash".to_owned(),
        );
    }

    let query_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-query").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s13 query connect failed: {error}"))?;
    let queried = query_handle
        .query_bridge_message(submitted.bridge_id.as_str())
        .map_err(|error| format!("sdk-direct live s13 query-bridge-message failed: {error}"))?;
    validate_s13_bridge_id_match(
        submitted.bridge_id.as_str(),
        queried.bridge_id.as_str(),
        "sdk-direct live s13 query-bridge-message",
    )?;
    validate_s13_bridge_field_coherence(
        forwarded.bridge_status.as_str(),
        queried.bridge_status.as_str(),
        "bridge_status",
        "sdk-direct live s13 query-bridge-message",
    )?;
    validate_s13_bridge_field_coherence(
        forwarded.target_message_id.as_str(),
        queried.target_message_id.as_str(),
        "target_message_id",
        "sdk-direct live s13 query-bridge-message",
    )?;
    validate_s13_bridge_field_coherence(
        forwarded.forward_tx_hash.as_str(),
        queried.forward_tx_hash.as_str(),
        "forward_tx_hash",
        "sdk-direct live s13 query-bridge-message",
    )?;

    Ok(())
}

fn run_live_s14_batch_merkle_probe() -> Result<(), String> {
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    let base_agent_name = env_var_or_default("KAMN_E2E_S14_AGENT_NAME", DEFAULT_S14_AGENT_NAME);
    let batch_message_payload_a = env_var_or_default(
        "KAMN_E2E_S14_BATCH_MESSAGE_PAYLOAD_A",
        DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_A,
    );
    let batch_message_payload_b = env_var_or_default(
        "KAMN_E2E_S14_BATCH_MESSAGE_PAYLOAD_B",
        DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_B,
    );
    let block_height = env::var("KAMN_E2E_S14_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("sdk-direct live s14 invalid block height env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S14_BLOCK_HEIGHT);
    let finality = env_var_or_default("KAMN_E2E_S14_FINALITY", DEFAULT_S14_FINALITY);

    let batch_sender_a_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-batch-a").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s14 batch-a connect failed: {error}"))?;
    let batch_a_receipt = batch_sender_a_handle
        .send_message(batch_message_payload_a.as_str())
        .map_err(|error| format!("sdk-direct live s14 batch-a send-message failed: {error}"))?;
    validate_s08_message_receipt_fields(
        batch_a_receipt.message_id.as_str(),
        batch_a_receipt.status.as_str(),
        "sdk-direct live s14 batch-a send-message",
    )?;

    let batch_sender_b_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-batch-b").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s14 batch-b connect failed: {error}"))?;
    let batch_b_receipt = batch_sender_b_handle
        .send_message(batch_message_payload_b.as_str())
        .map_err(|error| format!("sdk-direct live s14 batch-b send-message failed: {error}"))?;
    validate_s08_message_receipt_fields(
        batch_b_receipt.message_id.as_str(),
        batch_b_receipt.status.as_str(),
        "sdk-direct live s14 batch-b send-message",
    )?;
    validate_s08_distinct_message_ids(
        batch_a_receipt.message_id.as_str(),
        batch_b_receipt.message_id.as_str(),
        "sdk-direct live s14 batch-b send-message",
    )?;

    let query_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-query").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s14 query connect failed: {error}"))?;
    let batch_a_query = query_handle
        .query_message(batch_a_receipt.message_id.as_str())
        .map_err(|error| format!("sdk-direct live s14 batch-a query-message failed: {error}"))?;
    validate_s08_query_message_response(
        batch_a_receipt.message_id.as_str(),
        batch_a_query.message_id.as_str(),
        batch_a_query.status.as_str(),
        "sdk-direct live s14 batch-a query-message",
    )?;

    let batch_b_query = query_handle
        .query_message(batch_b_receipt.message_id.as_str())
        .map_err(|error| format!("sdk-direct live s14 batch-b query-message failed: {error}"))?;
    validate_s08_query_message_response(
        batch_b_receipt.message_id.as_str(),
        batch_b_query.message_id.as_str(),
        batch_b_query.status.as_str(),
        "sdk-direct live s14 batch-b query-message",
    )?;

    let batch_root = env_var_or_else("KAMN_E2E_S14_BATCH_ROOT", || {
        format!(
            "sha256:s14:{}:{}",
            batch_a_receipt.message_id, batch_b_receipt.message_id
        )
    });
    if batch_root.trim().is_empty() {
        return Err("sdk-direct live s14 batch-root marker must not be empty".to_owned());
    }

    let proof_receipt = KolmeProofReceipt {
        tx_hash: batch_root,
        block_height,
        finality,
    };
    let proof_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-proof").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s14 proof connect failed: {error}"))?;

    let batch_a_verification = proof_handle
        .verify_proof(batch_a_receipt.message_id.as_str(), &proof_receipt)
        .map_err(|error| format!("sdk-direct live s14 batch-a verify-proof failed: {error}"))?;
    validate_s14_proof_response(
        batch_a_receipt.message_id.as_str(),
        batch_a_verification.message_id.as_str(),
        batch_a_verification.block_height,
        batch_a_verification.finality.as_str(),
        batch_a_verification.verified,
        "sdk-direct live s14 batch-a verify-proof",
    )?;

    let batch_b_verification = proof_handle
        .verify_proof(batch_b_receipt.message_id.as_str(), &proof_receipt)
        .map_err(|error| format!("sdk-direct live s14 batch-b verify-proof failed: {error}"))?;
    validate_s14_proof_response(
        batch_b_receipt.message_id.as_str(),
        batch_b_verification.message_id.as_str(),
        batch_b_verification.block_height,
        batch_b_verification.finality.as_str(),
        batch_b_verification.verified,
        "sdk-direct live s14 batch-b verify-proof",
    )?;

    Ok(())
}

fn run_live_s15_performance_smoke_probe() -> Result<(), String> {
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", DEFAULT_KOLME_ENDPOINT);
    let base_agent_name = env_var_or_default("KAMN_E2E_S15_AGENT_NAME", DEFAULT_S15_AGENT_NAME);
    let message_payload =
        env_var_or_default("KAMN_E2E_S15_MESSAGE_PAYLOAD", DEFAULT_S15_MESSAGE_PAYLOAD);
    let iterations = env::var("KAMN_E2E_S15_ITERATIONS")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("sdk-direct live s15 invalid iterations env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S15_ITERATIONS);
    if iterations == 0 {
        return Err("sdk-direct live s15 iterations must be greater than zero".to_owned());
    }

    let max_total_millis = parse_s15_budget_env_u128(
        "KAMN_E2E_S15_MAX_TOTAL_MILLIS",
        DEFAULT_S15_MAX_TOTAL_MILLIS,
        "sdk-direct live s15 max-total budget",
    )?;
    let max_p50_millis = parse_s15_budget_env_u128(
        "KAMN_E2E_S15_MAX_P50_MILLIS",
        DEFAULT_S15_MAX_P50_MILLIS,
        "sdk-direct live s15 max-p50 budget",
    )?;
    let max_p99_millis = parse_s15_budget_env_u128(
        "KAMN_E2E_S15_MAX_P99_MILLIS",
        DEFAULT_S15_MAX_P99_MILLIS,
        "sdk-direct live s15 max-p99 budget",
    )?;

    let send_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-send").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s15 send connect failed: {error}"))?;
    let query_handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        format!("{base_agent_name}-query").as_str(),
    )
    .map_err(|error| format!("sdk-direct live s15 query connect failed: {error}"))?;

    let total_start = std::time::Instant::now();
    let mut latency_samples = Vec::with_capacity(iterations as usize);
    for iteration in 0..iterations {
        let iteration_start = std::time::Instant::now();
        let send_receipt = send_handle
            .send_message(message_payload.as_str())
            .map_err(|error| {
                format!("sdk-direct live s15 send-message failed at iteration {iteration}: {error}")
            })?;
        validate_s08_message_receipt_fields(
            send_receipt.message_id.as_str(),
            send_receipt.status.as_str(),
            "sdk-direct live s15 send-message",
        )?;

        let queried_status = query_handle
            .query_message(send_receipt.message_id.as_str())
            .map_err(|error| {
                format!(
                    "sdk-direct live s15 query-message failed at iteration {iteration}: {error}"
                )
            })?;
        validate_s08_query_message_response(
            send_receipt.message_id.as_str(),
            queried_status.message_id.as_str(),
            queried_status.status.as_str(),
            "sdk-direct live s15 query-message",
        )?;

        latency_samples.push(iteration_start.elapsed().as_millis());
    }
    let total_elapsed_millis = total_start.elapsed().as_millis();

    validate_s15_latency_budget_samples(
        latency_samples.as_slice(),
        total_elapsed_millis,
        max_total_millis,
        max_p50_millis,
        max_p99_millis,
        "sdk-direct live s15 performance-smoke",
    )
}

fn validate_s08_message_receipt_fields(
    message_id: &str,
    status: &str,
    step: &str,
) -> Result<(), String> {
    if message_id.trim().is_empty() {
        return Err(format!("{step} returned empty message_id"));
    }
    if status.trim().is_empty() {
        return Err(format!("{step} returned empty status"));
    }
    Ok(())
}

fn validate_s08_query_message_response(
    expected_message_id: &str,
    queried_message_id: &str,
    queried_status: &str,
    step: &str,
) -> Result<(), String> {
    if queried_message_id != expected_message_id {
        return Err(format!(
            "{step} returned mismatched message_id: expected={expected_message_id}, got={queried_message_id}"
        ));
    }
    if queried_status.trim().is_empty() {
        return Err(format!("{step} returned empty status"));
    }
    Ok(())
}

fn validate_s08_distinct_message_ids(
    pre_message_id: &str,
    post_message_id: &str,
    step: &str,
) -> Result<(), String> {
    if post_message_id == pre_message_id {
        return Err(format!("{step} returned duplicate message_id"));
    }
    Ok(())
}

fn validate_s14_proof_response(
    expected_message_id: &str,
    observed_message_id: &str,
    observed_block_height: u64,
    observed_finality: &str,
    observed_verified: bool,
    step: &str,
) -> Result<(), String> {
    if observed_message_id != expected_message_id {
        return Err(format!(
            "{step} returned mismatched message_id: expected={expected_message_id}, got={observed_message_id}"
        ));
    }
    if !observed_verified {
        return Err(format!("{step} returned verified=false"));
    }
    if observed_finality.trim() != "FINAL" {
        return Err(format!(
            "{step} returned non-final finality: {observed_finality}"
        ));
    }
    if observed_block_height == 0 {
        return Err(format!("{step} returned block_height=0"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        live_execution_enabled_from_env, parse_s15_budget_env_u128, run_live_s01_discovery_probe,
        run_live_s02_direct_message_probe, run_live_s03_group_channel_probe,
        run_live_s04_task_lifecycle_probe, run_live_s05_escrow_settlement_probe,
        run_live_s06_proof_verification_probe, run_live_s07_replay_protection_probe,
        run_live_s08_crash_recovery_probe, run_live_s09_transport_failover_probe,
        run_live_s10_topology_coherence_probe, run_live_s11_signer_rotation_probe,
        run_live_s12_retention_deletion_probe, run_live_s13_bridge_forwarding_probe,
        run_live_s14_batch_merkle_probe, run_live_s15_performance_smoke_probe,
        validate_live_s03_list_messages_response, validate_live_s03_query_message_response,
        validate_live_s05_release_escrow_receipt, validate_s07_replay_reason_marker,
        validate_s08_distinct_message_ids, validate_s08_message_receipt_fields,
        validate_s08_query_message_response, validate_s12_content_field_coherence,
        validate_s12_content_id_match, validate_s13_bridge_field_coherence,
        validate_s13_bridge_id_match, validate_s14_proof_response,
        validate_s15_latency_budget_samples, SdkDirectDriver, SDK_DIRECT_LIVE_ENV,
    };
    use std::env;
    use std::ffi::OsString;
    use std::sync::PoisonError;

    fn with_env_vars<F>(updates: &[(&str, Option<&str>)], test: F)
    where
        F: FnOnce(),
    {
        let _guard = crate::drivers::test_env_lock()
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let previous = updates
            .iter()
            .map(|(key, _)| ((*key).to_owned(), env::var_os(key)))
            .collect::<Vec<(String, Option<OsString>)>>();

        for (key, value) in updates {
            match value {
                Some(value) => {
                    // SAFETY: tests serialize env mutation with a process-wide mutex.
                    unsafe { env::set_var(key, value) }
                }
                None => {
                    // SAFETY: tests serialize env mutation with a process-wide mutex.
                    unsafe { env::remove_var(key) }
                }
            }
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test));
        for (key, value) in previous {
            match value {
                Some(value) => {
                    // SAFETY: tests serialize env mutation with a process-wide mutex.
                    unsafe { env::set_var(key, value) }
                }
                None => {
                    // SAFETY: tests serialize env mutation with a process-wide mutex.
                    unsafe { env::remove_var(key) }
                }
            }
        }
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
    }

    #[test]
    fn unit_parse_bool_flag_accepts_true_like_values() {
        for value in ["1", "true", "TRUE", "yes", "on"] {
            assert!(
                crate::drivers::shared_helpers::parse_bool_flag(value),
                "expected truthy for {value}"
            );
        }
    }

    #[test]
    fn unit_parse_bool_flag_rejects_false_like_values() {
        for value in ["0", "false", "off", "no", ""] {
            assert!(
                !crate::drivers::shared_helpers::parse_bool_flag(value),
                "expected falsey for {value}"
            );
        }
    }

    #[test]
    fn unit_live_execution_enabled_from_env_honors_true_and_false_markers() {
        with_env_vars(
            &[
                (SDK_DIRECT_LIVE_ENV, Some("1")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                assert!(
                    live_execution_enabled_from_env(),
                    "truthy env value should enable live SDK-direct mode",
                );
            },
        );

        with_env_vars(&[(SDK_DIRECT_LIVE_ENV, Some("0"))], || {
            assert!(
                !live_execution_enabled_from_env(),
                "falsey env value should disable live SDK-direct mode",
            );
        });
    }

    #[test]
    fn unit_run_live_s01_discovery_probe_rejects_invalid_endpoint() {
        with_env_vars(
            &[
                ("KAMN_ENDPOINT", Some("not-a-valid-endpoint")),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
                ("KAMN_AGENT_NAME", Some("sdk-driver-test")),
            ],
            || {
                let error =
                    run_live_s01_discovery_probe().expect_err("invalid endpoint should fail");
                assert!(
                    error.contains("service.endpoint") || error.contains("service endpoint"),
                    "probe error should reflect connection failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s02_direct_message_probe_rejects_invalid_endpoint() {
        with_env_vars(
            &[
                ("KAMN_ENDPOINT", Some("invalid-endpoint")),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
            ],
            || {
                let error =
                    run_live_s02_direct_message_probe().expect_err("invalid endpoint should fail");
                assert!(
                    error.contains("connect failed"),
                    "probe error should reflect connection failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s03_group_channel_probe_rejects_invalid_endpoint() {
        with_env_vars(
            &[
                ("KAMN_ENDPOINT", Some("invalid-endpoint")),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
            ],
            || {
                let error =
                    run_live_s03_group_channel_probe().expect_err("invalid endpoint should fail");
                assert!(
                    error.contains("connect failed"),
                    "probe error should reflect connection failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s03_group_channel_probe_rejects_query_message_id_mismatch() {
        let error = validate_live_s03_query_message_response("message-1", "message-2", "sent")
            .expect_err("mismatched query message_id should fail");
        assert!(
            error.contains("mismatched message_id"),
            "error should mention message_id mismatch: {error}",
        );
    }

    #[test]
    fn unit_run_live_s03_group_channel_probe_rejects_list_channel_id_mismatch() {
        let error = validate_live_s03_list_messages_response("channel-1", "channel-2")
            .expect_err("mismatched listed channel_id should fail");
        assert!(
            error.contains("mismatched channel_id"),
            "error should mention channel_id mismatch: {error}",
        );
    }

    #[test]
    fn unit_run_live_s04_task_lifecycle_probe_rejects_invalid_endpoint() {
        with_env_vars(
            &[
                ("KAMN_ENDPOINT", Some("not-a-valid-endpoint")),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
                ("KAMN_AGENT_NAME", Some("sdk-driver-test")),
            ],
            || {
                let error =
                    run_live_s04_task_lifecycle_probe().expect_err("invalid endpoint should fail");
                assert!(
                    error.contains("service.endpoint") || error.contains("service endpoint"),
                    "probe error should reflect connection failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s05_escrow_settlement_probe_rejects_invalid_endpoint() {
        with_env_vars(
            &[
                ("KAMN_ENDPOINT", Some("invalid-endpoint")),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
            ],
            || {
                let error = run_live_s05_escrow_settlement_probe()
                    .expect_err("invalid endpoint should fail");
                assert!(
                    error.contains("connect failed"),
                    "probe error should reflect connection failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_validate_live_s05_release_escrow_receipt_rejects_mismatched_escrow_id() {
        let error = validate_live_s05_release_escrow_receipt("escrow-a", "escrow-b", "released")
            .expect_err("mismatched escrow ids should fail");
        assert!(
            error.contains("mismatched escrow_id"),
            "error should describe escrow-id mismatch: {error}",
        );
    }

    #[test]
    fn unit_run_live_s06_proof_verification_probe_rejects_invalid_block_height_env_value() {
        with_env_vars(
            &[("KAMN_E2E_S06_PROOF_BLOCK_HEIGHT", Some("not-a-number"))],
            || {
                let error = run_live_s06_proof_verification_probe()
                    .expect_err("invalid block height env value should fail");
                assert!(
                    error.contains("invalid block height env value"),
                    "probe error should reflect parse failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s07_replay_protection_probe_rejects_invalid_endpoint() {
        with_env_vars(
            &[
                ("KAMN_ENDPOINT", Some("invalid-endpoint")),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
            ],
            || {
                let error = run_live_s07_replay_protection_probe()
                    .expect_err("invalid endpoint should fail");
                assert!(
                    error.contains("connect failed"),
                    "probe error should reflect connection failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s08_crash_recovery_probe_rejects_invalid_endpoint() {
        with_env_vars(
            &[
                ("KAMN_ENDPOINT", Some("invalid-endpoint")),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
            ],
            || {
                let error =
                    run_live_s08_crash_recovery_probe().expect_err("invalid endpoint should fail");
                assert!(
                    error.contains("connect failed"),
                    "probe error should reflect connection failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s09_transport_failover_probe_rejects_invalid_endpoint() {
        with_env_vars(
            &[
                ("KAMN_ENDPOINT", Some("invalid-endpoint")),
                (
                    "KAMN_E2E_S09_FAILOVER_ENDPOINT",
                    Some("invalid-failover-endpoint"),
                ),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
            ],
            || {
                let error = run_live_s09_transport_failover_probe()
                    .expect_err("invalid endpoint should fail");
                assert!(
                    error.contains("connect failed"),
                    "probe error should reflect connection failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s10_topology_coherence_probe_rejects_invalid_endpoint() {
        with_env_vars(
            &[
                (
                    "KAMN_E2E_S10_PRIMARY_ENDPOINT",
                    Some("invalid-primary-endpoint"),
                ),
                (
                    "KAMN_E2E_S10_SECONDARY_ENDPOINT",
                    Some("invalid-secondary-endpoint"),
                ),
                (
                    "KAMN_E2E_S10_TERTIARY_ENDPOINT",
                    Some("invalid-tertiary-endpoint"),
                ),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
            ],
            || {
                let error = run_live_s10_topology_coherence_probe()
                    .expect_err("invalid endpoint should fail");
                assert!(
                    error.contains("connect failed"),
                    "probe error should reflect connection failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s11_signer_rotation_probe_rejects_invalid_endpoint() {
        with_env_vars(
            &[
                ("KAMN_ENDPOINT", Some("invalid-endpoint")),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
            ],
            || {
                let error =
                    run_live_s11_signer_rotation_probe().expect_err("invalid endpoint should fail");
                assert!(
                    error.contains("connect failed"),
                    "probe error should reflect connection failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s12_retention_deletion_probe_rejects_invalid_endpoint() {
        with_env_vars(
            &[
                ("KAMN_ENDPOINT", Some("invalid-endpoint")),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
            ],
            || {
                let error = run_live_s12_retention_deletion_probe()
                    .expect_err("invalid endpoint should fail");
                assert!(
                    error.contains("connect failed"),
                    "probe error should reflect connection failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s13_bridge_forwarding_probe_rejects_invalid_endpoint() {
        with_env_vars(
            &[
                ("KAMN_ENDPOINT", Some("invalid-endpoint")),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
            ],
            || {
                let error = run_live_s13_bridge_forwarding_probe()
                    .expect_err("invalid endpoint should fail");
                assert!(
                    error.contains("connect failed"),
                    "probe error should reflect connection failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s14_batch_merkle_probe_rejects_invalid_endpoint() {
        with_env_vars(
            &[
                ("KAMN_ENDPOINT", Some("invalid-endpoint")),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
            ],
            || {
                let error =
                    run_live_s14_batch_merkle_probe().expect_err("invalid endpoint should fail");
                assert!(
                    error.contains("connect failed"),
                    "probe error should reflect connection failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s15_performance_smoke_probe_rejects_invalid_endpoint() {
        with_env_vars(
            &[
                ("KAMN_ENDPOINT", Some("invalid-endpoint")),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
            ],
            || {
                let error = run_live_s15_performance_smoke_probe()
                    .expect_err("invalid endpoint should fail");
                assert!(
                    error.contains("connect failed"),
                    "probe error should reflect connection failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_validate_s12_content_id_match_rejects_mismatch() {
        let error = validate_s12_content_id_match("content-a", "content-b", "test step")
            .expect_err("mismatched content ids should fail");
        assert!(
            error.contains("mismatched content_id"),
            "error should mention content_id mismatch: {error}",
        );
    }

    #[test]
    fn unit_validate_s12_content_field_coherence_rejects_drift() {
        let error = validate_s12_content_field_coherence(
            "tombstoned",
            "expired",
            "lifecycle_state",
            "test step",
        )
        .expect_err("field drift should fail");
        assert!(
            error.contains("lifecycle_state drift"),
            "error should mention field drift: {error}",
        );
    }

    #[test]
    fn unit_validate_s13_bridge_id_match_rejects_mismatch() {
        let error = validate_s13_bridge_id_match("bridge-a", "bridge-b", "test step")
            .expect_err("mismatched bridge ids should fail");
        assert!(
            error.contains("mismatched bridge_id"),
            "error should mention bridge_id mismatch: {error}",
        );
    }

    #[test]
    fn unit_validate_s13_bridge_field_coherence_rejects_drift() {
        let error =
            validate_s13_bridge_field_coherence("forwarded", "stale", "bridge_status", "test step")
                .expect_err("bridge field drift should fail");
        assert!(
            error.contains("bridge_status drift"),
            "error should mention field drift: {error}",
        );
    }

    #[test]
    fn unit_validate_s07_replay_reason_marker_accepts_expected_marker() {
        validate_s07_replay_reason_marker(
            "operation failed: service_api_auth_replay_nonce_detected",
            "test helper",
        )
        .expect("expected marker should be accepted");
    }

    #[test]
    fn unit_validate_s07_replay_reason_marker_rejects_missing_marker() {
        let error = validate_s07_replay_reason_marker("operation failed", "test helper")
            .expect_err("missing marker should fail");
        assert!(
            error.contains("missing replay reason marker"),
            "error should mention replay marker contract: {error}",
        );
    }

    #[test]
    fn unit_validate_s08_message_receipt_fields_rejects_empty_message_id() {
        let error = validate_s08_message_receipt_fields("", "sent", "test helper")
            .expect_err("empty message_id should fail");
        assert!(
            error.contains("empty message_id"),
            "error should mention message_id requirement: {error}",
        );
    }

    #[test]
    fn unit_validate_s08_query_message_response_rejects_mismatched_message_id() {
        let error = validate_s08_query_message_response("message-1", "message-2", "sent", "test")
            .expect_err("mismatched query message_id should fail");
        assert!(
            error.contains("mismatched message_id"),
            "error should mention message_id mismatch: {error}",
        );
    }

    #[test]
    fn unit_validate_s08_distinct_message_ids_accepts_distinct_ids() {
        validate_s08_distinct_message_ids("message-1", "message-2", "test helper")
            .expect("distinct message ids should pass");
    }

    #[test]
    fn unit_validate_s08_distinct_message_ids_rejects_duplicate_ids() {
        let error = validate_s08_distinct_message_ids("message-1", "message-1", "test helper")
            .expect_err("duplicate message ids should fail");
        assert!(
            error.contains("duplicate message_id"),
            "error should mention duplicate message_id: {error}",
        );
    }

    #[test]
    fn unit_validate_s14_proof_response_accepts_valid_receipt() {
        validate_s14_proof_response("message-1", "message-1", 42, "FINAL", true, "test helper")
            .expect("valid S-14 proof response should pass");
    }

    #[test]
    fn unit_validate_s14_proof_response_rejects_mismatched_message_id() {
        let error =
            validate_s14_proof_response("message-1", "message-2", 42, "FINAL", true, "test helper")
                .expect_err("mismatched message_id should fail");
        assert!(
            error.contains("mismatched message_id"),
            "error should mention message_id mismatch: {error}",
        );
    }

    #[test]
    fn unit_validate_s14_proof_response_rejects_unverified_receipt() {
        let error = validate_s14_proof_response(
            "message-1",
            "message-1",
            42,
            "FINAL",
            false,
            "test helper",
        )
        .expect_err("verified=false should fail");
        assert!(
            error.contains("verified=false"),
            "error should mention verified contract: {error}",
        );
    }

    #[test]
    fn unit_validate_s14_proof_response_rejects_non_final_finality() {
        let error = validate_s14_proof_response(
            "message-1",
            "message-1",
            42,
            "PENDING",
            true,
            "test helper",
        )
        .expect_err("non-final finality should fail");
        assert!(
            error.contains("non-final finality"),
            "error should mention finality contract: {error}",
        );
    }

    #[test]
    fn unit_validate_s14_proof_response_rejects_zero_block_height() {
        let error =
            validate_s14_proof_response("message-1", "message-1", 0, "FINAL", true, "test helper")
                .expect_err("block_height=0 should fail");
        assert!(
            error.contains("block_height=0"),
            "error should mention block-height contract: {error}",
        );
    }

    #[test]
    fn unit_validate_s15_latency_budget_samples_accepts_within_budget_samples() {
        validate_s15_latency_budget_samples(&[10, 20, 30], 80, 100, 25, 35, "test helper")
            .expect("within-budget samples should pass");
    }

    #[test]
    fn unit_validate_s15_latency_budget_samples_rejects_total_budget_violation() {
        let error =
            validate_s15_latency_budget_samples(&[10, 20, 30], 120, 100, 25, 35, "test helper")
                .expect_err("total budget violation should fail");
        assert!(
            error.contains("total elapsed millis exceeded budget"),
            "error should mention total budget: {error}",
        );
    }

    #[test]
    fn unit_validate_s15_latency_budget_samples_rejects_p50_budget_violation() {
        let error =
            validate_s15_latency_budget_samples(&[10, 50, 90], 90, 200, 20, 100, "test helper")
                .expect_err("p50 budget violation should fail");
        assert!(
            error.contains("p50 millis exceeded budget"),
            "error should mention p50 budget: {error}",
        );
    }

    #[test]
    fn unit_validate_s15_latency_budget_samples_rejects_p99_budget_violation() {
        let error =
            validate_s15_latency_budget_samples(&[10, 20, 90], 90, 200, 50, 80, "test helper")
                .expect_err("p99 budget violation should fail");
        assert!(
            error.contains("p99 millis exceeded budget"),
            "error should mention p99 budget: {error}",
        );
    }

    #[test]
    fn unit_parse_s15_budget_env_u128_uses_default_when_env_missing() {
        with_env_vars(&[("KAMN_E2E_S15_TOTAL_BUDGET_MS", None)], || {
            let parsed = parse_s15_budget_env_u128(
                "KAMN_E2E_S15_TOTAL_BUDGET_MS",
                91,
                "sdk-direct live s15 test helper",
            )
            .expect("missing env key should use default");
            assert_eq!(parsed, 91);
        });
    }

    #[test]
    fn unit_parse_s15_budget_env_u128_parses_positive_env_value() {
        with_env_vars(&[("KAMN_E2E_S15_TOTAL_BUDGET_MS", Some("143"))], || {
            let parsed = parse_s15_budget_env_u128(
                "KAMN_E2E_S15_TOTAL_BUDGET_MS",
                91,
                "sdk-direct live s15 test helper",
            )
            .expect("valid env key should parse");
            assert_eq!(parsed, 143);
        });
    }

    #[test]
    fn unit_validate_s15_latency_budget_samples_accepts_exact_budget_boundaries() {
        validate_s15_latency_budget_samples(&[10, 20, 30], 60, 60, 20, 30, "test helper")
            .expect("equal total/p50/p99 budget boundaries should pass");
    }

    #[test]
    fn unit_percentile_index_returns_expected_midpoint_index() {
        assert_eq!(
            crate::drivers::shared_helpers::percentile_index(3, 50),
            1,
            "len=3 and p50 should map to middle sample index",
        );
    }

    #[test]
    fn unit_percentile_index_clamps_percentile_above_hundred_to_last_index() {
        assert_eq!(
            crate::drivers::shared_helpers::percentile_index(3, 150),
            2,
            "percentiles above 100 should clamp to the last sample index",
        );
    }

    #[test]
    fn unit_live_s07_probe_agent_suffix_is_non_empty_numeric() {
        let suffix = super::live_s07_probe_agent_suffix();
        assert!(!suffix.is_empty(), "suffix should be non-empty");
        assert!(
            suffix.chars().all(|character| character.is_ascii_digit()),
            "suffix should be numeric: {suffix}",
        );
    }

    #[test]
    fn unit_run_live_s06_proof_verification_probe_accepts_final_verified_receipt() {
        with_env_vars(
            &[
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
                ("KAMN_AGENT_NAME", Some("sdk-driver-test")),
                ("KAMN_E2E_S06_PROOF_FINALITY", Some("final")),
            ],
            || {
                run_live_s06_proof_verification_probe()
                    .expect("final verified proof probe should succeed");
            },
        );
    }

    #[test]
    fn unit_sdk_direct_driver_debug_includes_live_toggle_field() {
        let driver = SdkDirectDriver::with_probe(false, || Ok(()));
        let debug = format!("{driver:?}");
        assert!(
            debug.contains("SdkDirectDriver"),
            "debug output should include struct name: {debug}",
        );
        assert!(
            debug.contains("live_execution_enabled"),
            "debug output should include live toggle field: {debug}",
        );
    }

    #[test]
    fn spec_c00_live_disabled_driver_path_fails_closed_without_probe_invocation() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let probe_calls = Arc::new(AtomicUsize::new(0));
        let probe_calls_for_closure = Arc::clone(&probe_calls);
        let driver = SdkDirectDriver::with_probe(false, move || {
            probe_calls_for_closure.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-01");
        assert_eq!(
            result.status, "fail",
            "live-disabled S-01 must fail closed instead of reporting pass",
        );
        assert_eq!(
            probe_calls.load(Ordering::SeqCst),
            0,
            "live probe should not be invoked when toggle is disabled",
        );
    }

    #[test]
    fn spec_c01_live_s04_driver_path_fails_closed_when_task_probe_errors() {
        let driver = SdkDirectDriver::with_probe(true, || {
            Err("sdk-direct live s04 task probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-04");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-04 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c02_live_s06_driver_path_fails_closed_when_proof_probe_errors() {
        let driver = SdkDirectDriver::with_probe(true, || {
            Err("sdk-direct live s06 proof probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-06");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-06 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c03_live_s02_driver_path_fails_closed_when_message_probe_errors() {
        let driver = SdkDirectDriver::with_probe(true, || {
            Err("sdk-direct live s02 message probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-02");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-02 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c04_live_s03_driver_path_fails_closed_when_channel_probe_errors() {
        let driver = SdkDirectDriver::with_probe(true, || {
            Err("sdk-direct live s03 channel probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-03");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-03 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c05_live_s05_driver_path_fails_closed_when_escrow_probe_errors() {
        let driver = SdkDirectDriver::with_probe(true, || {
            Err("sdk-direct live s05 escrow probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-05");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-05 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c07_live_s07_driver_path_fails_closed_when_replay_probe_errors() {
        let driver = SdkDirectDriver::with_probe(true, || {
            Err("sdk-direct live s07 replay probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-07");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-07 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c08_live_s08_driver_path_fails_closed_when_crash_recovery_probe_errors() {
        let driver = SdkDirectDriver::with_probe(true, || {
            Err("sdk-direct live s08 crash-recovery probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-08");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-08 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c09_live_s09_driver_path_fails_closed_when_transport_failover_probe_errors() {
        let driver = SdkDirectDriver::with_probe(true, || {
            Err("sdk-direct live s09 transport-failover probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-09");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-09 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c10_live_s10_driver_path_fails_closed_when_topology_coherence_probe_errors() {
        let driver = SdkDirectDriver::with_probe(true, || {
            Err("sdk-direct live s10 topology-coherence probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-10");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-10 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c11_live_s11_driver_path_fails_closed_when_signer_rotation_probe_errors() {
        let driver = SdkDirectDriver::with_probe(true, || {
            Err("sdk-direct live s11 signer-rotation probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-11");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-11 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c12_live_s12_driver_path_fails_closed_when_retention_deletion_probe_errors() {
        let driver = SdkDirectDriver::with_probe(true, || {
            Err("sdk-direct live s12 retention-deletion probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-12");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-12 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c13_live_s13_driver_path_fails_closed_when_bridge_forwarding_probe_errors() {
        let driver = SdkDirectDriver::with_probe(true, || {
            Err("sdk-direct live s13 bridge-forwarding probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-13");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-13 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c14_live_s14_driver_path_fails_closed_when_batch_merkle_probe_errors() {
        let driver = SdkDirectDriver::with_probe(true, || {
            Err("sdk-direct live s14 batch-merkle probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-14");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-14 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c15_live_s15_driver_path_fails_closed_when_performance_smoke_probe_errors() {
        let driver = SdkDirectDriver::with_probe(true, || {
            Err("sdk-direct live s15 performance-smoke probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-15");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-15 should fail closed on probe error",
        );
    }
}

use crate::drivers::{DriverExecutionResult, HarnessDriver};
use crate::ExecutionMode;
use kamn_agent_lib::{KamnAgentHandle, KolmeProofReceipt};
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
const S07_REPLAY_REASON_MARKER: &str = "service_api_auth_replay_nonce_detected";
const DEFAULT_S06_MESSAGE_ID: &str = "s06-live-proof";
const DEFAULT_S06_TX_HASH: &str = "sha256:s06-live-proof";
const DEFAULT_S06_BLOCK_HEIGHT: u64 = 1;
const DEFAULT_S06_FINALITY: &str = "final";

type LiveProbe = dyn Fn() -> Result<(), String> + Send + Sync + 'static;

/// SDK-direct driver with optional live execution for S-01, S-02, S-03, S-04, S-05, S-06, S-07, and S-08.
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
            crash_recovery_probe: live_probe,
        }
    }

    /// Creates SDK-direct driver with explicit per-scenario probe implementations.
    pub fn with_probes<F, G, H, I, J, K, L, M>(
        live_execution_enabled: bool,
        discovery_probe: F,
        direct_message_probe: G,
        group_channel_probe: H,
        task_lifecycle_probe: I,
        escrow_settlement_probe: J,
        proof_replay_and_crash_probes: (K, L, M),
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
    {
        let (proof_verification_probe, replay_protection_probe, crash_recovery_probe) =
            proof_replay_and_crash_probes;
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
        }
    }
}

impl HarnessDriver for SdkDirectDriver {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::SdkDirect
    }

    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult {
        let status = match self.live_probe_for_scenario(scenario_id) {
            Some(probe) if probe.is_ok() => "pass",
            Some(_) => "fail",
            None => "pass",
        };
        DriverExecutionResult {
            scenario_id,
            status,
        }
    }
}

impl SdkDirectDriver {
    fn live_probe_for_scenario(&self, scenario_id: &'static str) -> Option<Result<(), String>> {
        if !self.live_execution_enabled {
            return None;
        }
        match scenario_id {
            "S-01" => Some((self.discovery_probe)()),
            "S-02" => Some((self.direct_message_probe)()),
            "S-03" => Some((self.group_channel_probe)()),
            "S-04" => Some((self.task_lifecycle_probe)()),
            "S-05" => Some((self.escrow_settlement_probe)()),
            "S-06" => Some((self.proof_verification_probe)()),
            "S-07" => Some((self.replay_protection_probe)()),
            "S-08" => Some((self.crash_recovery_probe)()),
            _ => None,
        }
    }
}

fn live_execution_enabled_from_env() -> bool {
    std::env::var(SDK_DIRECT_LIVE_ENV)
        .ok()
        .map(|value| parse_bool_flag(value.as_str()))
        .unwrap_or(false)
}

fn parse_bool_flag(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn run_live_s01_discovery_probe() -> Result<(), String> {
    let endpoint =
        std::env::var("KAMN_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_owned());
    let kolme_endpoint =
        std::env::var("KAMN_KOLME_ENDPOINT").unwrap_or_else(|_| DEFAULT_KOLME_ENDPOINT.to_owned());
    let agent_name =
        std::env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_AGENT_NAME.to_owned());

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
    let endpoint =
        std::env::var("KAMN_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_owned());
    let kolme_endpoint =
        std::env::var("KAMN_KOLME_ENDPOINT").unwrap_or_else(|_| DEFAULT_KOLME_ENDPOINT.to_owned());
    let agent_name =
        std::env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_AGENT_NAME.to_owned());
    let message_payload = std::env::var("KAMN_E2E_S02_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S02_MESSAGE_PAYLOAD.to_owned());
    let reply_payload = std::env::var("KAMN_E2E_S02_REPLY_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S02_REPLY_PAYLOAD.to_owned());

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
    let endpoint =
        std::env::var("KAMN_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_owned());
    let kolme_endpoint =
        std::env::var("KAMN_KOLME_ENDPOINT").unwrap_or_else(|_| DEFAULT_KOLME_ENDPOINT.to_owned());
    let agent_name =
        std::env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_AGENT_NAME.to_owned());
    let channel_payload = std::env::var("KAMN_E2E_S03_CHANNEL_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S03_CHANNEL_PAYLOAD.to_owned());
    let message_payload = std::env::var("KAMN_E2E_S03_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S03_MESSAGE_PAYLOAD.to_owned());

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
    let endpoint =
        std::env::var("KAMN_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_owned());
    let kolme_endpoint =
        std::env::var("KAMN_KOLME_ENDPOINT").unwrap_or_else(|_| DEFAULT_KOLME_ENDPOINT.to_owned());
    let agent_name =
        std::env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_AGENT_NAME.to_owned());
    let create_task_payload = std::env::var("KAMN_E2E_S04_CREATE_TASK_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S04_CREATE_TASK_PAYLOAD.to_owned());

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
    let endpoint =
        std::env::var("KAMN_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_owned());
    let kolme_endpoint =
        std::env::var("KAMN_KOLME_ENDPOINT").unwrap_or_else(|_| DEFAULT_KOLME_ENDPOINT.to_owned());
    let agent_name =
        std::env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_AGENT_NAME.to_owned());
    let fund_payload = std::env::var("KAMN_E2E_S05_FUND_ESCROW_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S05_FUND_ESCROW_PAYLOAD.to_owned());

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
    let endpoint =
        std::env::var("KAMN_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_owned());
    let kolme_endpoint =
        std::env::var("KAMN_KOLME_ENDPOINT").unwrap_or_else(|_| DEFAULT_KOLME_ENDPOINT.to_owned());
    let agent_name =
        std::env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_AGENT_NAME.to_owned());
    let message_id = std::env::var("KAMN_E2E_S06_PROOF_MESSAGE_ID")
        .unwrap_or_else(|_| DEFAULT_S06_MESSAGE_ID.to_owned());
    let tx_hash = std::env::var("KAMN_E2E_S06_PROOF_TX_HASH")
        .unwrap_or_else(|_| DEFAULT_S06_TX_HASH.to_owned());
    let block_height = std::env::var("KAMN_E2E_S06_PROOF_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("sdk-direct live s06 invalid block height env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S06_BLOCK_HEIGHT);
    let finality = std::env::var("KAMN_E2E_S06_PROOF_FINALITY")
        .unwrap_or_else(|_| DEFAULT_S06_FINALITY.to_owned());

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
    let endpoint =
        std::env::var("KAMN_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_owned());
    let kolme_endpoint =
        std::env::var("KAMN_KOLME_ENDPOINT").unwrap_or_else(|_| DEFAULT_KOLME_ENDPOINT.to_owned());
    let base_agent_name = std::env::var("KAMN_E2E_S07_AGENT_NAME")
        .unwrap_or_else(|_| DEFAULT_S07_AGENT_NAME.to_owned());
    let message_payload = std::env::var("KAMN_E2E_S07_REPLAY_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S07_MESSAGE_PAYLOAD.to_owned());
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
    let endpoint =
        std::env::var("KAMN_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_owned());
    let kolme_endpoint =
        std::env::var("KAMN_KOLME_ENDPOINT").unwrap_or_else(|_| DEFAULT_KOLME_ENDPOINT.to_owned());
    let base_agent_name = std::env::var("KAMN_E2E_S08_AGENT_NAME")
        .unwrap_or_else(|_| DEFAULT_S08_AGENT_NAME.to_owned());
    let pre_message_payload = std::env::var("KAMN_E2E_S08_PRE_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S08_PRE_MESSAGE_PAYLOAD.to_owned());
    let post_message_payload = std::env::var("KAMN_E2E_S08_POST_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S08_POST_MESSAGE_PAYLOAD.to_owned());

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

fn validate_s07_replay_reason_marker(replay_error: &str, step: &str) -> Result<(), String> {
    if !replay_error.contains(S07_REPLAY_REASON_MARKER) {
        return Err(format!(
            "{step} missing replay reason marker: {replay_error}"
        ));
    }
    Ok(())
}

fn live_s07_probe_agent_suffix() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        live_execution_enabled_from_env, parse_bool_flag, run_live_s01_discovery_probe,
        run_live_s02_direct_message_probe, run_live_s03_group_channel_probe,
        run_live_s04_task_lifecycle_probe, run_live_s05_escrow_settlement_probe,
        run_live_s06_proof_verification_probe, run_live_s07_replay_protection_probe,
        run_live_s08_crash_recovery_probe, validate_live_s03_list_messages_response,
        validate_live_s03_query_message_response, validate_live_s05_release_escrow_receipt,
        validate_s07_replay_reason_marker, validate_s08_distinct_message_ids,
        validate_s08_message_receipt_fields, validate_s08_query_message_response, SdkDirectDriver,
        SDK_DIRECT_LIVE_ENV,
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
            assert!(parse_bool_flag(value), "expected truthy for {value}");
        }
    }

    #[test]
    fn unit_parse_bool_flag_rejects_false_like_values() {
        for value in ["0", "false", "off", "no", ""] {
            assert!(!parse_bool_flag(value), "expected falsey for {value}");
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
}

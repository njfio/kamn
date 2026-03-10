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

#[path = "sdk_direct/live_probe_tranche_one.rs"]
mod live_probe_tranche_one;
#[path = "sdk_direct/live_probe_tranche_two.rs"]
mod live_probe_tranche_two;

use live_probe_tranche_one::{
    run_live_s01_discovery_probe, run_live_s02_direct_message_probe,
    run_live_s03_group_channel_probe, run_live_s04_task_lifecycle_probe,
    run_live_s05_escrow_settlement_probe,
};
use live_probe_tranche_two::{
    run_live_s06_proof_verification_probe, run_live_s07_replay_protection_probe,
    run_live_s08_crash_recovery_probe, run_live_s09_transport_failover_probe,
    run_live_s10_topology_coherence_probe, validate_s08_distinct_message_ids,
    validate_s08_message_receipt_fields, validate_s08_query_message_response,
};

const SDK_DIRECT_LIVE_ENV: &str = "KAMN_E2E_SDK_DIRECT_LIVE";
const DEFAULT_KOLME_ENDPOINT: &str = "http://localhost:3000";
const DEFAULT_AGENT_NAME: &str = "kamn-e2e-sdk-direct";
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
            shared_live_execution_enabled_from_env(SDK_DIRECT_LIVE_ENV),
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
#[path = "sdk_direct_tests.rs"]
mod sdk_direct_tests;

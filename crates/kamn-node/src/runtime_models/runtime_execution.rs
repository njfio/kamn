use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlanningExecution {
    pub(crate) expected_state_hash: String,
    pub(crate) candidate_count: usize,
    pub(crate) scheduled_candidate_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecoveryExecution {
    pub(crate) expected_state_version: u64,
    pub(crate) expected_state_hash: String,
    pub(crate) attempt_count: usize,
    pub(crate) decisions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DaemonExecution {
    pub(crate) max_ticks: u64,
    pub(crate) tick_interval_ms: u64,
    pub(crate) executed_ticks: u64,
    pub(crate) completion_reason: String,
    pub(crate) service_api_relay_drained_count: u64,
    pub(crate) service_api_relay_projected_state_count: u64,
    pub(crate) observability_latency_p50_ms: u64,
    pub(crate) observability_latency_p99_ms: u64,
    pub(crate) observability_throughput_tps: u64,
    pub(crate) observability_error_rate_bps: u64,
    pub(crate) observability_availability_bps: u64,
    pub(crate) observability_health: String,
    pub(crate) observability_alert_count: usize,
    pub(crate) observability_reason_code: String,
    pub(crate) observability_transport_checkpoint_failures: u64,
    pub(crate) observability_signer_checkpoint_failures: u64,
    pub(crate) observability_commit_checkpoint_failures: u64,
    pub(crate) peer_id: Option<String>,
    pub(crate) peer_lifecycle_final_state: Option<String>,
    pub(crate) peer_lifecycle_applied_events: Option<Vec<String>>,
    pub(crate) phase6_runtime_reason_taxonomy_version: String,
    pub(crate) phase6_runtime_reason_codes_csv: String,
    pub(crate) phase6_runtime_reason_code: String,
    pub(crate) phase6_runtime_total_cycles: u64,
    pub(crate) phase6_runtime_executed_cycles: u64,
    pub(crate) phase6_runtime_deferred_cycles: u64,
    pub(crate) phase6_runtime_fail_closed_cycles: u64,
    pub(crate) convergence_reason_taxonomy_version: String,
    pub(crate) convergence_reason_codes_csv: String,
    pub(crate) convergence_decision: String,
    pub(crate) convergence_reason_code: String,
    pub(crate) convergence_schema_gate_passed: bool,
    pub(crate) convergence_error_path_gate_passed: bool,
    pub(crate) convergence_concurrency_gate_passed: bool,
    pub(crate) convergence_performance_budget_gate_passed: bool,
    pub(crate) convergence_cost_budget_gate_passed: bool,
    pub(crate) live_postgres_multi_host_execution_bundle_schema_version: String,
    pub(crate) live_postgres_multi_host_execution_bundle_selector_prefix: String,
    pub(crate) live_postgres_multi_host_execution_bundle_row_count: usize,
    pub(crate) live_postgres_multi_host_execution_bundle_selector_rows_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DaemonRuntimeOptions {
    pub(crate) daemon_max_ticks: Option<u64>,
    pub(crate) daemon_tick_interval_ms: Option<u64>,
    pub(crate) daemon_shutdown_signal_ticks: Vec<u64>,
    pub(crate) daemon_shutdown_os_signals: bool,
    pub(crate) daemon_shutdown_drain_ticks: Option<u64>,
    pub(crate) daemon_shutdown_timeout_ticks: Option<u64>,
    pub(crate) daemon_peer_id: Option<String>,
    pub(crate) daemon_lifecycle_events: Vec<PeerLifecycleEvent>,
    pub(crate) service_api_state_file: Option<String>,
    pub(crate) service_api_relay_spool_file: Option<String>,
    pub(crate) service_api_signature_state_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KolmeLiveExecution {
    pub(crate) provider_client_contract: String,
    pub(crate) base_url: String,
    pub(crate) provider_hint: String,
    pub(crate) signing_profile: String,
    pub(crate) signer_profile_selector_env: String,
    pub(crate) signer_profile: String,
    pub(crate) signer_key_source: String,
    pub(crate) signer_private_key_env: String,
    pub(crate) execution_status: String,
    pub(crate) observability_latency_p50_ms: u64,
    pub(crate) observability_latency_p99_ms: u64,
    pub(crate) observability_throughput_tps: u64,
    pub(crate) observability_error_rate_bps: u64,
    pub(crate) observability_availability_bps: u64,
    pub(crate) observability_health: String,
    pub(crate) observability_alert_count: usize,
    pub(crate) observability_reason_code: String,
    pub(crate) observability_transport_checkpoint_failures: u64,
    pub(crate) observability_signer_checkpoint_failures: u64,
    pub(crate) observability_commit_checkpoint_failures: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct RuntimeExecutionBundle {
    pub(crate) planning: Option<PlanningExecution>,
    pub(crate) recovery: Option<RecoveryExecution>,
    pub(crate) daemon: Option<DaemonExecution>,
    pub(crate) kolme_live: Option<KolmeLiveExecution>,
}

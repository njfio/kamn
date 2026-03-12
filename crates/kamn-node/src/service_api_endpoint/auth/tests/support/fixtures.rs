use super::super::super::*;
use kamn_core::service_auth_public_key_hex_from_private_key_hex;

pub(crate) const TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

pub(crate) fn test_service_api_snapshot() -> ServiceApiSnapshot {
    build_service_api_snapshot(&NodeBootstrapReport {
        runtime_mode: "test".to_owned(),
        diagnostics_mode: "basic".to_owned(),
        component_count: 1,
        planning_expected_state_hash: None,
        planning_candidate_count: None,
        planning_scheduled_candidate_ids: None,
        recovery_expected_state_version: None,
        recovery_expected_state_hash: None,
        recovery_attempt_count: None,
        recovery_decisions: None,
        daemon_max_ticks: None,
        daemon_tick_interval_ms: None,
        daemon_executed_ticks: None,
        daemon_completion_reason: None,
        daemon_service_api_relay_drained_count: None,
        daemon_service_api_relay_projected_state_count: None,
        daemon_observability_latency_p50_ms: None,
        daemon_observability_latency_p99_ms: None,
        daemon_observability_throughput_tps: None,
        daemon_observability_error_rate_bps: None,
        daemon_observability_availability_bps: None,
        daemon_observability_health: None,
        daemon_observability_alert_count: None,
        daemon_observability_reason_code: None,
        daemon_observability_transport_checkpoint_failures: None,
        daemon_observability_signer_checkpoint_failures: None,
        daemon_observability_commit_checkpoint_failures: None,
        daemon_peer_id: None,
        daemon_peer_lifecycle_final_state: None,
        daemon_peer_lifecycle_applied_events: None,
        daemon_phase6_runtime_reason_taxonomy_version: None,
        daemon_phase6_runtime_reason_codes_csv: None,
        daemon_phase6_runtime_reason_code: None,
        daemon_phase6_runtime_total_cycles: None,
        daemon_phase6_runtime_executed_cycles: None,
        daemon_phase6_runtime_deferred_cycles: None,
        daemon_phase6_runtime_fail_closed_cycles: None,
        daemon_convergence_reason_taxonomy_version: None,
        daemon_convergence_reason_codes_csv: None,
        daemon_convergence_decision: None,
        daemon_convergence_reason_code: None,
        daemon_convergence_schema_gate_passed: None,
        daemon_convergence_error_path_gate_passed: None,
        daemon_convergence_concurrency_gate_passed: None,
        daemon_convergence_performance_budget_gate_passed: None,
        daemon_convergence_cost_budget_gate_passed: None,
        daemon_live_postgres_multi_host_execution_bundle_schema_version: None,
        daemon_live_postgres_multi_host_execution_bundle_selector_prefix: None,
        daemon_live_postgres_multi_host_execution_bundle_row_count: None,
        daemon_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint: None,
        kolme_live_provider_client_contract: None,
        kolme_live_base_url: None,
        kolme_live_provider_hint: None,
        kolme_live_signing_profile: None,
        kolme_live_signer_profile_selector_env: None,
        kolme_live_signer_profile: None,
        kolme_live_signer_key_source: None,
        kolme_live_signer_private_key_env: None,
        kolme_live_execution_status: None,
        kolme_live_observability_latency_p50_ms: None,
        kolme_live_observability_latency_p99_ms: None,
        kolme_live_observability_throughput_tps: None,
        kolme_live_observability_error_rate_bps: None,
        kolme_live_observability_availability_bps: None,
        kolme_live_observability_health: None,
        kolme_live_observability_alert_count: None,
        kolme_live_observability_reason_code: None,
        kolme_live_observability_transport_checkpoint_failures: None,
        kolme_live_observability_signer_checkpoint_failures: None,
        kolme_live_observability_commit_checkpoint_failures: None,
        profile: None,
        role: "processor".to_owned(),
        chain_id: "kamn-devnet".to_owned(),
        chain_version: "v0.1.0".to_owned(),
        storage_dir: "./data".to_owned(),
        gossip_enabled: false,
        sync_mode: "local".to_owned(),
        sync_startup: "cold".to_owned(),
        sync_recovery: "disabled".to_owned(),
        state_version: 1,
        pending_migrations: 0,
        components: vec!["service-api".to_owned()],
    })
}

pub(crate) fn test_service_api_runtime_state() -> ServiceApiRuntimeState {
    let auth_public_key_hex =
        service_auth_public_key_hex_from_private_key_hex(TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX)
            .expect("service-auth public key should derive");
    let sender_anti_spam =
        AntiSpamEngine::new(AntiSpamConfig::default()).expect("anti-spam config should build");
    ServiceApiRuntimeState {
        snapshot: test_service_api_snapshot(),
        replay_guard: Arc::new(Mutex::new(ServiceApiReplayGuard::new(
            8,
            Duration::from_secs(60),
        ))),
        request_budget: Arc::new(ServiceApiRequestBudget::new(1)),
        websocket_events: ServiceApiWebsocketEventFanout::new(),
        runtime_observability: Arc::new(Mutex::new(ServiceApiRuntimeObservability::new(
            Instant::now(),
        ))),
        body_limit_bytes: 1024,
        concurrency_limiter: Arc::new(Semaphore::new(1)),
        ingress_rate_window: Arc::new(Mutex::new(ServiceApiIngressRateWindow::new(120))),
        sender_anti_spam: Arc::new(Mutex::new(sender_anti_spam)),
        auth_public_key_hex: Some(auth_public_key_hex),
        message_store: Arc::new(Mutex::new(
            ServiceApiMessageStore::from_optional_state_file(None)
                .expect("test message store should build"),
        )),
        relay_spool_file: None,
    }
}

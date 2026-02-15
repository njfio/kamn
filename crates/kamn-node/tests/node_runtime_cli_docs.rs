const DOC: &str = include_str!("../../../docs/foundation/node-runtime-cli.md");

#[test]
fn doc_contains_output_mode_scope_and_rules() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("--output text"));
    assert!(DOC.contains("--output json"));
    assert!(DOC.contains("ConfigError::InvalidOutputMode"));
    assert!(DOC.contains("--profile local-listener"));
    assert!(DOC.contains("ConfigError::InvalidNodeProfile"));
    assert!(DOC.contains("--diagnostics snapshot"));
    assert!(DOC.contains("ConfigError::InvalidDiagnosticsMode"));
    assert!(DOC.contains("--runtime-mode planning"));
    assert!(DOC.contains("--runtime-mode recovery-check"));
    assert!(DOC.contains("--runtime-mode daemon"));
    assert!(DOC.contains("--runtime-mode kolme-live"));
    assert!(DOC.contains("--kolme-live-base-url"));
    assert!(DOC.contains("--kolme-live-provider-hint"));
    assert!(DOC.contains("--kolme-live-signing-profile"));
    assert!(DOC.contains("ConfigError::InvalidRuntimeMode"));
    assert!(DOC.contains("ConfigError::InvalidDaemonControlArgument"));
    assert!(DOC.contains("ConfigError::InvalidDaemonLifecycleEvent"));
    assert!(DOC.contains("ConfigError::RuntimeDaemonLifecycle"));
    assert!(DOC.contains("ConfigError::InvalidKolmeLiveProviderHint"));
    assert!(DOC.contains("ConfigError::InvalidKolmeLiveSigningProfile"));
    assert!(DOC.contains("ConfigError::RuntimeKolmeLive"));
}

#[test]
fn doc_contains_deterministic_json_fields() {
    assert!(DOC.contains("JSON output is deterministic and includes:"));
    assert!(DOC.contains("runtime_mode"));
    assert!(DOC.contains("diagnostics_mode"));
    assert!(DOC.contains("profile"));
    assert!(DOC.contains("component_count"));
    assert!(DOC.contains("planning_candidate_count"));
    assert!(DOC.contains("planning_scheduled_candidate_ids"));
    assert!(DOC.contains("recovery_expected_state_version"));
    assert!(DOC.contains("recovery_attempt_count"));
    assert!(DOC.contains("recovery_decisions"));
    assert!(DOC.contains("daemon_max_ticks"));
    assert!(DOC.contains("daemon_executed_ticks"));
    assert!(DOC.contains("daemon_completion_reason"));
    assert!(DOC.contains("daemon_observability_latency_p50_ms"));
    assert!(DOC.contains("daemon_observability_latency_p99_ms"));
    assert!(DOC.contains("daemon_observability_throughput_tps"));
    assert!(DOC.contains("daemon_observability_error_rate_bps"));
    assert!(DOC.contains("daemon_observability_availability_bps"));
    assert!(DOC.contains("daemon_observability_health"));
    assert!(DOC.contains("daemon_observability_alert_count"));
    assert!(DOC.contains("daemon_peer_lifecycle_final_state"));
    assert!(DOC.contains("daemon_peer_lifecycle_applied_events"));
    assert!(DOC.contains("kolme_live_provider_client_contract"));
    assert!(DOC.contains("kolme_live_base_url"));
    assert!(DOC.contains("kolme_live_provider_hint"));
    assert!(DOC.contains("kolme_live_signing_profile"));
    assert!(DOC.contains("kolme_live_signer_profile_selector_env"));
    assert!(DOC.contains("kolme_live_signer_profile"));
    assert!(DOC.contains("kolme_live_signer_key_source"));
    assert!(DOC.contains("kolme_live_signer_private_key_env"));
    assert!(DOC.contains("kolme_live_execution_status"));
    assert!(DOC.contains("kolme_live_observability_latency_p50_ms"));
    assert!(DOC.contains("kolme_live_observability_latency_p99_ms"));
    assert!(DOC.contains("kolme_live_observability_throughput_tps"));
    assert!(DOC.contains("kolme_live_observability_error_rate_bps"));
    assert!(DOC.contains("kolme_live_observability_availability_bps"));
    assert!(DOC.contains("kolme_live_observability_health"));
    assert!(DOC.contains("kolme_live_observability_alert_count"));
    assert!(DOC.contains("sync_mode"));
    assert!(DOC.contains("components"));
}

#[test]
fn doc_contains_local_profile_rules() {
    assert!(DOC.contains("## Local Profile Rules"));
    assert!(DOC.contains("chain_id`: `kamn-localnet`"));
    assert!(DOC.contains("storage_dir`: role-scoped"));
    assert!(DOC.contains("Explicit CLI flags override profile defaults"));
}

#[test]
fn doc_contains_diagnostics_snapshot_rules() {
    assert!(DOC.contains("## Diagnostics Snapshot Rules"));
    assert!(DOC.contains("`basic` (default)"));
    assert!(DOC.contains("`snapshot`"));
    assert!(DOC.contains("component_count"));
}

#[test]
fn doc_contains_runtime_planning_rules() {
    assert!(DOC.contains("## Runtime Planning Rules"));
    assert!(DOC.contains("`planning`"));
    assert!(DOC.contains("--expected-state-hash"));
    assert!(DOC.contains("--proposal <id|sender-did|nonce|state-hash>"));
    assert!(DOC.contains("Duplicate candidate IDs and stale state hashes are rejected"));
}

#[test]
fn doc_contains_runtime_recovery_check_rules() {
    assert!(DOC.contains("## Recovery Check Rules"));
    assert!(DOC.contains("`recovery-check`"));
    assert!(DOC.contains("--expected-state-version"));
    assert!(DOC.contains("--rejoin-attempt <node-id|state-version|state-hash|resume-token>"));
    assert!(DOC.contains("Replay resume tokens and version/hash mismatch scenarios are rejected"));
}

#[test]
fn doc_contains_runtime_mode_command_examples() {
    assert!(DOC.contains("`kamn-node --role processor --runtime-mode planning`"));
    assert!(DOC.contains("`kamn-node --role processor --runtime-mode recovery-check`"));
    assert!(DOC.contains("`kamn-node --role processor --runtime-mode daemon`"));
}

#[test]
fn doc_contains_runtime_daemon_rules() {
    assert!(DOC.contains("## Daemon Runtime Rules"));
    assert!(DOC.contains("--daemon-max-ticks"));
    assert!(DOC.contains("--daemon-tick-interval-ms"));
    assert!(DOC.contains("--daemon-shutdown-signal-tick"));
    assert!(DOC.contains("--daemon-shutdown-drain-ticks"));
    assert!(DOC.contains("--daemon-shutdown-timeout-ticks"));
    assert!(DOC.contains("--daemon-lifecycle-event"));
    assert!(DOC.contains("active construct-lock lease owner"));
    assert!(DOC.contains("execute_processor_daemon_tick"));
    assert!(DOC.contains("typed construct-lock errors"));
    assert!(DOC.contains("tick-budget-exhausted"));
    assert!(DOC.contains("graceful-shutdown:"));
    assert!(DOC.contains("graceful-shutdown-timeout:"));
    assert!(DOC.contains("ignored_signals"));
}

#[test]
fn doc_contains_daemon_shutdown_drain_marker_fields() {
    assert!(DOC.contains("shutdown_drain_status"));
    assert!(DOC.contains("shutdown_signal_tick"));
    assert!(DOC.contains("shutdown_drain_ticks"));
    assert!(DOC.contains("shutdown_timeout_ticks"));
    assert!(DOC.contains("shutdown_ignored_signals"));
}

#[test]
fn doc_contains_runtime_kolme_live_rules() {
    assert!(DOC.contains("## Kolme Live Runtime Rules"));
    assert!(DOC.contains("`kolme-live`"));
    assert!(DOC.contains("--kolme-live-base-url"));
    assert!(DOC.contains("--kolme-live-provider-hint"));
    assert!(DOC.contains("--kolme-live-signing-profile"));
    assert!(DOC.contains("KolmeRuntimeCommitLiveProvider"));
    assert!(DOC.contains("kolme-fork-secp256k1-v1"));
    assert!(DOC.contains("signer-selection evidence markers"));
    assert!(DOC.contains("KAMN_KOLME_LIVE_SIGNER_PROFILE"));
    assert!(DOC.contains("KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND"));
    assert!(DOC.contains("KAMN_KOLME_LIVE_MANAGED_SIGNER_REQUIRED"));
    assert!(DOC.contains("KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX"));
    assert!(DOC.contains("KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY"));
    assert!(DOC.contains("ops-primary"));
    assert!(DOC.contains("ops-secondary"));
    assert!(DOC.contains("env-local"));
    assert!(DOC.contains("managed_signer_backend_required_missing"));
    assert!(DOC.contains("managed_signer_backend_required_invalid"));
    assert!(DOC.contains("managed_signer_public_key_marker_missing"));
    assert!(DOC.contains("managed_signer_public_key_marker_invalid"));
    assert!(DOC.contains("production_signer_key_source_env_local_forbidden"));
    assert!(DOC.contains("fallback_signer_secret_present_violation"));
    assert!(DOC.contains("managed_signer_raw_private_key_forbidden"));
    assert!(DOC.contains("KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING=true"));
    assert!(DOC.contains("signer key-source provenance matrix"));
    assert!(DOC.contains("runtime must not silently fall back to `env-local`"));
    assert!(DOC.contains("/runtime-commit/status"));
    assert!(DOC.contains("max-attempt budget `2`"));
    assert!(DOC.contains("finality-polled"));
    assert!(DOC.contains("finality-unavailable"));
    assert!(DOC.contains("kolme_live_observability_latency_p50_ms"));
    assert!(DOC.contains("kolme_live_observability_health"));
}

#[test]
fn doc_contains_service_api_ingress_limiter_matrix_rules() {
    assert!(DOC.contains("--api-body-limit-bytes"));
    assert!(DOC.contains("--api-concurrency-limit"));
    assert!(DOC.contains("--api-rate-limit-per-second"));
    assert!(DOC.contains("sender window limit: `3` messages over `5` seconds"));
    assert!(DOC.contains("suspension trigger: `2` consecutive sender rate-limit violations"));
    assert!(DOC.contains("suspension duration: `60` seconds"));
    assert!(DOC.contains("service_api_ingress_body_size_limit_exceeded"));
    assert!(DOC.contains("service_api_ingress_concurrency_limit_exceeded"));
    assert!(DOC.contains("service_api_ingress_rate_limit_exceeded"));
    assert!(DOC.contains("service_api_ingress_sender_rate_limit_exceeded"));
    assert!(DOC.contains("service_api_ingress_sender_suspended"));
}

#[test]
fn doc_contains_p2p_swarm_harness_contracts() {
    assert!(DOC.contains("## P2P Swarm Harness Contracts"));
    assert!(DOC.contains("build_p2p_swarm_deterministic_config"));
    assert!(DOC.contains("compose_libp2p_swarm_behavior_stack"));
    assert!(DOC.contains("compose_kademlia_discovery_bootstrap"));
    assert!(DOC.contains("build_runtime_wiring_with_transport_profile"));
    assert!(DOC.contains("RuntimeTransportProfile::Libp2pLive"));
    assert!(DOC.contains("Libp2pLivePeerLifecycleTransport"));
    assert!(DOC.contains("apply_live_transport_signal"));
    assert!(DOC.contains("build_libp2p_lifecycle_regression_corpus"));
    assert!(DOC.contains("run_libp2p_lifecycle_regression_case"));
    assert!(DOC.contains("run_libp2p_lifecycle_regression_corpus"));
    assert!(DOC.contains("P2pSwarmHarnessTask::start"));
    assert!(DOC.contains("p2p-libp2p-swarm-stack"));
    assert!(DOC.contains("p2p-libp2p-harness-ready"));
    assert!(DOC.contains("p2p-transport-profile:in-memory-deterministic"));
    assert!(DOC.contains("p2p-in-memory-transport-fallback"));
    assert!(DOC.contains("p2p-transport-profile:libp2p-live"));
    assert!(DOC.contains("p2p-live-libp2p-provider"));
    assert!(DOC.contains("P2pTransportError::InvalidSwarmListenAddress"));
    assert!(DOC.contains("P2pTransportError::InvalidSwarmBootstrapPeerAddress"));
    assert!(DOC.contains("P2pTransportError::InvalidSwarmHarnessTickBudget"));
    assert!(DOC.contains("P2pTransportError::GossipTransportDisabled"));
    assert!(DOC.contains("P2pTransportError::MissingKademliaBootstrapSeeds"));
    assert!(DOC.contains("discovery backend marker remains deterministic: `kademlia`."));
    assert!(DOC.contains("runtime_peer_transition_invalid"));
}

#[test]
fn doc_contains_decomposition_guardrails() {
    assert!(DOC.contains("## Decomposition Guardrails"));
    assert!(DOC.contains("main.rs` orchestrates only"));
    assert!(DOC.contains("docs/architecture/kamn-node-module-map.md"));
    assert!(DOC.contains("src/cli.rs"));
    assert!(DOC.contains("src/runtime_kolme_live.rs"));
    assert!(DOC.contains("src/signer.rs"));
    assert!(DOC.contains("src/wire_payload.rs"));
    assert!(DOC.contains("Regression: #2606"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-node"));
    assert!(DOC.contains("cargo test -p kamn-core construct_lock"));
    assert!(DOC.contains("cargo clippy -p kamn-node -- -D warnings"));
}

#[test]
fn doc_contains_docs_fast_lane_command_checks() {
    assert!(DOC.contains("cargo test -p kamn-node --test node_runtime_cli_docs"));
    assert!(DOC.contains("cargo test -p kamn-node --test node_module_map_docs"));
    assert!(DOC.contains("cargo test -p kamn-core --test runtime_network_docs"));
}

#[test]
fn doc_contains_daemon_focused_fast_lane_commands() {
    assert!(DOC.contains("### Daemon-focused fast lane"));
    assert!(DOC.contains(
        "cargo test -p kamn-node integration_runtime_daemon_renders_bounded_completion_output"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node regression_runtime_daemon_rejects_invalid_lifecycle_transition"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node functional_runtime_daemon_applies_graceful_shutdown_signal"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node integration_runtime_daemon_shutdown_timeout_is_fail_closed"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node regression_runtime_kolme_live_rejects_provider_marker_drift"
    ));
}

#[test]
fn doc_contains_processor_ha_reference_section() {
    assert!(DOC.contains("## Processor HA Runtime References"));
    assert!(DOC.contains("docs/foundation/runtime-processor-ha.md"));
}

#[test]
fn regression_requires_invalid_output_mode_rule() {
    // Regression: #307
    assert!(DOC.contains("Invalid modes are rejected with explicit typed error."));
}

#[test]
fn regression_requires_invalid_profile_rule() {
    // Regression: #310
    assert!(DOC.contains("Invalid profiles are rejected with explicit typed error."));
}

#[test]
fn regression_requires_invalid_diagnostics_mode_rule() {
    // Regression: #313
    assert!(DOC.contains("Invalid diagnostics modes are rejected with explicit typed error."));
}

#[test]
fn regression_requires_runtime_planning_candidate_rules() {
    // Regression: #335
    assert!(
        DOC.contains("duplicate/stale runtime planning candidate rejection (`Regression: #335`)")
    );
}

#[test]
fn regression_requires_runtime_recovery_rejection_rules() {
    // Regression: #336
    assert!(DOC.contains("replay/version/hash recovery-check rejection (`Regression: #336`)"));
}

#[test]
fn regression_requires_runtime_recovery_error_rule_references() {
    // Regression: #337
    assert!(DOC.contains("ConfigError::InvalidExpectedStateVersion"));
    assert!(DOC.contains("ConfigError::InvalidRejoinAttemptArgument"));
    assert!(DOC.contains("ConfigError::RuntimeRecovery"));
}

#[test]
fn regression_requires_runtime_daemon_control_rules() {
    // Regression: #348
    assert!(DOC.contains("zero/invalid daemon bounded-loop control rejection (`Regression: #348`)"));
}

#[test]
fn regression_requires_runtime_daemon_lifecycle_rules() {
    // Regression: #349
    assert!(DOC.contains("invalid daemon lifecycle transition rejection (`Regression: #349`)"));
}

#[test]
fn regression_requires_runtime_daemon_lease_guard_rules() {
    // Regression: #388
    assert!(
        DOC.contains("daemon lease guard no-lease/invalid-owner rejection (`Regression: #388`)")
    );
}

#[test]
fn regression_requires_runtime_kolme_live_guard_rules() {
    // Regression: #2175
    assert!(DOC.contains(
        "in-memory fallback and invalid signing profile rejection (`Regression: #2175`)"
    ));
}

#[test]
fn regression_requires_runtime_kolme_live_provider_drift_guard_rules() {
    // Regression: #2176
    assert!(DOC.contains(
        "provider marker drift rejection in live submit/finality flow (`Regression: #2176`)"
    ));
}

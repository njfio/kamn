use super::*;

#[test]
fn functional_json_render_is_deterministic() {
    let report = NodeBootstrapReport {
        runtime_mode: "bootstrap".to_owned(),
        diagnostics_mode: "basic".to_owned(),
        component_count: 2,
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
        gossip_enabled: true,
        sync_mode: "fast".to_owned(),
        sync_startup: "StateSyncToLatest".to_owned(),
        sync_recovery: "ResumeRecentState".to_owned(),
        state_version: 1,
        pending_migrations: 0,
        components: vec!["processor".to_owned(), "listener".to_owned()],
    };

    let first = render_bootstrap_report(&report, OutputMode::json());
    let second = render_bootstrap_report(&report, OutputMode::json());
    assert_eq!(first, second, "json output should be deterministic");
    assert!(first.contains("\"role\":\"processor\""));
    assert!(first.contains("\"components\":[\"processor\",\"listener\"]"));
}

#[test]
fn integration_parse_bootstrap_and_render_json() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args(args).expect("args should parse");
    let config = NodeConfig {
        chain_id: parsed.chain_id,
        chain_version: parsed.chain_version,
        role: parsed.role,
        storage_dir: parsed.storage_dir,
        enable_gossip: parsed.enable_gossip,
        sync_mode: parsed.sync_mode,
    };
    let plan = bootstrap(config).expect("bootstrap should succeed");
    let report = build_bootstrap_report(
        &plan,
        parsed.profile,
        parsed.diagnostics_mode,
        RuntimeMode::bootstrap(),
        RuntimeExecutionBundle::default(),
    );
    let rendered = render_bootstrap_report(&report, parsed.output_mode);

    assert!(rendered.contains("\"diagnostics_mode\":\"basic\""));
    assert!(rendered.contains("\"profile\":null"));
    assert!(rendered.contains("\"role\":\"processor\""));
    assert!(rendered.contains("\"chain_id\":\"kamn-devnet\""));
    assert!(rendered.contains("\"sync_mode\":\"fast\""));
    assert!(rendered.contains("\"components\":["));
}

#[test]
fn integration_profile_bootstrap_and_render_json() {
    let args = vec![
        "kamn-node".to_owned(),
        "--profile".to_owned(),
        "local-listener".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args(args).expect("profile args should parse");
    let config = NodeConfig {
        chain_id: parsed.chain_id,
        chain_version: parsed.chain_version,
        role: parsed.role,
        storage_dir: parsed.storage_dir,
        enable_gossip: parsed.enable_gossip,
        sync_mode: parsed.sync_mode,
    };
    let plan = bootstrap(config).expect("bootstrap should succeed");
    let report = build_bootstrap_report(
        &plan,
        parsed.profile,
        parsed.diagnostics_mode,
        RuntimeMode::bootstrap(),
        RuntimeExecutionBundle::default(),
    );
    let rendered = render_bootstrap_report(&report, parsed.output_mode);

    assert!(rendered.contains("\"diagnostics_mode\":\"basic\""));
    assert!(rendered.contains("\"profile\":\"local-listener\""));
    assert!(rendered.contains("\"role\":\"listener\""));
    assert!(rendered.contains("\"chain_id\":\"kamn-localnet\""));
    assert!(rendered.contains("\"storage_dir\":\"./data/listener\""));
}

#[test]
fn integration_diagnostics_snapshot_includes_component_count() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
        "--diagnostics".to_owned(),
        "snapshot".to_owned(),
    ];
    let parsed = parse_args(args).expect("diagnostics args should parse");
    let config = NodeConfig {
        chain_id: parsed.chain_id,
        chain_version: parsed.chain_version,
        role: parsed.role,
        storage_dir: parsed.storage_dir,
        enable_gossip: parsed.enable_gossip,
        sync_mode: parsed.sync_mode,
    };
    let plan = bootstrap(config).expect("bootstrap should succeed");
    let report = build_bootstrap_report(
        &plan,
        parsed.profile,
        parsed.diagnostics_mode,
        RuntimeMode::bootstrap(),
        RuntimeExecutionBundle::default(),
    );
    let rendered = render_bootstrap_report(&report, parsed.output_mode);

    assert!(rendered.contains("\"diagnostics_mode\":\"snapshot\""));
    assert!(rendered.contains("\"component_count\":"));
}

#[test]
fn integration_runtime_planning_renders_sorted_candidate_ids() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "planning".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-1".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
        "--proposal".to_owned(),
        "tx-2|did:kamn:agent:bbb|2|state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-1|did:kamn:agent:aaa|1|state-1".to_owned(),
    ];

    let parsed = parse_args(args).expect("planning args should parse");
    let report = execute(parsed).expect("planning execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"runtime_mode\":\"planning\""));
    assert!(rendered.contains("\"planning_candidate_count\":2"));
    assert!(rendered.contains("\"planning_scheduled_candidate_ids\":[\"tx-1\",\"tx-2\"]"));
}

#[test]
fn integration_runtime_recovery_check_renders_deterministic_decision_output() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "recovery-check".to_owned(),
        "--expected-state-version".to_owned(),
        "42".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-42".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
        "--rejoin-attempt".to_owned(),
        "node-a|40|state-40|resume-1".to_owned(),
    ];

    let parsed = parse_args(args).expect("recovery-check args should parse");
    let report = execute(parsed).expect("recovery-check execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"runtime_mode\":\"recovery-check\""));
    assert!(rendered.contains("\"recovery_expected_state_version\":42"));
    assert!(rendered.contains("\"recovery_expected_state_hash\":\"state-42\""));
    assert!(rendered.contains("\"recovery_attempt_count\":1"));
    assert!(rendered.contains("\"recovery_decisions\":[\"catch-up-required:40->42\"]"));
}

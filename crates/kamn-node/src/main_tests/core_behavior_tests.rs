use super::*;

#[test]
fn parses_required_role_and_defaults() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
    ];

    let parsed = parse_args(args).expect("args should parse");
    assert_eq!(parsed.profile, None);
    assert_eq!(parsed.role, NodeRole::Processor);
    assert_eq!(parsed.chain_id, "kamn-devnet");
    assert_eq!(parsed.chain_version, "v0.1.0");
    assert_eq!(parsed.storage_dir, "./data");
    assert!(parsed.enable_gossip);
    assert_eq!(parsed.sync_mode, SyncMode::Fast);
    assert_eq!(parsed.runtime_mode, RuntimeMode::bootstrap());
    assert_eq!(parsed.expected_state_hash, None);
    assert_eq!(parsed.expected_state_version, None);
    assert!(parsed.proposals.is_empty());
    assert!(parsed.rejoin_attempts.is_empty());
    assert_eq!(parsed.daemon_max_ticks, None);
    assert_eq!(parsed.daemon_tick_interval_ms, None);
    assert!(parsed.daemon_shutdown_signal_ticks.is_empty());
    assert_eq!(parsed.daemon_shutdown_drain_ticks, None);
    assert_eq!(parsed.daemon_shutdown_timeout_ticks, None);
    assert_eq!(parsed.daemon_peer_id, None);
    assert!(parsed.daemon_lifecycle_events.is_empty());
    assert_eq!(parsed.kolme_live_base_url, None);
    assert_eq!(parsed.kolme_live_provider_hint, None);
    assert_eq!(parsed.kolme_live_signing_profile, None);
    assert_eq!(parsed.output_mode, OutputMode::text());
    assert_eq!(parsed.diagnostics_mode, DiagnosticsMode::basic());
}

#[test]
fn parses_disable_gossip_flag() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "listener".to_owned(),
        "--disable-gossip".to_owned(),
    ];

    let parsed = parse_args(args).expect("args should parse");
    assert_eq!(parsed.role, NodeRole::Listener);
    assert!(!parsed.enable_gossip);
    assert_eq!(parsed.sync_mode, SyncMode::Fast);
}

#[test]
fn parses_sync_mode_flag() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--sync-mode".to_owned(),
        "archive".to_owned(),
    ];

    let parsed = parse_args(args).expect("args should parse");
    assert_eq!(parsed.sync_mode, SyncMode::Archive);
}

#[test]
fn parses_output_mode_json_flag() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args(args).expect("args should parse");
    assert_eq!(parsed.output_mode, OutputMode::json());
}

#[test]
fn parses_diagnostics_snapshot_flag() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--diagnostics".to_owned(),
        "snapshot".to_owned(),
    ];

    let parsed = parse_args(args).expect("diagnostics args should parse");
    assert_eq!(parsed.diagnostics_mode, DiagnosticsMode::snapshot());
}

#[test]
fn unit_log_config_parses_level_and_format_inputs() {
    let config = resolve_log_config_from_inputs(Some("debug"), Some("json"))
        .expect("log config inputs should parse");
    assert_eq!(
        config,
        NodeLogConfig {
            level: NodeLogLevel::Debug,
            format: NodeLogFormat::Json,
        }
    );
}

#[test]
fn unit_log_renderer_renders_json_event_fields() {
    let line = render_log_event_line(
        NodeLogConfig {
            level: NodeLogLevel::Info,
            format: NodeLogFormat::Json,
        },
        NodeLogLevel::Info,
        "kolme.live.submit.start",
        &[
            ("correlation_id", "runtime-commit:abc"),
            ("provider_hint", "local"),
        ],
    );
    assert!(line.contains("\"level\":\"INFO\""));
    assert!(line.contains("\"event\":\"kolme.live.submit.start\""));
    assert!(line.contains("\"correlation_id\":\"runtime-commit:abc\""));
    assert!(line.contains("\"provider_hint\":\"local\""));
}

#[test]
fn integration_bootstrap_runtime_emits_structured_marker() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
    ])
    .expect("args should parse");
    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("bootstrap execution should succeed");
    assert_eq!(report.runtime_mode, "bootstrap");
    assert!(
        captured_logs
            .iter()
            .any(|line| line.contains("\"event\":\"node.runtime.bootstrap.plan.ready\"")),
        "bootstrap runtime should emit structured bootstrap marker"
    );
}

#[test]
fn functional_kolme_live_submit_and_finality_logs_keep_correlation_id() {
    let _signer_lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"pending"}"#,
        ),
        MockHttpReply::ok(
            r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
        ),
    ]);
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ])
    .expect("kolme-live args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("kolme-live execution should succeed");
    assert_eq!(report.runtime_mode, "kolme-live");

    let required_events = [
        "kolme.live.submit.start",
        "kolme.live.submit.outcome",
        "kolme.live.finality.poll.start",
        "kolme.live.finality.poll.outcome",
        "kolme.live.execution.complete",
    ];

    let mut correlation_id = None;
    for event_name in required_events {
        let matching_line = captured_logs
            .iter()
            .find(|line| line.contains(format!("\"event\":\"{event_name}\"").as_str()))
            .expect("required structured event should be present");
        let observed = extract_json_string_field(matching_line, "correlation_id")
            .expect("structured event should include correlation id");
        if let Some(expected) = correlation_id.as_deref() {
            assert_eq!(observed, expected);
        } else {
            assert!(!observed.is_empty(), "correlation id must not be empty");
            correlation_id = Some(observed);
        }
    }
}

#[test]
fn functional_runtime_daemon_emits_structured_transition_markers() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ])
    .expect("daemon args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("daemon execution should succeed");
    assert_eq!(report.runtime_mode, "daemon");

    let start_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.daemon.execute.start\""))
        .expect("daemon execution should emit structured start marker");
    assert_eq!(
        extract_json_string_field(start_line, "runtime_mode").as_deref(),
        Some("daemon")
    );
    assert_eq!(
        extract_json_string_field(start_line, "max_ticks").as_deref(),
        Some("3")
    );
    assert_eq!(
        extract_json_string_field(start_line, "tick_interval_ms").as_deref(),
        Some("25")
    );

    let complete_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.daemon.execute.complete\""))
        .expect("daemon execution should emit structured completion marker");
    assert_eq!(
        extract_json_string_field(complete_line, "runtime_mode").as_deref(),
        Some("daemon")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "executed_ticks").as_deref(),
        Some("3")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "completion_reason").as_deref(),
        Some("tick-budget-exhausted")
    );
}

#[test]
fn functional_kolme_live_retry_emits_structured_retry_markers() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
        MockHttpReply {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{\"error\":\"submit unavailable\"}".to_owned(),
        },
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"pending"}"#,
        ),
        MockHttpReply {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{\"error\":\"finality unavailable\"}".to_owned(),
        },
        MockHttpReply::ok(
            r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
        ),
    ]);
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ])
    .expect("kolme-live args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("runtime should recover from transient provider failures");
    assert_eq!(report.runtime_mode, "kolme-live");

    let submit_retry_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"kolme.live.submit.retry\""))
        .expect("kolme-live retry flow should emit submit retry marker");
    let finality_retry_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"kolme.live.finality.retry\""))
        .expect("kolme-live retry flow should emit finality retry marker");

    let submit_correlation = extract_json_string_field(submit_retry_line, "correlation_id")
        .expect("submit retry marker should include correlation id");
    let finality_correlation = extract_json_string_field(finality_retry_line, "correlation_id")
        .expect("finality retry marker should include correlation id");
    assert_eq!(submit_correlation, finality_correlation);
    assert_eq!(
        extract_json_string_field(submit_retry_line, "reason").as_deref(),
        Some("unavailable")
    );
    assert_eq!(
        extract_json_string_field(finality_retry_line, "reason").as_deref(),
        Some("unavailable")
    );
}

#[test]
fn regression_invalid_log_level_config_fails_closed() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("invalid-level"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
    ])
    .expect("args should parse");
    let error = execute(parsed).expect_err("invalid log level should fail closed");
    assert!(
        matches!(error, ConfigError::InvalidLogConfig(message) if message.contains("KAMN_NODE_LOG_LEVEL")),
        "invalid log level should produce InvalidLogConfig"
    );
}

#[test]
fn performance_structured_logging_rendering_stays_bounded() {
    let started = Instant::now();
    for _ in 0..5_000 {
        let line = render_log_event_line(
            NodeLogConfig {
                level: NodeLogLevel::Info,
                format: NodeLogFormat::Json,
            },
            NodeLogLevel::Info,
            "kolme.live.submit.outcome",
            &[
                ("correlation_id", "runtime-commit:benchmark"),
                ("commit_id", "kolme-commit:benchmark"),
                ("finality", "pending"),
            ],
        );
        assert!(
            line.contains("\"event\":\"kolme.live.submit.outcome\""),
            "rendered line should contain expected event marker"
        );
    }
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "structured log rendering baseline exceeded 1s bound for 5k iterations"
    );
}

#[test]
fn parses_runtime_mode_planning_with_proposals() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "planning".to_owned(),
        "--expected-state-hash".to_owned(),
        "state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-2|did:kamn:agent:bbb|2|state-1".to_owned(),
        "--proposal".to_owned(),
        "tx-1|did:kamn:agent:aaa|1|state-1".to_owned(),
    ];

    let parsed = parse_args(args).expect("planning args should parse");
    assert_eq!(parsed.runtime_mode, RuntimeMode::planning());
    assert_eq!(parsed.expected_state_hash, Some("state-1".to_owned()));
    assert_eq!(parsed.proposals.len(), 2);
}

#[test]
fn parses_runtime_mode_recovery_check_with_rejoin_attempt() {
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
        "--rejoin-attempt".to_owned(),
        "node-a|42|state-42|resume-1".to_owned(),
    ];

    let parsed = parse_args(args).expect("recovery-check args should parse");
    assert_eq!(parsed.runtime_mode, RuntimeMode::recovery_check());
    assert_eq!(parsed.expected_state_version, Some(42));
    assert_eq!(parsed.expected_state_hash, Some("state-42".to_owned()));
    assert_eq!(parsed.rejoin_attempts.len(), 1);
}

#[test]
fn parses_runtime_mode_daemon_with_bounded_controls() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-peer-id".to_owned(),
        "peer-alpha".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "start-connect".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "handshake-succeeded".to_owned(),
    ];

    let parsed = parse_args(args).expect("daemon args should parse");
    assert_eq!(parsed.runtime_mode, RuntimeMode::daemon());
    assert_eq!(parsed.daemon_max_ticks, Some(3));
    assert_eq!(parsed.daemon_tick_interval_ms, Some(25));
    assert_eq!(parsed.daemon_peer_id, Some("peer-alpha".to_owned()));
    assert_eq!(parsed.daemon_lifecycle_events.len(), 2);
}

#[test]
fn parses_runtime_mode_daemon_with_shutdown_controls() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "8".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "3".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
    ];

    let parsed = parse_args(args).expect("daemon args with shutdown controls should parse");
    assert_eq!(parsed.daemon_shutdown_signal_ticks, vec![3]);
    assert_eq!(parsed.daemon_shutdown_drain_ticks, Some(2));
    assert_eq!(parsed.daemon_shutdown_timeout_ticks, Some(4));
}

#[test]
fn parses_runtime_mode_kolme_live_with_required_flags() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ];

    let parsed = parse_args(args).expect("kolme-live args should parse");
    assert_eq!(parsed.runtime_mode.as_str(), "kolme-live");
    assert_eq!(
        parsed.kolme_live_base_url,
        Some("http://127.0.0.1:3000".to_owned())
    );
    assert_eq!(
        parsed.kolme_live_provider_hint,
        Some("kolme-fork-local".to_owned())
    );
    assert_eq!(
        parsed.kolme_live_signing_profile,
        Some("kolme-fork-secp256k1-v1".to_owned())
    );
    assert!(!parsed.kolme_live_strict_signer_contracts);
    assert_eq!(parsed.kolme_live_signer_profile, None);
    assert_eq!(
        parsed.kolme_live_signer_key_source,
        Some("env-local".to_owned())
    );
}

#[test]
fn parses_local_listener_profile_defaults() {
    let args = vec![
        "kamn-node".to_owned(),
        "--profile".to_owned(),
        "local-listener".to_owned(),
    ];

    let parsed = parse_args(args).expect("profile args should parse");
    assert_eq!(parsed.profile, Some(LocalProfile::Listener));
    assert_eq!(parsed.role, NodeRole::Listener);
    assert_eq!(parsed.chain_id, "kamn-localnet");
    assert_eq!(parsed.storage_dir, "./data/listener");
    assert_eq!(parsed.sync_mode, SyncMode::Fast);
    assert!(parsed.enable_gossip);
}

#[test]
fn profile_defaults_can_be_overridden_by_explicit_flags() {
    let args = vec![
        "kamn-node".to_owned(),
        "--profile".to_owned(),
        "local-listener".to_owned(),
        "--chain-id".to_owned(),
        "kamn-custom".to_owned(),
        "--storage-dir".to_owned(),
        "./tmp/custom-listener".to_owned(),
        "--disable-gossip".to_owned(),
        "--sync-mode".to_owned(),
        "archive".to_owned(),
    ];

    let parsed = parse_args(args).expect("profile args with overrides should parse");
    assert_eq!(parsed.profile, Some(LocalProfile::Listener));
    assert_eq!(parsed.role, NodeRole::Listener);
    assert_eq!(parsed.chain_id, "kamn-custom");
    assert_eq!(parsed.storage_dir, "./tmp/custom-listener");
    assert_eq!(parsed.sync_mode, SyncMode::Archive);
    assert!(!parsed.enable_gossip);
}

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
        daemon_peer_id: None,
        daemon_peer_lifecycle_final_state: None,
        daemon_peer_lifecycle_applied_events: None,
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

#[test]
fn integration_runtime_daemon_renders_bounded_completion_output() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-peer-id".to_owned(),
        "peer-alpha".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "start-connect".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "handshake-succeeded".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "heartbeat-missed".to_owned(),
        "--daemon-lifecycle-event".to_owned(),
        "heartbeat-restored".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args(args).expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"runtime_mode\":\"daemon\""));
    assert!(rendered.contains("\"daemon_max_ticks\":3"));
    assert!(rendered.contains("\"daemon_tick_interval_ms\":25"));
    assert!(rendered.contains("\"daemon_executed_ticks\":3"));
    assert!(rendered.contains("\"daemon_completion_reason\":\"tick-budget-exhausted\""));
    assert!(rendered.contains("\"daemon_observability_latency_p50_ms\":25"));
    assert!(rendered.contains("\"daemon_observability_latency_p99_ms\":50"));
    assert!(rendered.contains("\"daemon_observability_throughput_tps\":2000"));
    assert!(rendered.contains("\"daemon_observability_error_rate_bps\":50"));
    assert!(rendered.contains("\"daemon_observability_availability_bps\":9990"));
    assert!(rendered.contains("\"daemon_observability_health\":\"healthy\""));
    assert!(rendered.contains("\"daemon_observability_alert_count\":0"));
    assert!(rendered.contains("\"daemon_peer_id\":\"peer-alpha\""));
    assert!(rendered.contains("\"daemon_peer_lifecycle_final_state\":\"active\""));
    assert!(
        rendered.contains(
            "\"daemon_peer_lifecycle_applied_events\":[\"start-connect\",\"handshake-succeeded\",\"heartbeat-missed\",\"heartbeat-restored\"]"
        )
    );
}

#[test]
fn functional_runtime_daemon_applies_graceful_shutdown_signal() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "10".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "3".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args(args).expect("daemon shutdown args should parse");
    let report = execute(parsed).expect("daemon graceful shutdown execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"daemon_executed_ticks\":5"));
    assert!(rendered.contains(
        "\"daemon_completion_reason\":\"graceful-shutdown:signal@3;drain_ticks=2;timeout_ticks=4;ignored_signals=0\""
    ));
    assert!(rendered.contains("\"daemon_observability_health\":\"healthy\""));
    assert!(rendered.contains("\"daemon_observability_alert_count\":0"));
}

#[test]
fn integration_runtime_daemon_shutdown_timeout_is_fail_closed() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "10".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "7".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "4".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "2".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args(args).expect("daemon timeout args should parse");
    let report = execute(parsed).expect("daemon timeout execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"daemon_executed_ticks\":9"));
    assert!(rendered.contains(
        "\"daemon_completion_reason\":\"graceful-shutdown-timeout:signal@7;drain_ticks=4;timeout_ticks=2;ignored_signals=0\""
    ));
    assert!(rendered.contains("\"daemon_observability_latency_p50_ms\":145"));
    assert!(rendered.contains("\"daemon_observability_latency_p99_ms\":425"));
    assert!(rendered.contains("\"daemon_observability_throughput_tps\":900"));
    assert!(rendered.contains("\"daemon_observability_error_rate_bps\":250"));
    assert!(rendered.contains("\"daemon_observability_availability_bps\":9800"));
    assert!(rendered.contains("\"daemon_observability_health\":\"critical\""));
    assert!(rendered.contains("\"daemon_observability_alert_count\":4"));
}

#[test]
fn integration_runtime_kolme_live_renders_provider_contract_markers() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"pending"}"#,
        ),
        MockHttpReply::ok(
            r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
        ),
    ]);
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args(args).expect("kolme-live args should parse");
    let report = execute(parsed).expect("kolme-live execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"runtime_mode\":\"kolme-live\""));
    assert!(rendered
        .contains("\"kolme_live_provider_client_contract\":\"KolmeRuntimeCommitLiveProvider\""));
    assert!(rendered.contains("\"kolme_live_signing_profile\":\"kolme-fork-secp256k1-v1\""));
    assert!(rendered
        .contains("\"kolme_live_signer_profile_selector_env\":\"KAMN_KOLME_LIVE_SIGNER_PROFILE\""));
    assert!(rendered.contains("\"kolme_live_signer_profile\":\"ops-primary\""));
    assert!(rendered.contains("\"kolme_live_signer_key_source\":\"env-local\""));
    assert!(rendered.contains(
        "\"kolme_live_signer_private_key_env\":\"KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX\""
    ));
    assert!(rendered.contains("\"kolme_live_execution_status\":\"submitted;"));
    assert!(rendered.contains("\"kolme_live_observability_latency_p50_ms\":40"));
    assert!(rendered.contains("\"kolme_live_observability_latency_p99_ms\":120"));
    assert!(rendered.contains("\"kolme_live_observability_throughput_tps\":2200"));
    assert!(rendered.contains("\"kolme_live_observability_error_rate_bps\":40"));
    assert!(rendered.contains("\"kolme_live_observability_availability_bps\":9995"));
    assert!(rendered.contains("\"kolme_live_observability_health\":\"healthy\""));
    assert!(rendered.contains("\"kolme_live_observability_alert_count\":0"));
    assert!(rendered.contains("submit_attempts=1"));
    assert!(rendered.contains("submit_retry_reason=none"));
    assert!(rendered.contains("finality_retry_attempts=1"));
    assert!(rendered.contains("finality_retry_reason=none"));

    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        3,
        "live runtime should issue nonce, submit, and finality requests"
    );
    assert!(recorded_requests[0].contains("GET /get-next-nonce?pubkey="));
    assert!(recorded_requests[1].contains("PUT /broadcast HTTP/1.1"));
    assert!(recorded_requests[1].contains("X-Idempotency-Key: "));
    let signature =
        extract_json_string_field(request_body(recorded_requests[1].as_str()), "signature")
            .expect("submit request should contain signature JSON field");
    // Regression: #2197
    assert!(
        signature.len() == 128
            && signature
                .as_bytes()
                .iter()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
        "live runtime submit must not fall back to synthetic idempotency-key signatures"
    );
    assert!(recorded_requests[2]
        .contains("GET /runtime-commit/status?commit_id=kolme-commit%3Aab12cd34 HTTP/1.1"));
}

#[test]
fn functional_runtime_kolme_live_retries_transient_submit_and_finality_unavailable_errors() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
        MockHttpReply {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{\"error\":\"submit unavailable\"}".to_owned(),
        },
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"pending"}"#,
        ),
        MockHttpReply {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{\"error\":\"finality unavailable\"}".to_owned(),
        },
        MockHttpReply::ok(
            r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
        ),
    ]);
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args(args).expect("kolme-live args should parse");
    let report = execute(parsed).expect("runtime should recover from transient provider failures");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("submit_attempts=2"));
    assert!(rendered.contains("submit_retry_reason=unavailable"));
    assert!(rendered.contains("finality_retry_attempts=2"));
    assert!(rendered.contains("finality_retry_reason=unavailable"));
    assert!(rendered.contains("resolution=finality-polled"));

    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        5,
        "retry path should issue one extra submit and one extra finality request"
    );
}

#[test]
fn regression_runtime_kolme_live_submit_malformed_response_fails_fast_without_retry() {
    // Regression: #2673
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
        MockHttpReply::ok(r#"{"provider":"kolme-fork-local"}"#),
    ]);
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ])
    .expect("kolme-live args should parse");
    let error = execute(parsed).expect_err("malformed submit response must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("malformed")),
        "malformed submit responses should stay fail-fast and non-retriable"
    );

    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        2,
        "malformed submit response should fail without retrying submit requests"
    );
}

#[test]
fn performance_runtime_kolme_live_retry_recovery_stays_within_budget() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
        MockHttpReply {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{\"error\":\"submit unavailable\"}".to_owned(),
        },
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"pending"}"#,
        ),
        MockHttpReply {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{\"error\":\"finality unavailable\"}".to_owned(),
        },
        MockHttpReply::ok(
            r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
        ),
    ]);
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ])
    .expect("kolme-live args should parse");
    let started = Instant::now();
    let report = execute(parsed).expect("retry flow should recover within bounded runtime budget");
    assert_eq!(report.runtime_mode, "kolme-live");
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "retry flow exceeded 1s budget for one submit and one finality retry"
    );
}

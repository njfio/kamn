use super::*;

fn write_temp_node_config(contents: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    let unique_suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    path.push(format!(
        "kamn-node-config-layering-{}-{unique_suffix}.conf",
        std::process::id()
    ));
    std::fs::write(&path, contents).expect("temp config file should write");
    path
}

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
    assert!(!parsed.daemon_shutdown_os_signals);
    assert_eq!(parsed.daemon_shutdown_drain_ticks, None);
    assert_eq!(parsed.daemon_shutdown_timeout_ticks, None);
    assert_eq!(parsed.daemon_peer_id, None);
    assert!(parsed.daemon_lifecycle_events.is_empty());
    assert_eq!(parsed.kolme_live_base_url, None);
    assert_eq!(parsed.kolme_live_provider_hint, None);
    assert_eq!(parsed.kolme_live_signing_profile, None);
    assert_eq!(parsed.observability_endpoint_bind_addr, None);
    assert_eq!(parsed.observability_endpoint_metrics_path, "/metrics");
    assert_eq!(parsed.observability_endpoint_health_path, "/healthz");
    assert_eq!(parsed.observability_endpoint_max_requests, 1);
    assert_eq!(parsed.observability_endpoint_idle_timeout_ms, 5_000);
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
fn unit_kolme_live_local_signer_override_marker_defaults_false() {
    let _signer_lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _override_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING", None);
    assert!(!resolve_kolme_live_allow_local_signer_testing_override()
        .expect("override marker should default false when unset"));
}

#[test]
fn unit_kolme_live_local_signer_override_marker_parses_boolean_values() {
    let _signer_lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");

    {
        let _true_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING", Some("true"));
        assert!(resolve_kolme_live_allow_local_signer_testing_override()
            .expect("true override marker should parse"));
    }
    {
        let _false_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING", Some("false"));
        assert!(!resolve_kolme_live_allow_local_signer_testing_override()
            .expect("false override marker should parse"));
    }
}

#[test]
fn regression_kolme_live_local_signer_override_marker_rejects_invalid_value() {
    let _signer_lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _override_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING", Some("maybe"));
    let error = resolve_kolme_live_allow_local_signer_testing_override()
        .expect_err("invalid override marker value must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("legacy_local_signer_path_override_invalid")),
        "invalid local signer override marker must fail with deterministic reason code"
    );
}

#[test]
fn unit_kolme_live_signer_contract_policy_allows_strict_override_or_test_build() {
    enforce_kolme_live_signer_contract_policy(true, false, false)
        .expect("strict signer contracts should satisfy policy");
    enforce_kolme_live_signer_contract_policy(false, true, false)
        .expect("explicit local testing override should satisfy policy");
    enforce_kolme_live_signer_contract_policy(false, false, true)
        .expect("test build should bypass strict signer policy guard");
}

#[test]
fn regression_kolme_live_signer_contract_policy_rejects_legacy_local_path_in_production() {
    let error = enforce_kolme_live_signer_contract_policy(false, false, false)
        .expect_err("production runtime must reject legacy local signer path");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("legacy_local_signer_path_forbidden")),
        "production policy enforcement must emit deterministic legacy local signer rejection marker"
    );
}

#[test]
fn unit_kolme_live_signer_key_source_policy_classifier_matrix() {
    assert_eq!(
        classify_kolme_live_signer_key_source_policy_violation(
            false,
            Some("env-local"),
            false,
            false,
        ),
        None
    );
    assert_eq!(
        classify_kolme_live_signer_key_source_policy_violation(
            true,
            Some("managed-external"),
            false,
            false,
        ),
        None
    );
    assert_eq!(
        classify_kolme_live_signer_key_source_policy_violation(
            true,
            Some("env-local"),
            false,
            false,
        ),
        Some("production_signer_key_source_env_local_forbidden")
    );
    assert_eq!(
        classify_kolme_live_signer_key_source_policy_violation(
            true,
            Some("env-local"),
            true,
            false,
        ),
        None
    );
    assert_eq!(
        classify_kolme_live_signer_key_source_policy_violation(
            true,
            Some("env-local"),
            false,
            true,
        ),
        None
    );
}

#[test]
pub(super) fn functional_kolme_live_strict_env_local_key_source_rejects_with_reason_code() {
    let parsed = parse_args(vec![
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
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ])
    .expect("strict args should parse");
    let error = enforce_kolme_live_signer_key_source_policy(
        parsed.kolme_live_strict_signer_contracts,
        parsed.kolme_live_signer_key_source.as_deref(),
        false,
        false,
    )
    .expect_err("strict env-local key source must fail closed for production-targeted runs");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("production_signer_key_source_env_local_forbidden")),
        "strict env-local key source policy must emit deterministic reason code"
    );
}

#[test]
pub(super) fn functional_kolme_live_strict_env_local_key_source_allows_with_local_override() {
    let parsed = parse_args(vec![
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
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ])
    .expect("strict args should parse");
    enforce_kolme_live_signer_key_source_policy(
        parsed.kolme_live_strict_signer_contracts,
        parsed.kolme_live_signer_key_source.as_deref(),
        true,
        false,
    )
    .expect("explicit local override should allow strict env-local key source");
}

#[test]
pub(super) fn integration_kolme_live_strict_managed_external_key_source_policy_passes() {
    let parsed = parse_args(vec![
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
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "managed-external".to_owned(),
    ])
    .expect("strict managed-external args should parse");
    enforce_kolme_live_signer_key_source_policy(
        parsed.kolme_live_strict_signer_contracts,
        parsed.kolme_live_signer_key_source.as_deref(),
        false,
        false,
    )
    .expect("strict managed-external key source should satisfy production policy");
}

#[test]
fn regression_runtime_kolme_live_honors_declared_managed_external_key_source_without_strict_flag() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _signer_public_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some(managed_signer_public_key_hex(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE).as_str()),
    );
    let _managed_command_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND", None);

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:39000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "managed-external".to_owned(),
    ])
    .expect("kolme-live args should parse");
    let error = execute(parsed).expect_err(
        "declared managed-external key source must be honored without strict flag in local/test execution",
    );
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_required_missing")),
        "declared managed-external key source must not silently fall back to env-local signer path"
    );
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
    let dispatch_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.mode.dispatch\""))
        .expect("runtime dispatch marker should be emitted");
    let ready_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.bootstrap.plan.ready\""))
        .expect("bootstrap ready marker should be emitted");
    let dispatch_execution_id = extract_json_string_field(dispatch_line, "execution_id")
        .expect("runtime dispatch marker should include execution_id");
    let ready_execution_id = extract_json_string_field(ready_line, "execution_id")
        .expect("bootstrap ready marker should include execution_id");
    assert_eq!(dispatch_execution_id, ready_execution_id);
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
        extract_json_string_field(submit_retry_line, "backoff_ms").as_deref(),
        Some("10")
    );
    assert_eq!(
        extract_json_string_field(submit_retry_line, "max_attempts").as_deref(),
        Some("3")
    );
    assert_eq!(
        extract_json_string_field(finality_retry_line, "reason").as_deref(),
        Some("unavailable")
    );
    assert_eq!(
        extract_json_string_field(finality_retry_line, "backoff_ms").as_deref(),
        Some("10")
    );
    assert_eq!(
        extract_json_string_field(finality_retry_line, "max_attempts").as_deref(),
        Some("3")
    );
}

#[test]
fn functional_kolme_live_nonce_retry_emits_structured_retry_marker() {
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
        MockHttpReply {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{\"error\":\"nonce unavailable\"}".to_owned(),
        },
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"final"}"#,
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
    let report = report_result.expect("runtime should recover from transient nonce failure");
    assert_eq!(report.runtime_mode, "kolme-live");

    let nonce_retry_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"kolme.live.nonce.retry\""))
        .expect("kolme-live retry flow should emit nonce retry marker");
    assert_eq!(
        extract_json_string_field(nonce_retry_line, "reason").as_deref(),
        Some("unavailable")
    );
}

#[test]
fn regression_runtime_kolme_live_rejects_fallback_signer_secret_env_with_reason_code() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _primary_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let _fallback_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
    );
    let (base_url, requests) = spawn_kolme_live_mock_server(Vec::new());
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
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ])
    .expect("kolme-live args should parse");
    let error = execute(parsed).expect_err("fallback signer secret env path must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("fallback_signer_secret_present_violation")),
        "fallback signer secret env path must emit deterministic fail-closed reason code"
    );
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        0,
        "fallback signer secret env path must fail before nonce/finality network dispatch"
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
fn parses_runtime_mode_full_with_required_controls() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "full".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19081".to_owned(),
    ];

    let parsed = parse_args(args).expect("full args should parse");
    assert_eq!(parsed.runtime_mode.as_str(), "full");
    assert_eq!(parsed.daemon_max_ticks, Some(3));
    assert_eq!(parsed.daemon_tick_interval_ms, Some(25));
    assert_eq!(parsed.api_bind_addr, Some("127.0.0.1:19081".to_owned()));
}

#[test]
fn integration_runtime_full_emits_ordered_bootstrap_readiness_markers() {
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
        "full".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "10".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19082".to_owned(),
    ])
    .expect("full args should parse");
    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("runtime-mode full execution should succeed");
    assert_eq!(report.runtime_mode, "full");
    assert_eq!(report.daemon_max_ticks, Some(2));
    let start_idx = captured_logs
        .iter()
        .position(|line| line.contains("\"event\":\"node.runtime.full.bootstrap.start\""))
        .expect("full bootstrap start marker should be emitted");
    let daemon_idx = captured_logs
        .iter()
        .position(|line| {
            line.contains("\"event\":\"node.runtime.full.bootstrap.component.ready\"")
                && line.contains("\"component\":\"daemon\"")
        })
        .expect("daemon readiness marker should be emitted");
    let api_idx = captured_logs
        .iter()
        .position(|line| {
            line.contains("\"event\":\"node.runtime.full.bootstrap.component.ready\"")
                && line.contains("\"component\":\"api\"")
        })
        .expect("api readiness marker should be emitted");
    let transport_idx = captured_logs
        .iter()
        .position(|line| {
            line.contains("\"event\":\"node.runtime.full.bootstrap.component.ready\"")
                && line.contains("\"component\":\"transport\"")
        })
        .expect("transport readiness marker should be emitted");
    let commit_idx = captured_logs
        .iter()
        .position(|line| {
            line.contains("\"event\":\"node.runtime.full.bootstrap.component.ready\"")
                && line.contains("\"component\":\"kolme-commit\"")
        })
        .expect("kolme commit readiness marker should be emitted");
    let ready_idx = captured_logs
        .iter()
        .position(|line| line.contains("\"event\":\"node.runtime.full.bootstrap.ready\""))
        .expect("full bootstrap ready marker should be emitted");
    assert!(
        start_idx < daemon_idx
            && daemon_idx < api_idx
            && api_idx < transport_idx
            && transport_idx < commit_idx
            && commit_idx < ready_idx,
        "full-mode readiness markers must preserve deterministic ordering"
    );
    let dispatch_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.mode.dispatch\""))
        .expect("runtime dispatch marker should be emitted");
    let ready_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.full.bootstrap.ready\""))
        .expect("full runtime ready marker should be emitted");
    let dispatch_execution_id = extract_json_string_field(dispatch_line, "execution_id")
        .expect("dispatch marker should include execution id");
    let ready_execution_id = extract_json_string_field(ready_line, "execution_id")
        .expect("full ready marker should include execution id");
    assert_eq!(dispatch_execution_id, ready_execution_id);
}

#[test]
fn regression_runtime_full_emits_supervisor_stop_markers_with_daemon_reason() {
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
        "full".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "10".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19084".to_owned(),
    ])
    .expect("full args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("runtime-mode full execution should succeed");
    assert_eq!(report.runtime_mode, "full");
    let stop_requested_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.full.supervisor.stop.requested\""))
        .expect("full runtime should emit supervisor stop-request marker");
    let stop_complete_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.full.supervisor.stop.complete\""))
        .expect("full runtime should emit supervisor stop-complete marker");
    assert_eq!(
        extract_json_string_field(stop_requested_line, "stop_reason").as_deref(),
        Some("daemon-execution-complete")
    );
    assert_eq!(
        extract_json_string_field(stop_complete_line, "stop_reason").as_deref(),
        Some("daemon-execution-complete")
    );
    assert_eq!(
        extract_json_string_field(stop_complete_line, "daemon_completion_reason").as_deref(),
        Some("tick-budget-exhausted")
    );
    assert_eq!(
        extract_json_string_field(stop_requested_line, "shutdown_snapshot_flush_status").as_deref(),
        Some("snapshot-not-requested")
    );
    assert_eq!(
        extract_json_string_field(stop_complete_line, "shutdown_snapshot_flush_status").as_deref(),
        Some("snapshot-not-requested")
    );
    let requested_execution_id = extract_json_string_field(stop_requested_line, "execution_id")
        .expect("stop-request marker should include execution id");
    let complete_execution_id = extract_json_string_field(stop_complete_line, "execution_id")
        .expect("stop-complete marker should include execution id");
    assert_eq!(requested_execution_id, complete_execution_id);
}

#[test]
fn unit_full_supervisor_bootstrap_component_contract_rejects_order_drift() {
    let reason = classify_full_bootstrap_component_contract_violation(&[
        "daemon",
        "transport",
        "api",
        "kolme-commit",
    ]);
    assert_eq!(
        reason,
        Some("full_supervisor_bootstrap_component_order_mismatch")
    );
}

#[test]
fn regression_full_supervisor_stop_contract_rejects_unknown_completion_reason() {
    // Regression: #3283
    let error = validate_full_supervisor_stop_contract(
        "legacy-stop-reason",
        "not-signaled",
        "snapshot-not-requested",
    )
    .expect_err("unknown supervisor stop completion reason must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeDaemonLifecycle(message) if message.contains("full_supervisor_invariant_violation:full_supervisor_stop_unknown_completion_reason")),
        "unknown supervisor stop completion reason must emit deterministic fail-closed reason code"
    );
}

#[test]
fn unit_full_supervisor_stop_contract_classifier_rejects_status_mismatch() {
    let reason = classify_full_supervisor_stop_contract_violation(
        "graceful-shutdown:signal@2;drain_ticks=1;timeout_ticks=3;ignored_signals=0",
        "not-signaled",
        "snapshot-flushed",
    );
    assert_eq!(
        reason,
        Some("full_supervisor_stop_graceful_status_mismatch")
    );
}

#[test]
fn regression_full_supervisor_stop_contract_classifier_rejects_snapshot_flush_mismatch() {
    // Regression: #3597
    let reason = classify_full_supervisor_stop_contract_violation(
        "graceful-shutdown-timeout:signal@2;drain_ticks=3;timeout_ticks=1;ignored_signals=0",
        "timeout",
        "snapshot-flushed",
    );
    assert_eq!(
        reason,
        Some("full_supervisor_stop_graceful_timeout_snapshot_flush_status_mismatch")
    );
}

#[test]
fn integration_runtime_full_emits_timeout_shutdown_supervisor_reason_codes() {
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
        "full".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "4".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "10".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "5".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "1".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19085".to_owned(),
    ])
    .expect("full args with timeout shutdown controls should parse");
    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("full runtime with timeout shutdown controls should succeed");
    assert_eq!(report.runtime_mode, "full");
    let stop_complete_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.full.supervisor.stop.complete\""))
        .expect("full runtime should emit supervisor stop-complete marker");
    assert_eq!(
        extract_json_string_field(stop_complete_line, "shutdown_drain_status").as_deref(),
        Some("timeout")
    );
    assert_eq!(
        extract_json_string_field(stop_complete_line, "shutdown_snapshot_flush_status").as_deref(),
        Some("snapshot-flush-timeout")
    );
    assert!(
        extract_json_string_field(stop_complete_line, "daemon_completion_reason")
            .as_deref()
            .is_some_and(|value| value.starts_with("graceful-shutdown-timeout:signal@")),
        "timeout shutdown flow must preserve deterministic graceful-shutdown-timeout reason marker"
    );
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
fn parses_config_file_layer_for_core_node_fields() {
    let config_path = write_temp_node_config(
        "role=listener\nchain_id=kamn-config-file\nchain_version=v0.2.0\nstorage_dir=./tmp/config-listener\nenable_gossip=false\nsync_mode=archive\noutput=json\ndiagnostics=snapshot\n",
    );
    let args = vec![
        "kamn-node".to_owned(),
        "--config-file".to_owned(),
        config_path.to_string_lossy().to_string(),
    ];

    let parsed_result = parse_args(args);
    std::fs::remove_file(config_path).expect("temp config file should clean up");
    let parsed = parsed_result.expect("config file args should parse");

    assert_eq!(parsed.role, NodeRole::Listener);
    assert_eq!(parsed.chain_id, "kamn-config-file");
    assert_eq!(parsed.chain_version, "v0.2.0");
    assert_eq!(parsed.storage_dir, "./tmp/config-listener");
    assert_eq!(parsed.sync_mode, SyncMode::Archive);
    assert!(!parsed.enable_gossip);
    assert_eq!(parsed.output_mode, OutputMode::json());
    assert_eq!(parsed.diagnostics_mode, DiagnosticsMode::snapshot());
}

#[test]
fn env_overrides_config_file_chain_id_and_sync_mode() {
    let _env_lock = signer_env_lock()
        .lock()
        .expect("env lock should guard process-level overrides");
    let _chain_id_guard = EnvVarGuard::set("KAMN_NODE_CHAIN_ID", Some("kamn-env"));
    let _sync_mode_guard = EnvVarGuard::set("KAMN_NODE_SYNC_MODE", Some("slow"));
    let config_path = write_temp_node_config(
        "role=listener\nchain_id=kamn-config-file\nsync_mode=archive\nstorage_dir=./tmp/config-listener\n",
    );
    let args = vec![
        "kamn-node".to_owned(),
        "--config-file".to_owned(),
        config_path.to_string_lossy().to_string(),
    ];

    let parsed_result = parse_args(args);
    std::fs::remove_file(config_path).expect("temp config file should clean up");
    let parsed = parsed_result.expect("config + env layered args should parse");

    assert_eq!(parsed.role, NodeRole::Listener);
    assert_eq!(parsed.chain_id, "kamn-env");
    assert_eq!(parsed.sync_mode, SyncMode::Slow);
}

#[test]
fn cli_values_override_env_and_config_layers() {
    let _env_lock = signer_env_lock()
        .lock()
        .expect("env lock should guard process-level overrides");
    let _chain_id_guard = EnvVarGuard::set("KAMN_NODE_CHAIN_ID", Some("kamn-env"));
    let config_path = write_temp_node_config("role=listener\nchain_id=kamn-config-file\n");
    let args = vec![
        "kamn-node".to_owned(),
        "--config-file".to_owned(),
        config_path.to_string_lossy().to_string(),
        "--chain-id".to_owned(),
        "kamn-cli".to_owned(),
    ];

    let parsed_result = parse_args(args);
    std::fs::remove_file(config_path).expect("temp config file should clean up");
    let parsed = parsed_result.expect("config + env + cli layered args should parse");

    assert_eq!(parsed.role, NodeRole::Listener);
    assert_eq!(parsed.chain_id, "kamn-cli");
}

#[test]
fn regression_2967_invalid_env_override_fails_closed() {
    let _env_lock = signer_env_lock()
        .lock()
        .expect("env lock should guard process-level overrides");
    let _sync_mode_guard = EnvVarGuard::set("KAMN_NODE_SYNC_MODE", Some("turbo"));
    let config_path = write_temp_node_config("role=processor\n");

    let args = vec![
        "kamn-node".to_owned(),
        "--config-file".to_owned(),
        config_path.to_string_lossy().to_string(),
    ];

    let parse_result = parse_args(args);
    std::fs::remove_file(config_path).expect("temp config file should clean up");

    assert!(
        matches!(
            parse_result,
            Err(ConfigError::InvalidSyncMode(value)) if value == "turbo"
        ),
        "invalid KAMN_NODE_SYNC_MODE override must fail closed with typed config error"
    );
}

#[test]
fn integration_config_layering_executes_bootstrap_report_with_expected_precedence() {
    let _env_lock = signer_env_lock()
        .lock()
        .expect("env lock should guard process-level overrides");
    let _chain_id_guard = EnvVarGuard::set("KAMN_NODE_CHAIN_ID", Some("kamn-env"));
    let config_path = write_temp_node_config("role=listener\nchain_id=kamn-config-file\n");
    let args = vec![
        "kamn-node".to_owned(),
        "--config-file".to_owned(),
        config_path.to_string_lossy().to_string(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--chain-id".to_owned(),
        "kamn-cli".to_owned(),
    ];

    let parsed_result = parse_args(args);
    std::fs::remove_file(config_path).expect("temp config file should clean up");
    let parsed = parsed_result.expect("layered args should parse");
    let report = execute(parsed).expect("bootstrap execution should succeed");

    assert_eq!(report.role, "processor");
    assert_eq!(report.chain_id, "kamn-cli");
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
    assert!(rendered.contains("\"content-storage:file-default\""));
    assert!(rendered.contains("\"did-registry:file-default\""));
    assert!(rendered.contains("\"task-operation-snapshot-store:file-default\""));
    assert!(rendered.contains("\"durable-guard-snapshot-store:file-default\""));
    assert!(rendered.contains("\"channel-snapshot-store:file-default\""));
    assert!(rendered.contains("\"message-lifecycle-snapshot-store:file-default\""));
    assert!(rendered.contains("\"runtime-snapshot-store:file-default\""));
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
    assert!(rendered.contains("\"kolme_live_observability_reason_code\":\"none\""));
    assert!(rendered.contains("\"kolme_live_observability_transport_checkpoint_failures\":0"));
    assert!(rendered.contains("\"kolme_live_observability_signer_checkpoint_failures\":0"));
    assert!(rendered.contains("\"kolme_live_observability_commit_checkpoint_failures\":0"));
    assert!(rendered.contains("submit_attempts=1"));
    assert!(rendered.contains("submit_retry_reason=none"));
    assert!(rendered.contains("finality_retry_attempts=1"));
    assert!(rendered.contains("finality_retry_reason=none"));
    assert!(rendered.contains("submit_retry_max_attempts=3"));
    assert!(rendered.contains("finality_retry_max_attempts=3"));
    assert!(rendered.contains("retry_backoff_base_ms=10"));
    assert!(rendered.contains("retry_backoff_cap_ms=40"));
    assert!(rendered.contains("signer_previous_profile=ops-primary"));
    assert!(rendered.contains("signer_failover_active=false"));
    assert!(rendered.contains("signer_rotation_epoch=1"));
    assert!(rendered.contains("signer_previous_rotation_epoch=1"));
    assert!(rendered.contains("signer_quorum_linkage_contract_version=v1"));
    assert!(rendered.contains("signer_quorum_required_approvals=1"));
    assert!(rendered.contains("signer_quorum_approved_signers_count=1"));
    assert!(rendered.contains("signer_quorum_profile_linked=true"));
    assert!(rendered.contains("signer_quorum_satisfied=true"));
    assert!(rendered.contains("signer_quorum_linked=true"));

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
fn functional_runtime_kolme_live_rejects_stale_signer_rotation_epoch_preflight() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
    let _previous_profile_env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE",
        Some("ops-primary"),
    );
    let _rotation_epoch_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", Some("2"));
    let _previous_rotation_epoch_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", Some("2"));
    let _key_env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
    );
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#,
    )]);
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
        "--kolme-live-signer-profile".to_owned(),
        "ops-secondary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args(args).expect("kolme-live args should parse");
    let error = execute(parsed).expect_err("stale signer rotation epoch must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("runtime_signer_rotation_epoch_stale")),
        "stale signer rotation epochs must return runtime_signer_rotation_epoch_stale reason code"
    );
}

#[test]
fn functional_runtime_kolme_live_rejects_signer_quorum_shortfall_preflight() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _required_approvals_env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS",
        Some("2"),
    );
    let _approved_signers_env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS",
        Some("ops-primary"),
    );
    let _key_env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#,
    )]);
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
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args(args).expect("kolme-live args should parse");
    let error = execute(parsed).expect_err("signer quorum shortfall must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("runtime_signer_attestation_quorum_shortfall")),
        "quorum shortfall must return runtime_signer_attestation_quorum_shortfall reason code"
    );
}

#[test]
pub(super) fn functional_runtime_kolme_live_continuous_mode_executes_multiple_cycles() {
    // Regression: #2931
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
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor-cycle-1"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:cycle-1","finality":"pending"}"#,
        ),
        MockHttpReply::ok(
            r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:cycle-1","finality":"final"}"#,
        ),
        MockHttpReply::ok(r#"{"next_nonce":18,"account_id":"acct-live-processor-cycle-2"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:cycle-2","finality":"pending"}"#,
        ),
        MockHttpReply::ok(
            r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:cycle-2","finality":"final"}"#,
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
        "--daemon-max-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "1".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    let parsed = parse_args(args).expect("kolme-live continuous args should parse");
    let report = execute(parsed).expect("kolme-live continuous execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"runtime_mode\":\"kolme-live\""));
    assert!(rendered.contains("continuous_mode=enabled"));
    assert!(rendered.contains("continuous_cycle_count=2"));
    assert!(rendered.contains("continuous_completed_cycles=2"));
    assert!(rendered.contains("continuous_cycle_interval_ms=1"));

    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        6,
        "continuous mode should execute nonce/submit/finality sequence for each cycle"
    );
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
    assert!(rendered.contains("submit_retry_max_attempts=3"));
    assert!(rendered.contains("finality_retry_max_attempts=3"));
    assert!(rendered.contains("retry_backoff_base_ms=10"));
    assert!(rendered.contains("retry_backoff_cap_ms=40"));
    assert!(rendered.contains("resolution=finality-polled"));
    assert!(rendered.contains(
        "\"kolme_live_observability_reason_code\":\"transport_finality_retry_unavailable\""
    ));
    assert!(rendered.contains("\"kolme_live_observability_transport_checkpoint_failures\":2"));
    assert!(rendered.contains("\"kolme_live_observability_signer_checkpoint_failures\":0"));
    assert!(rendered.contains("\"kolme_live_observability_commit_checkpoint_failures\":0"));

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

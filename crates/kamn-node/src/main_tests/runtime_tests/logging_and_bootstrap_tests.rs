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
fn unit_log_config_parses_bootstrap_level_with_whitespace_and_case_insensitive_inputs() {
    let config = resolve_log_config_from_inputs(Some(" WARN "), Some(" JSON "))
        .expect("bootstrap log config inputs should parse after trim/lowercase normalization");
    assert_eq!(
        config,
        NodeLogConfig {
            level: NodeLogLevel::Warn,
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
fn regression_log_renderer_projects_default_correlation_and_reason_fields_when_missing() {
    // Regression: #4120
    let line = render_log_event_line(
        NodeLogConfig {
            level: NodeLogLevel::Info,
            format: NodeLogFormat::Json,
        },
        NodeLogLevel::Info,
        "node.runtime.bootstrap.plan.ready",
        &[("component", "planner")],
    );
    assert_eq!(
        extract_json_string_field(&line, "correlation_id").as_deref(),
        Some("none"),
        "structured events must project deterministic fallback correlation marker"
    );
    assert_eq!(
        extract_json_string_field(&line, "reason_code").as_deref(),
        Some("none"),
        "structured events must project deterministic fallback reason marker"
    );
}

#[test]
fn regression_log_renderer_text_projects_default_correlation_and_reason_fields_when_missing() {
    // Regression: #4120
    let line = render_log_event_line(
        NodeLogConfig {
            level: NodeLogLevel::Info,
            format: NodeLogFormat::Text,
        },
        NodeLogLevel::Info,
        "node.runtime.bootstrap.plan.ready",
        &[("component", "planner")],
    );
    assert!(
        line.contains("correlation_id=none"),
        "text events must project deterministic fallback correlation marker"
    );
    assert!(
        line.contains("reason_code=none"),
        "text events must project deterministic fallback reason marker"
    );
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
        extract_json_string_field(submit_retry_line, "reason_code").as_deref(),
        Some("unavailable")
    );
    assert_eq!(
        extract_json_string_field(submit_retry_line, "decision").as_deref(),
        Some("retry")
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
        extract_json_string_field(finality_retry_line, "reason_code").as_deref(),
        Some("unavailable")
    );
    assert_eq!(
        extract_json_string_field(finality_retry_line, "decision").as_deref(),
        Some("retry")
    );
    assert_eq!(
        extract_json_string_field(finality_retry_line, "max_attempts").as_deref(),
        Some("3")
    );
    let submit_jitter_seed = extract_json_string_field(submit_retry_line, "jitter_seed")
        .expect("submit retry marker should include deterministic jitter seed");
    let finality_jitter_seed = extract_json_string_field(finality_retry_line, "jitter_seed")
        .expect("finality retry marker should include deterministic jitter seed");
    assert_eq!(
        submit_jitter_seed, finality_jitter_seed,
        "submit/finality retry markers should project the same jitter seed for one correlation id"
    );
    assert!(
        !submit_jitter_seed.is_empty(),
        "retry jitter seed must not be empty"
    );
    let submit_backoff_ms = extract_json_string_field(submit_retry_line, "backoff_ms")
        .expect("submit retry marker should include backoff")
        .parse::<u64>()
        .expect("submit retry backoff should parse as u64");
    let finality_backoff_ms = extract_json_string_field(finality_retry_line, "backoff_ms")
        .expect("finality retry marker should include backoff")
        .parse::<u64>()
        .expect("finality retry backoff should parse as u64");
    assert!(
        (10..=40).contains(&submit_backoff_ms),
        "submit retry backoff must stay bounded"
    );
    assert!(
        (10..=40).contains(&finality_backoff_ms),
        "finality retry backoff must stay bounded"
    );
    assert_eq!(
        submit_backoff_ms, finality_backoff_ms,
        "submit/finality retry backoff should match for same attempt and jitter seed"
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
    assert_eq!(
        extract_json_string_field(nonce_retry_line, "reason_code").as_deref(),
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


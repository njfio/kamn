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
fn regression_runtime_kolme_live_submit_retry_exhaustion_emits_terminal_decision_marker() {
    // Regression: #4110
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
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
        MockHttpReply {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{\"error\":\"submit unavailable\"}".to_owned(),
        },
        MockHttpReply {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{\"error\":\"submit unavailable\"}".to_owned(),
        },
        MockHttpReply {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{\"error\":\"submit unavailable\"}".to_owned(),
        },
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
    let error = report_result.expect_err("submit retry exhaustion must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("submit retries exhausted")),
        "submit retry exhaustion should preserve deterministic fail-closed message"
    );

    let terminal_retry_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"kolme.live.submit.retry.terminal\""))
        .expect("submit retry exhaustion should emit terminal retry decision marker");
    assert_eq!(
        extract_json_string_field(terminal_retry_line, "decision").as_deref(),
        Some("stop")
    );
    assert_eq!(
        extract_json_string_field(terminal_retry_line, "terminal_decision").as_deref(),
        Some("attempt_ceiling_reached")
    );
    assert_eq!(
        extract_json_string_field(terminal_retry_line, "reason").as_deref(),
        Some("unavailable")
    );
    assert_eq!(
        extract_json_string_field(terminal_retry_line, "reason_code").as_deref(),
        Some("unavailable")
    );
    assert_eq!(
        extract_json_string_field(terminal_retry_line, "attempt").as_deref(),
        Some("3")
    );
    assert_eq!(
        extract_json_string_field(terminal_retry_line, "max_attempts").as_deref(),
        Some("3")
    );

    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        4,
        "submit retry exhaustion should issue nonce plus three submit attempts"
    );
}

#[test]
fn functional_kolme_live_finality_retry_exhaustion_emits_terminal_decision_marker() {
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
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":17,"account_id":"acct-live-processor"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34","finality":"pending"}"#,
        ),
        MockHttpReply {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{\"error\":\"finality unavailable\"}".to_owned(),
        },
        MockHttpReply {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{\"error\":\"finality unavailable\"}".to_owned(),
        },
        MockHttpReply {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{\"error\":\"finality unavailable\"}".to_owned(),
        },
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
        "--output".to_owned(),
        "json".to_owned(),
    ])
    .expect("kolme-live args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("finality retry exhaustion should resolve gracefully");
    assert_eq!(report.runtime_mode, "kolme-live");

    let terminal_retry_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"kolme.live.finality.retry.terminal\""))
        .expect("finality retry exhaustion should emit terminal retry decision marker");
    assert_eq!(
        extract_json_string_field(terminal_retry_line, "decision").as_deref(),
        Some("stop")
    );
    assert_eq!(
        extract_json_string_field(terminal_retry_line, "terminal_decision").as_deref(),
        Some("attempt_ceiling_reached")
    );
    assert_eq!(
        extract_json_string_field(terminal_retry_line, "reason").as_deref(),
        Some("unavailable")
    );
    assert_eq!(
        extract_json_string_field(terminal_retry_line, "reason_code").as_deref(),
        Some("unavailable")
    );
    assert_eq!(
        extract_json_string_field(terminal_retry_line, "attempt").as_deref(),
        Some("3")
    );
    assert_eq!(
        extract_json_string_field(terminal_retry_line, "max_attempts").as_deref(),
        Some("3")
    );

    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("resolution=finality-unavailable"));
    assert!(rendered.contains("finality_retry_attempts=3"));
    assert!(rendered.contains("finality_retry_terminal_decision=attempt_ceiling_reached"));
    assert!(rendered.contains("retry_jitter_seed="));

    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        5,
        "finality retry exhaustion should issue nonce, submit, and three finality attempts"
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

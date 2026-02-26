#[test]
fn integration_runtime_full_emits_ordered_bootstrap_readiness_markers() {
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
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
        .unwrap_or_else(|error| error.into_inner());
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
fn integration_runtime_full_supervisor_starts_service_api_lane_before_daemon_stop() {
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
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
        "127.0.0.1:19086".to_owned(),
        "--api-idle-timeout-ms".to_owned(),
        "100".to_owned(),
    ])
    .expect("full args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("full supervisor runtime should succeed");
    assert_eq!(report.runtime_mode, "full");

    let endpoint_start_idx = captured_logs
        .iter()
        .position(|line| {
            line.contains("\"event\":\"node.runtime.service_api.endpoint.start\"")
                && line.contains("\"bind_addr\":\"127.0.0.1:19086\"")
        })
        .expect("full supervisor should emit service-api endpoint start marker");
    let supervisor_stop_complete_idx = captured_logs
        .iter()
        .enumerate()
        .skip(endpoint_start_idx)
        .find(|(_, line)| {
            line.contains("\"event\":\"node.runtime.full.supervisor.stop.complete\"")
        })
        .map(|(index, _)| index)
        .expect("full supervisor should emit stop-complete marker");
    let endpoint_complete_idx = captured_logs
        .iter()
        .enumerate()
        .skip(endpoint_start_idx)
        .find(|(_, line)| {
            line.contains("\"event\":\"node.runtime.service_api.endpoint.complete\"")
                && line.contains("\"bind_addr\":\"127.0.0.1:19086\"")
        })
        .map(|(index, _)| index)
        .expect("full supervisor should emit service-api endpoint complete marker");
    assert!(
        endpoint_start_idx < supervisor_stop_complete_idx,
        "service-api lane must start before full supervisor stop completion"
    );
    assert!(
        supervisor_stop_complete_idx < endpoint_complete_idx,
        "service-api lane should finish cleanly after full supervisor stop completion"
    );
}

#[test]
fn integration_runtime_full_supervisor_starts_observability_lane_before_daemon_stop() {
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
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
        "127.0.0.1:19090".to_owned(),
        "--api-idle-timeout-ms".to_owned(),
        "100".to_owned(),
        "--observability-endpoint-bind".to_owned(),
        "127.0.0.1:19091".to_owned(),
        "--observability-endpoint-idle-timeout-ms".to_owned(),
        "100".to_owned(),
    ])
    .expect("full args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("full supervisor runtime should succeed");
    assert_eq!(report.runtime_mode, "full");

    let endpoint_start_idx = captured_logs
        .iter()
        .position(|line| {
            line.contains("\"event\":\"node.runtime.observability.endpoint.start\"")
                && line.contains("\"bind_addr\":\"127.0.0.1:19091\"")
        })
        .expect("full supervisor should emit observability endpoint start marker");
    let supervisor_stop_complete_idx = captured_logs
        .iter()
        .enumerate()
        .skip(endpoint_start_idx)
        .find(|(_, line)| {
            line.contains("\"event\":\"node.runtime.full.supervisor.stop.complete\"")
        })
        .map(|(index, _)| index)
        .expect("full supervisor should emit stop-complete marker");
    let endpoint_complete_idx = captured_logs
        .iter()
        .enumerate()
        .skip(endpoint_start_idx)
        .find(|(_, line)| {
            line.contains("\"event\":\"node.runtime.observability.endpoint.complete\"")
                && line.contains("\"bind_addr\":\"127.0.0.1:19091\"")
        })
        .map(|(index, _)| index)
        .expect("full supervisor should emit observability endpoint complete marker");
    assert!(
        endpoint_start_idx < supervisor_stop_complete_idx,
        "observability lane must start before full supervisor stop completion"
    );
    assert!(
        supervisor_stop_complete_idx < endpoint_complete_idx,
        "observability lane should finish cleanly after full supervisor stop completion"
    );
}

#[test]
fn regression_runtime_full_supervisor_rejects_service_api_lane_max_requests_drift() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "full".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "5".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19087".to_owned(),
        "--api-max-requests".to_owned(),
        "2".to_owned(),
    ])
    .expect("full args should parse");

    let error = execute(parsed).expect_err(
        "full supervisor must fail closed when service-api lane max-request contract drifts",
    );
    assert!(
        matches!(error, ConfigError::RuntimeDaemonLifecycle(message) if message.contains("full_supervisor_service_api_lane_max_requests_contract_violation")),
        "service-api lane max-request drift must emit deterministic full-supervisor reason code"
    );
}

#[test]
fn regression_runtime_full_supervisor_rejects_observability_lane_max_requests_drift() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "full".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "5".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19089".to_owned(),
        "--observability-endpoint-bind".to_owned(),
        "127.0.0.1:19088".to_owned(),
        "--observability-endpoint-max-requests".to_owned(),
        "2".to_owned(),
    ])
    .expect("full args should parse");

    let error = execute(parsed).expect_err(
        "full supervisor must fail closed when observability lane max-request contract drifts",
    );
    assert!(
        matches!(error, ConfigError::RuntimeDaemonLifecycle(message) if message.contains("full_supervisor_observability_lane_max_requests_contract_violation")),
        "observability lane max-request drift must emit deterministic full-supervisor reason code"
    );
}

fn spawn_delayed_http_health_request(
    bind_addr: &'static str,
    path: &'static str,
    delay_ms: u64,
) -> std::thread::JoinHandle<bool> {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(750);
        let request = format!(
            "GET {path} HTTP/1.1\r\nHost: {bind_addr}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
        );

        loop {
            match std::net::TcpStream::connect(bind_addr) {
                Ok(mut stream) => {
                    let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(200)));
                    let _ = stream.set_write_timeout(Some(std::time::Duration::from_millis(200)));
                    if std::io::Write::write_all(&mut stream, request.as_bytes()).is_ok() {
                        let mut buffer = [0_u8; 128];
                        let _ = std::io::Read::read(&mut stream, &mut buffer);
                        return true;
                    }
                    return false;
                }
                Err(_) => {
                    if std::time::Instant::now() >= deadline {
                        return false;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
    })
}

#[test]
fn regression_runtime_full_supervisor_service_api_lane_early_exit_fails_with_liveness_reason() {
    let request_trigger = spawn_delayed_http_health_request("127.0.0.1:19095", "/healthz", 50);
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "full".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "250".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "2".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19095".to_owned(),
        "--api-idle-timeout-ms".to_owned(),
        "500".to_owned(),
    ])
    .expect("full args should parse");

    let error = execute(parsed).expect_err(
        "full supervisor must fail closed when service-api lane exits before daemon completion",
    );
    let request_dispatched = request_trigger
        .join()
        .expect("service-api trigger thread should join");
    assert!(
        request_dispatched,
        "service-api trigger request should be dispatched during daemon execution"
    );
    assert!(
        matches!(error, ConfigError::RuntimeDaemonLifecycle(ref message) if message.contains("full_supervisor_service_api_lane_liveness_failed")),
        "service-api lane early-exit path must emit deterministic liveness reason code: {error:?}"
    );
}

#[test]
fn regression_runtime_full_supervisor_observability_lane_early_exit_fails_with_liveness_reason() {
    let request_trigger = spawn_delayed_http_health_request("127.0.0.1:19096", "/healthz", 50);
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "full".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "250".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "2".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19097".to_owned(),
        "--api-idle-timeout-ms".to_owned(),
        "500".to_owned(),
        "--observability-endpoint-bind".to_owned(),
        "127.0.0.1:19096".to_owned(),
        "--observability-endpoint-idle-timeout-ms".to_owned(),
        "500".to_owned(),
    ])
    .expect("full args should parse");

    let error = execute(parsed).expect_err(
        "full supervisor must fail closed when observability lane exits before daemon completion",
    );
    let request_dispatched = request_trigger
        .join()
        .expect("observability trigger thread should join");
    assert!(
        request_dispatched,
        "observability trigger request should be dispatched during daemon execution"
    );
    assert!(
        matches!(error, ConfigError::RuntimeDaemonLifecycle(ref message) if message.contains("full_supervisor_observability_lane_liveness_failed")),
        "observability lane early-exit path must emit deterministic liveness reason code: {error:?}"
    );
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
fn regression_full_supervisor_stop_contract_classifier_rejects_empty_or_non_numeric_signal_tick() {
    // Regression: #4331
    let empty_tick_reason = classify_full_supervisor_stop_contract_violation(
        "graceful-shutdown:signal@;drain_ticks=1;timeout_ticks=3;ignored_signals=0",
        "completed",
        "snapshot-flushed",
    );
    assert_eq!(
        empty_tick_reason,
        Some("full_supervisor_stop_missing_signal_tick")
    );

    let non_numeric_tick_reason = classify_full_supervisor_stop_contract_violation(
        "graceful-shutdown-timeout:signal@abc;drain_ticks=3;timeout_ticks=1;ignored_signals=0",
        "timeout",
        "snapshot-flush-timeout",
    );
    assert_eq!(
        non_numeric_tick_reason,
        Some("full_supervisor_stop_missing_signal_tick")
    );
}

#[test]
fn regression_full_supervisor_stop_contract_classifier_rejects_graceful_drain_timeout_mismatch() {
    // Regression: #4332
    let reason = classify_full_supervisor_stop_contract_violation(
        "graceful-shutdown:signal@4;drain_ticks=5;timeout_ticks=2;ignored_signals=0",
        "completed",
        "snapshot-flushed",
    );
    assert_eq!(
        reason,
        Some("full_supervisor_stop_graceful_drain_timeout_contract_mismatch")
    );
}

#[test]
fn unit_full_supervisor_stop_contract_classifier_rejects_invalid_numeric_shutdown_fields() {
    let invalid_drain_reason = classify_full_supervisor_stop_contract_violation(
        "graceful-shutdown:signal@4;drain_ticks=abc;timeout_ticks=2;ignored_signals=0",
        "completed",
        "snapshot-flushed",
    );
    assert_eq!(
        invalid_drain_reason,
        Some("full_supervisor_stop_invalid_drain_ticks")
    );

    let invalid_timeout_reason = classify_full_supervisor_stop_contract_violation(
        "graceful-shutdown-timeout:signal@4;drain_ticks=3;timeout_ticks=abc;ignored_signals=0",
        "timeout",
        "snapshot-flush-timeout",
    );
    assert_eq!(
        invalid_timeout_reason,
        Some("full_supervisor_stop_invalid_timeout_ticks")
    );

    let invalid_ignored_signals_reason = classify_full_supervisor_stop_contract_violation(
        "graceful-shutdown-timeout:signal@4;drain_ticks=3;timeout_ticks=1;ignored_signals=abc",
        "timeout",
        "snapshot-flush-timeout",
    );
    assert_eq!(
        invalid_ignored_signals_reason,
        Some("full_supervisor_stop_invalid_ignored_signals")
    );
}

#[test]
fn unit_shutdown_checkpoint_reconciliation_classifier_rejects_timeout_reason_mapping_drift() {
    let reason = crate::classify_shutdown_checkpoint_reconciliation_violation(
        "graceful-shutdown-timeout:signal@2;drain_ticks=3;timeout_ticks=1;ignored_signals=0",
        "daemon_shutdown_signal",
        0,
        0,
        1,
    );
    assert_eq!(
        reason,
        Some("shutdown_checkpoint_reconciliation_timeout_reason_code_mismatch")
    );
}

#[test]
fn regression_shutdown_checkpoint_reconciliation_classifier_rejects_checkpoint_counter_drift() {
    // Regression: #4333
    let reason = crate::classify_shutdown_checkpoint_reconciliation_violation(
        "graceful-shutdown:signal@2;drain_ticks=1;timeout_ticks=3;ignored_signals=0",
        "daemon_shutdown_signal",
        0,
        0,
        1,
    );
    assert_eq!(
        reason,
        Some("shutdown_checkpoint_reconciliation_graceful_checkpoint_mismatch")
    );
}

#[test]
fn regression_shutdown_checkpoint_reconciliation_validator_fails_closed_with_stable_reason() {
    // Regression: #4333
    let error = crate::validate_shutdown_checkpoint_reconciliation(
        "tick-budget-exhausted",
        "daemon_shutdown_timeout",
        0,
        0,
        0,
    )
    .expect_err("shutdown checkpoint reconciliation drift must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeDaemonLifecycle(message) if message.contains("runtime_shutdown_invariant_violation:shutdown_checkpoint_reconciliation_not_signaled_reason_code_mismatch")),
        "shutdown checkpoint reconciliation drift must map to deterministic reason"
    );
}

#[test]
fn integration_runtime_full_emits_timeout_shutdown_supervisor_reason_codes() {
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
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
        "1".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
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
    let expected_execution_id = format!(
        "node-runtime:{}:{}:{}",
        report.runtime_mode, report.chain_id, report.role
    );
    let stop_complete_line = captured_logs
        .iter()
        .find(|line| {
            line.contains("\"event\":\"node.runtime.full.supervisor.stop.complete\"")
                && extract_json_string_field(line, "execution_id").as_deref()
                    == Some(expected_execution_id.as_str())
                && extract_json_string_field(line, "daemon_completion_reason")
                    .as_deref()
                    .is_some_and(|value| value.starts_with("graceful-shutdown-timeout:signal@"))
        })
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
    assert_eq!(
        report.daemon_observability_reason_code.as_deref(),
        Some("daemon_shutdown_timeout")
    );
    assert_eq!(
        report.daemon_observability_transport_checkpoint_failures,
        Some(0)
    );
    assert_eq!(
        report.daemon_observability_signer_checkpoint_failures,
        Some(0)
    );
    assert_eq!(
        report.daemon_observability_commit_checkpoint_failures,
        Some(1)
    );
}

#[cfg(unix)]
#[test]
fn regression_runtime_full_os_signal_stop_markers_project_shutdown_field_parity() {
    // Regression: #3732
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "full".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "100".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "1".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19086".to_owned(),
    ])
    .expect("full args should parse");
    configure_os_signal_test_triggers(vec![OsSignalTestTrigger::new(5, OsSignalTestKind::Sigterm)]);
    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("full runtime os-signal execution should succeed");
    let stop_complete_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.full.supervisor.stop.complete\""))
        .expect("full runtime should emit supervisor stop-complete marker");
    let completion_reason = report
        .daemon_completion_reason
        .as_deref()
        .expect("daemon completion reason should be present");
    assert!(
        completion_reason.starts_with("graceful-shutdown:signal@"),
        "full runtime os-signal flow must preserve graceful shutdown reason marker"
    );
    assert_eq!(
        extract_json_string_field(stop_complete_line, "daemon_completion_reason").as_deref(),
        Some(completion_reason)
    );
    assert!(
        extract_json_string_field(stop_complete_line, "shutdown_signal_tick").is_some(),
        "supervisor stop marker should include shutdown_signal_tick parity field"
    );
    assert!(
        extract_json_string_field(stop_complete_line, "shutdown_drain_ticks").is_some(),
        "supervisor stop marker should include shutdown_drain_ticks parity field"
    );
    assert!(
        extract_json_string_field(stop_complete_line, "shutdown_timeout_ticks").is_some(),
        "supervisor stop marker should include shutdown_timeout_ticks parity field"
    );
    assert!(
        extract_json_string_field(stop_complete_line, "shutdown_ignored_signals").is_some(),
        "supervisor stop marker should include shutdown_ignored_signals parity field"
    );
}

#[cfg(unix)]
#[test]
fn regression_runtime_full_os_signal_timeout_stop_markers_project_shutdown_field_parity() {
    // Regression: #4330
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "full".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "100".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-os-signals".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "5".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "1".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19087".to_owned(),
    ])
    .expect("full args with os-signal timeout controls should parse");

    configure_os_signal_test_triggers(vec![OsSignalTestTrigger::new(5, OsSignalTestKind::Sigint)]);
    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("full runtime os-signal timeout execution should succeed");
    let stop_complete_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"node.runtime.full.supervisor.stop.complete\""))
        .expect("full runtime should emit supervisor stop-complete marker");
    let completion_reason = report
        .daemon_completion_reason
        .as_deref()
        .expect("daemon completion reason should be present");
    assert!(
        completion_reason.starts_with("graceful-shutdown-timeout:signal@"),
        "full runtime os-signal timeout flow must preserve graceful-shutdown-timeout reason marker"
    );
    assert_eq!(
        extract_json_string_field(stop_complete_line, "daemon_completion_reason").as_deref(),
        Some(completion_reason)
    );
    assert_eq!(
        extract_json_string_field(stop_complete_line, "shutdown_drain_status").as_deref(),
        Some("timeout")
    );
    assert_eq!(
        extract_json_string_field(stop_complete_line, "shutdown_snapshot_flush_status").as_deref(),
        Some("snapshot-flush-timeout")
    );
    assert!(
        extract_json_string_field(stop_complete_line, "shutdown_signal_tick").is_some(),
        "timeout stop marker should include shutdown_signal_tick parity field"
    );
    assert!(
        extract_json_string_field(stop_complete_line, "shutdown_drain_ticks").is_some(),
        "timeout stop marker should include shutdown_drain_ticks parity field"
    );
    assert!(
        extract_json_string_field(stop_complete_line, "shutdown_timeout_ticks").is_some(),
        "timeout stop marker should include shutdown_timeout_ticks parity field"
    );
    assert!(
        extract_json_string_field(stop_complete_line, "shutdown_ignored_signals").is_some(),
        "timeout stop marker should include shutdown_ignored_signals parity field"
    );
}

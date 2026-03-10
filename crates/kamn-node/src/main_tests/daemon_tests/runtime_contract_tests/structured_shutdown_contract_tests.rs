#[test]
fn functional_runtime_daemon_graceful_shutdown_emits_structured_drain_markers() {
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let chain_id = "daemon-graceful-contract";
    let parsed = parse_args_with_clean_daemon_env(vec![
        "kamn-node".to_owned(),
        "--chain-id".to_owned(),
        chain_id.to_owned(),
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
    ])
    .expect("daemon args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("daemon execution should succeed");
    assert_eq!(report.runtime_mode, "daemon");
    let expected_execution_id = format!("node-runtime:daemon:{chain_id}:processor");

    let complete_line = captured_logs
        .iter()
        .find(|line| {
            line.contains("\"event\":\"node.runtime.daemon.execute.complete\"")
                && extract_json_string_field(line, "execution_id").as_deref()
                    == Some(expected_execution_id.as_str())
        })
        .expect("daemon execution should emit structured completion marker");
    assert_eq!(
        extract_json_string_field(complete_line, "completion_reason").as_deref(),
        Some("graceful-shutdown:signal@3;drain_ticks=2;timeout_ticks=4;ignored_signals=0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_status").as_deref(),
        Some("completed")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_signal_tick").as_deref(),
        Some("3")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_ticks").as_deref(),
        Some("2")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_timeout_ticks").as_deref(),
        Some("4")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_ignored_signals").as_deref(),
        Some("0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_snapshot_flush_status").as_deref(),
        Some("snapshot-flushed")
    );
}

#[test]
pub(super) fn regression_runtime_daemon_shutdown_timeout_emits_structured_timeout_drain_markers() {
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let chain_id = "daemon-timeout-contract";
    let parsed = parse_args_with_clean_daemon_env(vec![
        "kamn-node".to_owned(),
        "--chain-id".to_owned(),
        chain_id.to_owned(),
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
    ])
    .expect("daemon timeout args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("daemon timeout execution should succeed");
    assert_eq!(report.runtime_mode, "daemon");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"daemon_convergence_decision\":\"no_go\""));
    assert!(rendered.contains(
        "\"daemon_convergence_reason_code\":\"convergence_performance_budget_exceeded\""
    ));
    let expected_execution_id = format!("node-runtime:daemon:{chain_id}:processor");

    let complete_line = captured_logs
        .iter()
        .find(|line| {
            line.contains("\"event\":\"node.runtime.daemon.execute.complete\"")
                && extract_json_string_field(line, "execution_id").as_deref()
                    == Some(expected_execution_id.as_str())
        })
        .expect("daemon execution should emit structured completion marker");
    assert_eq!(
        extract_json_string_field(complete_line, "completion_reason").as_deref(),
        Some("graceful-shutdown-timeout:signal@7;drain_ticks=4;timeout_ticks=2;ignored_signals=0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_status").as_deref(),
        Some("timeout")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_signal_tick").as_deref(),
        Some("7")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_ticks").as_deref(),
        Some("4")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_timeout_ticks").as_deref(),
        Some("2")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_ignored_signals").as_deref(),
        Some("0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_snapshot_flush_status").as_deref(),
        Some("snapshot-flush-timeout")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "convergence_decision").as_deref(),
        Some("no_go")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "convergence_reason_code").as_deref(),
        Some("convergence_performance_budget_exceeded")
    );
}


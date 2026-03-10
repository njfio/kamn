#[test]
fn functional_runtime_daemon_emits_structured_transition_markers() {
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let chain_id = "daemon-transition-contract";
    let parsed = parse_args_with_clean_daemon_env(vec![
        "kamn-node".to_owned(),
        "--chain-id".to_owned(),
        chain_id.to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "99".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "1".to_owned(),
    ])
    .expect("daemon args should parse");

    let (report_result, captured_logs) = capture_test_logs(|| execute(parsed));
    let report = report_result.expect("daemon execution should succeed");
    assert_eq!(report.runtime_mode, "daemon");
    let expected_execution_id = format!("node-runtime:daemon:{chain_id}:processor");

    let start_line = captured_logs
        .iter()
        .find(|line| {
            line.contains("\"event\":\"node.runtime.daemon.execute.start\"")
                && extract_json_string_field(line, "execution_id").as_deref()
                    == Some(expected_execution_id.as_str())
        })
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
    let start_execution_id = extract_json_string_field(start_line, "execution_id")
        .expect("daemon start marker should include execution_id");

    let complete_line = captured_logs
        .iter()
        .find(|line| {
            line.contains("\"event\":\"node.runtime.daemon.execute.complete\"")
                && extract_json_string_field(line, "execution_id").as_deref()
                    == Some(expected_execution_id.as_str())
        })
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
        Some("tick-budget-exhausted;ignored_signals=1")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_status").as_deref(),
        Some("not-signaled")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_snapshot_flush_status").as_deref(),
        Some("snapshot-not-requested")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_signal_tick").as_deref(),
        Some("none")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_drain_ticks").as_deref(),
        Some("0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_timeout_ticks").as_deref(),
        Some("0")
    );
    assert_eq!(
        extract_json_string_field(complete_line, "shutdown_ignored_signals").as_deref(),
        Some("1")
    );
    let complete_execution_id = extract_json_string_field(complete_line, "execution_id")
        .expect("daemon completion marker should include execution_id");
    assert_eq!(start_execution_id, complete_execution_id);
}


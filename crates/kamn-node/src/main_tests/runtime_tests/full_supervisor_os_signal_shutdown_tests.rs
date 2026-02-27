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

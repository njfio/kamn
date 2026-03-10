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
        "--daemon-shutdown-signal-tick".to_owned(),
        "99".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "1".to_owned(),
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

    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"runtime_mode\":\"daemon\""));
    assert!(rendered.contains("\"daemon_max_ticks\":3"));
    assert!(rendered.contains("\"daemon_tick_interval_ms\":25"));
    assert!(rendered.contains("\"daemon_executed_ticks\":3"));
    assert!(rendered
        .contains("\"daemon_completion_reason\":\"tick-budget-exhausted;ignored_signals=1\""));
    assert!(rendered.contains("\"daemon_observability_latency_p50_ms\":1"));
    assert!(rendered.contains("\"daemon_observability_latency_p99_ms\":1"));
    assert!(rendered.contains("\"daemon_observability_throughput_tps\":1000"));
    assert!(rendered.contains("\"daemon_observability_error_rate_bps\":0"));
    assert!(rendered.contains("\"daemon_observability_availability_bps\":10000"));
    assert!(rendered.contains("\"daemon_observability_health\":\"healthy\""));
    assert!(rendered.contains("\"daemon_observability_alert_count\":0"));
    assert!(rendered.contains("\"daemon_observability_reason_code\":\"none\""));
    assert!(rendered.contains("\"daemon_observability_transport_checkpoint_failures\":0"));
    assert!(rendered.contains("\"daemon_observability_signer_checkpoint_failures\":0"));
    assert!(rendered.contains("\"daemon_observability_commit_checkpoint_failures\":0"));
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

    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon shutdown args should parse");
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

    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon timeout args should parse");
    let report = execute(parsed).expect("daemon timeout execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"daemon_executed_ticks\":9"));
    assert!(rendered.contains(
        "\"daemon_completion_reason\":\"graceful-shutdown-timeout:signal@7;drain_ticks=4;timeout_ticks=2;ignored_signals=0\""
    ));
    assert!(rendered.contains("\"daemon_observability_latency_p50_ms\":1"));
    assert!(rendered.contains("\"daemon_observability_latency_p99_ms\":1"));
    assert!(rendered.contains("\"daemon_observability_throughput_tps\":1000"));
    assert!(rendered.contains("\"daemon_observability_error_rate_bps\":500"));
    assert!(rendered.contains("\"daemon_observability_availability_bps\":9500"));
    assert!(rendered.contains("\"daemon_observability_health\":\"critical\""));
    assert!(rendered.contains("\"daemon_observability_alert_count\":2"));
    assert!(rendered.contains("\"daemon_observability_reason_code\":\"daemon_shutdown_timeout\""));
    assert!(rendered.contains("\"daemon_observability_transport_checkpoint_failures\":0"));
    assert!(rendered.contains("\"daemon_observability_signer_checkpoint_failures\":0"));
    assert!(rendered.contains("\"daemon_observability_commit_checkpoint_failures\":1"));
}

#[cfg(unix)]
#[test]
pub(super) fn integration_runtime_daemon_applies_graceful_shutdown_on_os_signal() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "100".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-os-signals".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "5".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed =
        parse_args_with_clean_daemon_env(args).expect("daemon os-signal args should parse");
    configure_os_signal_test_triggers(vec![OsSignalTestTrigger::new(5, OsSignalTestKind::Sigterm)]);
    let report = execute(parsed).expect("daemon os-signal execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"daemon_completion_reason\":\"graceful-shutdown:signal@"));
}


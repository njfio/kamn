#[test]
fn parses_runtime_mode_daemon_with_bounded_controls() {
    let parsed = parse_daemon(bounded_control_args());
    assert_bounded_controls(&parsed);
}

#[test]
fn parses_runtime_mode_daemon_with_shutdown_controls() {
    let parsed = parse_daemon(&[
        "--daemon-max-ticks",
        "8",
        "--daemon-tick-interval-ms",
        "25",
        "--daemon-shutdown-signal-tick",
        "3",
        "--daemon-shutdown-drain-ticks",
        "2",
        "--daemon-shutdown-timeout-ticks",
        "4",
    ]);
    assert_eq!(parsed.daemon_shutdown_signal_ticks, vec![3]);
    assert!(!parsed.daemon_shutdown_os_signals);
    assert_eq!(parsed.daemon_shutdown_drain_ticks, Some(2));
    assert_eq!(parsed.daemon_shutdown_timeout_ticks, Some(4));
}

#[test]
fn parses_runtime_mode_daemon_with_os_signal_shutdown_controls() {
    let parsed = parse_daemon(&[
        "--daemon-max-ticks",
        "12",
        "--daemon-tick-interval-ms",
        "5",
        "--daemon-shutdown-os-signals",
        "--daemon-shutdown-drain-ticks",
        "2",
        "--daemon-shutdown-timeout-ticks",
        "4",
    ]);
    assert_eq!(parsed.daemon_shutdown_signal_ticks, Vec::<u64>::new());
    assert!(parsed.daemon_shutdown_os_signals);
    assert_eq!(parsed.daemon_shutdown_drain_ticks, Some(2));
    assert_eq!(parsed.daemon_shutdown_timeout_ticks, Some(4));
}

#[test]
fn parses_runtime_mode_daemon_with_observability_endpoint_controls() {
    let parsed = parse_daemon(observability_control_args());
    assert_observability_controls(&parsed);
}

#[test]
fn env_only_daemon_controls_parse_without_config_file() {
    let _env_lock = daemon_test_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _max_ticks_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_MAX_TICKS", Some("12"));
    let _tick_interval_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_TICK_INTERVAL_MS", Some("25"));

    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
    ];

    let parsed = parse_args(args).expect("env-only daemon controls should parse");
    assert_eq!(parsed.daemon_max_ticks, Some(12));
    assert_eq!(parsed.daemon_tick_interval_ms, Some(25));
}

#[test]
fn regression_3202_invalid_daemon_env_override_fails_closed_without_config_file() {
    let _env_lock = daemon_test_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _max_ticks_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_MAX_TICKS", Some("invalid"));
    let _tick_interval_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_TICK_INTERVAL_MS", Some("25"));

    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
    ];

    let parse_result = parse_args(args);
    assert!(
        matches!(
            parse_result,
            Err(ConfigError::InvalidDaemonControlArgument(value)) if value == "invalid"
        ),
        "invalid daemon env override must fail closed with typed daemon control error"
    );
}

fn bounded_control_args() -> &'static [&'static str] {
    &[
        "--daemon-max-ticks",
        "3",
        "--daemon-tick-interval-ms",
        "25",
        "--daemon-shutdown-signal-tick",
        "99",
        "--daemon-shutdown-drain-ticks",
        "1",
        "--daemon-shutdown-timeout-ticks",
        "1",
        "--daemon-peer-id",
        "peer-alpha",
        "--daemon-lifecycle-event",
        "start-connect",
        "--daemon-lifecycle-event",
        "handshake-succeeded",
    ]
}

fn observability_control_args() -> &'static [&'static str] {
    &[
        "--daemon-max-ticks",
        "12",
        "--daemon-tick-interval-ms",
        "5",
        "--observability-endpoint-bind",
        "127.0.0.1:9108",
        "--observability-endpoint-metrics-path",
        "/runtime/metrics",
        "--observability-endpoint-health-path",
        "/runtime/health",
        "--observability-endpoint-max-requests",
        "3",
        "--observability-endpoint-idle-timeout-ms",
        "1200",
    ]
}

fn assert_bounded_controls(parsed: &crate::NodeCli) {
    assert_eq!(parsed.runtime_mode, RuntimeMode::daemon());
    assert_eq!(parsed.daemon_max_ticks, Some(3));
    assert_eq!(parsed.daemon_tick_interval_ms, Some(25));
    assert!(!parsed.daemon_shutdown_os_signals);
    assert_eq!(parsed.daemon_peer_id, Some("peer-alpha".to_owned()));
    assert_eq!(parsed.daemon_lifecycle_events.len(), 2);
}

fn assert_observability_controls(parsed: &crate::NodeCli) {
    assert_eq!(
        parsed.observability_endpoint_bind_addr,
        Some("127.0.0.1:9108".to_owned())
    );
    assert_eq!(
        parsed.observability_endpoint_metrics_path,
        "/runtime/metrics"
    );
    assert_eq!(parsed.observability_endpoint_health_path, "/runtime/health");
    assert_eq!(parsed.observability_endpoint_max_requests, 3);
    assert_eq!(parsed.observability_endpoint_idle_timeout_ms, 1200);
}

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
    ];

    let parsed = parse_args_with_clean_daemon_env(args).expect("daemon args should parse");
    assert_eq!(parsed.runtime_mode, RuntimeMode::daemon());
    assert_eq!(parsed.daemon_max_ticks, Some(3));
    assert_eq!(parsed.daemon_tick_interval_ms, Some(25));
    assert!(!parsed.daemon_shutdown_os_signals);
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

    let parsed = parse_args_with_clean_daemon_env(args)
        .expect("daemon args with shutdown controls should parse");
    assert_eq!(parsed.daemon_shutdown_signal_ticks, vec![3]);
    assert!(!parsed.daemon_shutdown_os_signals);
    assert_eq!(parsed.daemon_shutdown_drain_ticks, Some(2));
    assert_eq!(parsed.daemon_shutdown_timeout_ticks, Some(4));
}

#[test]
fn parses_runtime_mode_daemon_with_os_signal_shutdown_controls() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "12".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "5".to_owned(),
        "--daemon-shutdown-os-signals".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "4".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args)
        .expect("daemon args with os signal controls should parse");
    assert_eq!(parsed.daemon_shutdown_signal_ticks, Vec::<u64>::new());
    assert!(parsed.daemon_shutdown_os_signals);
    assert_eq!(parsed.daemon_shutdown_drain_ticks, Some(2));
    assert_eq!(parsed.daemon_shutdown_timeout_ticks, Some(4));
}

#[test]
fn parses_runtime_mode_daemon_with_observability_endpoint_controls() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "12".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "5".to_owned(),
        "--observability-endpoint-bind".to_owned(),
        "127.0.0.1:9108".to_owned(),
        "--observability-endpoint-metrics-path".to_owned(),
        "/runtime/metrics".to_owned(),
        "--observability-endpoint-health-path".to_owned(),
        "/runtime/health".to_owned(),
        "--observability-endpoint-max-requests".to_owned(),
        "3".to_owned(),
        "--observability-endpoint-idle-timeout-ms".to_owned(),
        "1200".to_owned(),
    ];

    let parsed = parse_args_with_clean_daemon_env(args)
        .expect("daemon args with observability endpoint should parse");
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


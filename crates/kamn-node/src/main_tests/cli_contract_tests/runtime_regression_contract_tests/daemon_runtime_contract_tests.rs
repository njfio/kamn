use super::super::*;

#[test]
fn regression_runtime_daemon_rejects_zero_tick_budget() {
    assert_parse_error(
        with_pairs(
            daemon_args(),
            &[
                ("--daemon-max-ticks", "0"),
                ("--daemon-tick-interval-ms", "25"),
            ],
        ),
        ConfigError::InvalidDaemonControlArgument("0".to_owned()),
    );
}

#[test]
fn regression_runtime_daemon_rejects_invalid_lifecycle_transition() {
    let args = with_pairs(
        daemon_args(),
        &[
            ("--daemon-max-ticks", "3"),
            ("--daemon-tick-interval-ms", "25"),
            ("--daemon-peer-id", "peer-alpha"),
            ("--daemon-lifecycle-event", "handshake-succeeded"),
        ],
    );
    let parsed = parse_cli(args, "daemon args should parse");
    assert_eq!(
        execute(parsed),
        Err(ConfigError::RuntimeDaemonLifecycle(
            "invalid peer lifecycle transition from Disconnected via HandshakeSucceeded".to_owned()
        ))
    );
}

#[test]
fn regression_runtime_daemon_ignores_replayed_and_late_shutdown_signals() {
    let report = execute_cli(
        daemon_shutdown_replay_args(),
        "daemon replay execution should succeed",
    );
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"daemon_executed_ticks\":5"));
    assert!(rendered.contains(
        "\"daemon_completion_reason\":\"graceful-shutdown:signal@3;drain_ticks=2;timeout_ticks=4;ignored_signals=2\""
    ));
}

#[test]
fn performance_runtime_daemon_shutdown_drain_stays_bounded_by_timeout_budget() {
    let report = execute_cli(
        daemon_timeout_args(),
        "daemon bounded shutdown should succeed",
    );
    let executed_ticks = report
        .daemon_executed_ticks
        .expect("daemon execution must report executed ticks");
    assert!(
        executed_ticks <= 9,
        "shutdown drain execution must remain within max tick budget"
    );
    assert_eq!(
        report.daemon_completion_reason.as_deref(),
        Some("graceful-shutdown-timeout:signal@8;drain_ticks=5;timeout_ticks=1;ignored_signals=0")
    );
}

fn daemon_shutdown_replay_args() -> Vec<String> {
    with_pairs(
        daemon_args(),
        &[
            ("--daemon-max-ticks", "8"),
            ("--daemon-tick-interval-ms", "25"),
            ("--daemon-shutdown-signal-tick", "3"),
            ("--daemon-shutdown-signal-tick", "7"),
            ("--daemon-shutdown-signal-tick", "11"),
            ("--daemon-shutdown-drain-ticks", "2"),
            ("--daemon-shutdown-timeout-ticks", "4"),
            ("--output", "json"),
        ],
    )
}

fn daemon_timeout_args() -> Vec<String> {
    with_pairs(
        daemon_args(),
        &[
            ("--daemon-max-ticks", "9"),
            ("--daemon-tick-interval-ms", "25"),
            ("--daemon-shutdown-signal-tick", "8"),
            ("--daemon-shutdown-drain-ticks", "5"),
            ("--daemon-shutdown-timeout-ticks", "1"),
        ],
    )
}

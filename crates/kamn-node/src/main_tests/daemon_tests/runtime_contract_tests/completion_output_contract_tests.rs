#[test]
fn integration_runtime_daemon_renders_bounded_completion_output() {
    let rendered = daemon_bounded_completion_json();
    assert_bounded_completion_output(&rendered);
}

#[test]
fn functional_runtime_daemon_applies_graceful_shutdown_signal() {
    let rendered = daemon_graceful_shutdown_json();
    assert_graceful_shutdown_output(&rendered);
}

#[test]
fn integration_runtime_daemon_shutdown_timeout_is_fail_closed() {
    let rendered = daemon_shutdown_timeout_json();
    assert_shutdown_timeout_output(&rendered);
}

#[cfg(unix)]
#[test]
pub(super) fn integration_runtime_daemon_applies_graceful_shutdown_on_os_signal() {
    let rendered = daemon_os_signal_shutdown_json();
    assert!(rendered.contains("\"daemon_completion_reason\":\"graceful-shutdown:signal@"));
}

fn assert_bounded_completion_output(rendered: &str) {
    assert_bounded_runtime_markers(rendered);
    assert_bounded_observability_markers(rendered);
    assert_bounded_peer_lifecycle_markers(rendered);
}

fn assert_bounded_runtime_markers(rendered: &str) {
    assert_rendered_contains_all(
        rendered,
        &[
            "\"runtime_mode\":\"daemon\"",
            "\"daemon_max_ticks\":3",
            "\"daemon_tick_interval_ms\":25",
            "\"daemon_executed_ticks\":3",
            "\"daemon_completion_reason\":\"tick-budget-exhausted;ignored_signals=1\"",
        ],
    );
}

fn assert_bounded_observability_markers(rendered: &str) {
    assert_rendered_contains_all(
        rendered,
        &[
            "\"daemon_observability_latency_p50_ms\":1",
            "\"daemon_observability_latency_p99_ms\":1",
            "\"daemon_observability_throughput_tps\":1000",
            "\"daemon_observability_error_rate_bps\":0",
            "\"daemon_observability_availability_bps\":10000",
            "\"daemon_observability_health\":\"healthy\"",
            "\"daemon_observability_alert_count\":0",
            "\"daemon_observability_reason_code\":\"none\"",
            "\"daemon_observability_transport_checkpoint_failures\":0",
            "\"daemon_observability_signer_checkpoint_failures\":0",
            "\"daemon_observability_commit_checkpoint_failures\":0",
        ],
    );
}

fn assert_bounded_peer_lifecycle_markers(rendered: &str) {
    assert_rendered_contains_all(
        rendered,
        &[
            "\"daemon_peer_id\":\"peer-alpha\"",
            "\"daemon_peer_lifecycle_final_state\":\"active\"",
            "\"daemon_peer_lifecycle_applied_events\":[\"start-connect\",\"handshake-succeeded\",\"heartbeat-missed\",\"heartbeat-restored\"]",
        ],
    );
}

fn assert_graceful_shutdown_output(rendered: &str) {
    assert_rendered_contains_all(
        rendered,
        &[
            "\"daemon_executed_ticks\":5",
            "\"daemon_completion_reason\":\"graceful-shutdown:signal@3;drain_ticks=2;timeout_ticks=4;ignored_signals=0\"",
            "\"daemon_observability_health\":\"healthy\"",
            "\"daemon_observability_alert_count\":0",
        ],
    );
}

fn assert_shutdown_timeout_output(rendered: &str) {
    assert_rendered_contains_all(
        rendered,
        &[
            "\"daemon_executed_ticks\":9",
            "\"daemon_completion_reason\":\"graceful-shutdown-timeout:signal@7;drain_ticks=4;timeout_ticks=2;ignored_signals=0\"",
            "\"daemon_observability_latency_p50_ms\":1",
            "\"daemon_observability_latency_p99_ms\":1",
            "\"daemon_observability_throughput_tps\":1000",
            "\"daemon_observability_error_rate_bps\":500",
            "\"daemon_observability_availability_bps\":9500",
            "\"daemon_observability_health\":\"critical\"",
            "\"daemon_observability_alert_count\":2",
            "\"daemon_observability_reason_code\":\"daemon_shutdown_timeout\"",
            "\"daemon_observability_transport_checkpoint_failures\":0",
            "\"daemon_observability_signer_checkpoint_failures\":0",
            "\"daemon_observability_commit_checkpoint_failures\":1",
        ],
    );
}

fn daemon_bounded_completion_json() -> String {
    execute_daemon_json(BOUNDED_COMPLETION_ARGS)
}

fn daemon_graceful_shutdown_json() -> String {
    execute_daemon_json(&[
        "--daemon-max-ticks",
        "10",
        "--daemon-tick-interval-ms",
        "25",
        "--daemon-shutdown-signal-tick",
        "3",
        "--daemon-shutdown-drain-ticks",
        "2",
        "--daemon-shutdown-timeout-ticks",
        "4",
        "--output",
        "json",
    ])
}

fn daemon_shutdown_timeout_json() -> String {
    execute_daemon_json(&[
        "--daemon-max-ticks",
        "10",
        "--daemon-tick-interval-ms",
        "25",
        "--daemon-shutdown-signal-tick",
        "7",
        "--daemon-shutdown-drain-ticks",
        "4",
        "--daemon-shutdown-timeout-ticks",
        "2",
        "--output",
        "json",
    ])
}

#[cfg(unix)]
fn daemon_os_signal_shutdown_json() -> String {
    configure_os_signal_test_triggers(vec![OsSignalTestTrigger::new(5, OsSignalTestKind::Sigterm)]);
    execute_daemon_json(&[
        "--daemon-max-ticks",
        "100",
        "--daemon-tick-interval-ms",
        "1",
        "--daemon-shutdown-os-signals",
        "--daemon-shutdown-drain-ticks",
        "2",
        "--daemon-shutdown-timeout-ticks",
        "5",
        "--output",
        "json",
    ])
}

const BOUNDED_COMPLETION_ARGS: &[&str] = &[
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
    "--daemon-lifecycle-event",
    "heartbeat-missed",
    "--daemon-lifecycle-event",
    "heartbeat-restored",
    "--output",
    "json",
];

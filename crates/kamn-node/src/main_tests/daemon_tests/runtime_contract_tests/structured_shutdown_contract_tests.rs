#[test]
fn functional_runtime_daemon_graceful_shutdown_emits_structured_drain_markers() {
    let (_, captured_logs, execution_id) = capture_graceful_shutdown_logs();
    assert_graceful_shutdown_complete_log(&captured_logs, &execution_id);
}

#[test]
pub(super) fn regression_runtime_daemon_shutdown_timeout_emits_structured_timeout_drain_markers() {
    let (rendered, captured_logs, execution_id) = capture_shutdown_timeout_logs();
    assert_timeout_rendered_markers(&rendered);
    assert_timeout_complete_log(&captured_logs, &execution_id);
}

fn capture_graceful_shutdown_logs() -> (String, Vec<String>, String) {
    let _guards = runtime_json_log_guards();
    capture_daemon_json_with_chain(
        "daemon-graceful-contract",
        &[
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
        ],
    )
}

fn assert_graceful_shutdown_complete_log(captured_logs: &[String], execution_id: &str) {
    let complete_line = find_daemon_log(
        captured_logs,
        "\"event\":\"node.runtime.daemon.execute.complete\"",
        execution_id,
    );
    assert_json_log_fields(
        complete_line,
        &[
            (
                "completion_reason",
                "graceful-shutdown:signal@3;drain_ticks=2;timeout_ticks=4;ignored_signals=0",
            ),
            ("shutdown_drain_status", "completed"),
            ("shutdown_signal_tick", "3"),
            ("shutdown_drain_ticks", "2"),
            ("shutdown_timeout_ticks", "4"),
            ("shutdown_ignored_signals", "0"),
            ("shutdown_snapshot_flush_status", "snapshot-flushed"),
        ],
    );
}

fn capture_shutdown_timeout_logs() -> (String, Vec<String>, String) {
    let _guards = runtime_json_log_guards();
    capture_daemon_json_with_chain(
        "daemon-timeout-contract",
        &[
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
        ],
    )
}

fn assert_timeout_rendered_markers(rendered: &str) {
    assert_rendered_contains_all(
        rendered,
        &[
            "\"daemon_convergence_decision\":\"no_go\"",
            "\"daemon_convergence_reason_code\":\"convergence_performance_budget_exceeded\"",
        ],
    );
}

fn assert_timeout_complete_log(captured_logs: &[String], execution_id: &str) {
    let complete_line = find_daemon_log(
        captured_logs,
        "\"event\":\"node.runtime.daemon.execute.complete\"",
        execution_id,
    );
    assert_json_log_fields(
        complete_line,
        &[
            ("completion_reason", "graceful-shutdown-timeout:signal@7;drain_ticks=4;timeout_ticks=2;ignored_signals=0"),
            ("shutdown_drain_status", "timeout"),
            ("shutdown_signal_tick", "7"),
            ("shutdown_drain_ticks", "4"),
            ("shutdown_timeout_ticks", "2"),
            ("shutdown_ignored_signals", "0"),
            ("shutdown_snapshot_flush_status", "snapshot-flush-timeout"),
            ("convergence_decision", "no_go"),
            ("convergence_reason_code", "convergence_performance_budget_exceeded"),
        ],
    );
}

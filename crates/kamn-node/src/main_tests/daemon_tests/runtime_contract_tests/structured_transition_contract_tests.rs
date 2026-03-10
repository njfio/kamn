#[test]
fn functional_runtime_daemon_emits_structured_transition_markers() {
    let (_rendered, captured_logs, execution_id) = capture_transition_logs();
    assert_transition_start_log(&captured_logs, &execution_id);
    assert_transition_complete_log(&captured_logs, &execution_id);
}

fn capture_transition_logs() -> (String, Vec<String>, String) {
    let _guards = runtime_json_log_guards();
    capture_daemon_json_with_chain(
        "daemon-transition-contract",
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
        ],
    )
}

fn assert_transition_start_log(captured_logs: &[String], execution_id: &str) {
    let start_line = find_daemon_log(
        captured_logs,
        "\"event\":\"node.runtime.daemon.execute.start\"",
        execution_id,
    );
    assert_json_log_fields(
        start_line,
        &[
            ("runtime_mode", "daemon"),
            ("max_ticks", "3"),
            ("tick_interval_ms", "25"),
        ],
    );
}

fn assert_transition_complete_log(captured_logs: &[String], execution_id: &str) {
    let complete_line = find_daemon_log(
        captured_logs,
        "\"event\":\"node.runtime.daemon.execute.complete\"",
        execution_id,
    );
    assert_json_log_fields(
        complete_line,
        &[("runtime_mode", "daemon"), ("executed_ticks", "3")],
    );
    assert_transition_shutdown_fields(complete_line);
}

fn assert_transition_shutdown_fields(complete_line: &str) {
    assert_json_log_fields(
        complete_line,
        &[
            (
                "completion_reason",
                "tick-budget-exhausted;ignored_signals=1",
            ),
            ("shutdown_drain_status", "not-signaled"),
            ("shutdown_snapshot_flush_status", "snapshot-not-requested"),
            ("shutdown_signal_tick", "none"),
            ("shutdown_drain_ticks", "0"),
            ("shutdown_timeout_ticks", "0"),
            ("shutdown_ignored_signals", "1"),
        ],
    );
}

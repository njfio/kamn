#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice() {
    let Some(_context) = live_postgres_validation_context() else {
        return;
    };
    let rendered = daemon_json_report(&["--daemon-max-ticks", "5", "--daemon-tick-interval-ms", "25"]);
    assert_rendered_phase6_reason(&rendered, "m10_phase6_scheduler_cycle_applied");
}

#[test]
fn regression_runtime_daemon_live_postgres_validation_slice_reports_unset_env_gate_reason() {
    // Regression: #5340
    let _lock = hold_log_env_lock();
    assert_gate_resolution(None, None, LIVE_POSTGRES_ENV_UNSET_REASON_CODE, None);
}

#[test]
fn unit_runtime_daemon_live_postgres_validation_slice_prefers_kamn_test_postgres_url() {
    let preferred = "postgres://preferred:5432/kamn_test";
    let fallback = "postgres://fallback:5432/kamn_test";
    let _lock = hold_log_env_lock();
    assert_gate_resolution(
        Some(preferred),
        Some(fallback),
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
        Some(preferred),
    );
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_deferred_path() {
    let Some(_context) = live_postgres_validation_context() else {
        return;
    };
    let rendered = daemon_json_report(&[
        "--daemon-max-ticks",
        "5",
        "--daemon-tick-interval-ms",
        "25",
        "--daemon-shutdown-signal-tick",
        "3",
        "--daemon-shutdown-drain-ticks",
        "2",
        "--daemon-shutdown-timeout-ticks",
        "4",
    ]);
    assert_rendered_phase6_reason(&rendered, "m10_phase6_scheduler_cycle_deferred");
    assert_rendered_deferred_cycles(&rendered, 1);
}

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_env_matrix_contract_is_deterministic() {
    let _lock = hold_log_env_lock();
    let preferred = "postgres://preferred:5432/kamn_test";
    let fallback = "postgres://fallback:5432/kamn_test";
    assert_gate_resolution(None, None, LIVE_POSTGRES_ENV_UNSET_REASON_CODE, None);
    assert_gate_resolution(
        Some(preferred),
        Some(fallback),
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
        Some(preferred),
    );
    assert_gate_resolution(
        Some("   "),
        Some(fallback),
        LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
        Some(fallback),
    );
}

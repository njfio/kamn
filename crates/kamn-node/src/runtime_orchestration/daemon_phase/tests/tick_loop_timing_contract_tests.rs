use super::super::service_api_relay_tick_loop::{
    daemon_tick_remaining_sleep_duration, execute_daemon_service_api_relay_tick_loop,
};
use super::support::lock_daemon_phase_test_guard;
use std::time::Duration;

#[test]
fn unit_daemon_relay_tick_loop_sleeps_between_ticks_when_interval_budget_remains() {
    let _test_lock = lock_daemon_phase_test_guard();
    let _log_lock = crate::logging::lock_log_config_for_tests();
    let runtime_processing = run_tick_loop(3, 50);
    assert_eq!(runtime_processing.executed_ticks, 3);
    assert_eq!(runtime_processing.tick_processing_samples_ms.len(), 3);
    assert_eq!(runtime_processing.tick_sleep_count, 2);
}

#[test]
fn regression_daemon_relay_tick_loop_single_tick_never_sleeps() {
    let _test_lock = lock_daemon_phase_test_guard();
    let _log_lock = crate::logging::lock_log_config_for_tests();
    let runtime_processing = run_tick_loop(1, 50);
    assert_eq!(runtime_processing.tick_sleep_count, 0);
    assert_eq!(runtime_processing.tick_processing_samples_ms.len(), 1);
}

#[test]
fn unit_daemon_tick_remaining_sleep_duration_contract_is_deterministic() {
    let _test_lock = lock_daemon_phase_test_guard();
    let _log_lock = crate::logging::lock_log_config_for_tests();
    assert_remaining_sleep_case(0, 3, 50, 20, Some(30));
    assert_remaining_sleep_case(1, 3, 50, 50, None);
    assert_remaining_sleep_case(1, 3, 50, 60, None);
    assert_remaining_sleep_case(2, 3, 50, 1, None);
}

fn run_tick_loop(
    executed_ticks: u64,
    tick_interval_ms: u64,
) -> crate::daemon_observability::DaemonRuntimeProcessingTelemetry {
    execute_daemon_service_api_relay_tick_loop(
        executed_ticks,
        tick_interval_ms,
        None,
        None,
        "service-api:test:v1",
    )
    .expect("tick loop")
}

fn assert_remaining_sleep_case(
    tick: u64,
    executed_ticks: u64,
    tick_duration_ms: u64,
    elapsed_ms: u64,
    expected_ms: Option<u64>,
) {
    assert_eq!(
        daemon_tick_remaining_sleep_duration(
            tick,
            executed_ticks,
            Duration::from_millis(tick_duration_ms),
            Duration::from_millis(elapsed_ms),
        ),
        expected_ms.map(Duration::from_millis)
    );
}

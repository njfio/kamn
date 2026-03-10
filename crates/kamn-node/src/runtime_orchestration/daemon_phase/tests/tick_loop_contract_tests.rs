use super::super::service_api_relay_tick_loop::{
    execute_daemon_service_api_relay_tick_loop, SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV_FOR_TEST,
};
use super::support::{
    assert_spool_contains, lock_daemon_phase_test_guard, relay_fixture_paths, remove_relay_fixture,
    write_relay_fixture, RelayFixturePaths, TestEnvGuard,
};

#[test]
fn unit_daemon_relay_tick_loop_reports_deterministic_projection_counters() {
    let _test_lock = lock_daemon_phase_test_guard();
    let _log_lock = crate::logging::lock_log_config_for_tests();
    let fixtures = relay_fixture_paths("kamn-node-daemon-phase-projection");
    write_relay_fixture(
        &fixtures,
        "msg-daemon-projection-unit-1",
        r#"{"message":"project"}"#,
        1_700_000_888,
    );
    let _route_guard = TestEnvGuard::set(SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV_FOR_TEST, None);
    let runtime_processing = run_single_tick(&fixtures)
        .expect("daemon relay tick loop should complete without synthetic projection");
    assert_runtime_processing_counts(&runtime_processing, 1, 0, Some(0));
    assert_spool_contains(&fixtures, "msg-daemon-projection-unit-1");
    remove_relay_fixture(&fixtures);
}

#[test]
fn regression_daemon_relay_tick_loop_requeues_failed_cross_node_forward_entries() {
    let _test_lock = lock_daemon_phase_test_guard();
    let _log_lock = crate::logging::lock_log_config_for_tests();
    let fixtures = relay_fixture_paths("kamn-node-daemon-phase-failed-forward");
    write_relay_fixture(
        &fixtures,
        "msg-forward-failure-unit-1",
        r#"{"message":"unreachable"}"#,
        1_700_000_999,
    );
    let _route_guard = TestEnvGuard::set(
        SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV_FOR_TEST,
        Some(r#"{"kamn:did:agent:recipient":"127.0.0.1:9"}"#),
    );
    let _signer_guard = TestEnvGuard::set(
        "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
        Some("658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4"),
    );
    let runtime_processing = run_single_tick(&fixtures)
        .expect("daemon relay tick loop should complete on forward failure");
    assert_runtime_processing_counts(&runtime_processing, 1, 0, Some(1));
    assert_spool_contains(&fixtures, "msg-forward-failure-unit-1");
    remove_relay_fixture(&fixtures);
}

#[test]
fn regression_daemon_relay_tick_loop_without_route_map_preserves_pending_relay_entries() {
    let _test_lock = lock_daemon_phase_test_guard();
    let _log_lock = crate::logging::lock_log_config_for_tests();
    let fixtures = relay_fixture_paths("kamn-node-daemon-phase-no-route");
    write_relay_fixture(
        &fixtures,
        "msg-no-route-unit-1",
        r#"{"message":"pending"}"#,
        1_700_001_001,
    );
    let _route_guard = TestEnvGuard::set(SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV_FOR_TEST, None);
    let runtime_processing = run_single_tick(&fixtures)
        .expect("daemon relay tick loop should complete when route map is not configured");
    assert_runtime_processing_counts(&runtime_processing, 1, 0, None);
    assert_spool_contains(&fixtures, "msg-no-route-unit-1");
    remove_relay_fixture(&fixtures);
}

fn run_single_tick(
    fixtures: &RelayFixturePaths,
) -> Result<crate::daemon_observability::DaemonRuntimeProcessingTelemetry, crate::ConfigError> {
    execute_daemon_service_api_relay_tick_loop(
        1,
        1,
        Some(fixtures.state_file.to_string_lossy().as_ref()),
        Some(fixtures.relay_spool_file.to_string_lossy().as_ref()),
        "service-api:kamn-devnet:v0.1.0",
    )
}

fn assert_runtime_processing_counts(
    runtime_processing: &crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    drained_count: u64,
    projected_state_count: u64,
    processing_error_count: Option<u64>,
) {
    assert_eq!(runtime_processing.relay_drained_count, drained_count);
    assert_eq!(
        runtime_processing.relay_projected_state_count,
        projected_state_count
    );
    if let Some(processing_error_count) = processing_error_count {
        assert_eq!(
            runtime_processing.processing_error_count,
            processing_error_count
        );
    }
}

use kamn_core::runtime::{WatchdogAnomalyError, WatchdogAnomalyKind, WatchdogAnomalySeverity};
use kamn_core::{
    simulate_daemon_network_fault, DeterministicNetworkFaultSimulator, NetworkFaultSimulationError,
    NetworkFaultSimulationInput, PeerLifecycleState, RuntimeBackpressureAction,
};

fn make_input(
    sample_id: &str,
    peer_id: &str,
    expected_deliveries: u32,
    delivered_deliveries: u32,
    queue_capacity: usize,
) -> Result<NetworkFaultSimulationInput, NetworkFaultSimulationError> {
    NetworkFaultSimulationInput::new(
        sample_id,
        peer_id,
        expected_deliveries,
        delivered_deliveries,
        4,
        4,
        30,
        0,
        queue_capacity,
        8,
    )
}

fn valid_input() -> NetworkFaultSimulationInput {
    make_input("sample-a", "peer-1", 10, 10, 10).expect("valid network fault input")
}

fn assert_invalid_input(
    sample_id: &str,
    peer_id: &str,
    expected_deliveries: u32,
    delivered_deliveries: u32,
    queue_capacity: usize,
    expected_error: NetworkFaultSimulationError,
) {
    let error = make_input(
        sample_id,
        peer_id,
        expected_deliveries,
        delivered_deliveries,
        queue_capacity,
    )
    .expect_err("input must fail closed");
    assert_eq!(error, expected_error);
}

#[test]
fn integration_runtime_network_fault_valid_simulation_returns_expected_report() {
    let simulator = DeterministicNetworkFaultSimulator::default();

    let report = simulator
        .simulate(valid_input())
        .expect("simulation should succeed");

    assert_eq!(report.sample_id, "sample-a");
    assert_eq!(report.final_lifecycle_state, PeerLifecycleState::Active);
    assert_eq!(report.queue_capacity, 10);
    assert_eq!(report.queued_events, 8);
    assert_eq!(report.queue_overflow_attempts, 0);
    assert_eq!(
        report.backpressure_last_action,
        RuntimeBackpressureAction::SlowProducer
    );
    assert_eq!(
        report.backpressure_last_reason_code,
        "runtime_backpressure_slow_producer"
    );
    assert_eq!(report.backpressure_rejected_events, 0);
    assert_eq!(report.backpressure_purged_events, 0);
    assert_eq!(report.backpressure_slow_events, 1);
    assert_eq!(report.watchdog_kind, WatchdogAnomalyKind::Nominal);
    assert_eq!(report.watchdog_severity, WatchdogAnomalySeverity::Info);
    assert_eq!(report.watchdog_delivery_ratio_per_mille, 1000);
    assert_eq!(report.watchdog_liveness_ratio_per_mille, 1000);
}

#[test]
fn integration_runtime_network_fault_invalid_inputs_fail_closed_with_reason_codes() {
    assert_invalid_input(
        "",
        "peer-1",
        10,
        10,
        10,
        NetworkFaultSimulationError::InvalidSampleId,
    );
    assert_invalid_input(
        "sample-a",
        "",
        10,
        10,
        10,
        NetworkFaultSimulationError::InvalidPeerId,
    );
    assert_invalid_input(
        "sample-a",
        "peer-1",
        10,
        10,
        0,
        NetworkFaultSimulationError::InvalidQueueCapacity { capacity: 0 },
    );
    assert_invalid_input(
        "sample-a",
        "peer-1",
        0,
        0,
        10,
        NetworkFaultSimulationError::Watchdog(WatchdogAnomalyError::InvalidExpectedDeliveries {
            expected_deliveries: 0,
        }),
    );
}

#[test]
fn integration_runtime_network_fault_daemon_helper_matches_direct_simulation() {
    let simulator = DeterministicNetworkFaultSimulator::default();
    let input = valid_input();

    let direct = simulator
        .simulate(input.clone())
        .expect("direct simulation should succeed");
    let daemon = simulate_daemon_network_fault(&simulator, input)
        .expect("daemon helper simulation should succeed");

    assert_eq!(daemon, direct);
}

use kamn_core::{
    DeterministicNetworkFaultSimulator, NetworkFaultSimulationError, NetworkFaultSimulationInput,
    PeerLifecycleState, RuntimeBackpressureAction, simulate_daemon_network_fault,
};
use kamn_core::runtime::{
    WatchdogAnomalyError, WatchdogAnomalyKind, WatchdogAnomalySeverity,
};

fn valid_input() -> NetworkFaultSimulationInput {
    NetworkFaultSimulationInput::new("sample-a", "peer-1", 10, 10, 4, 4, 30, 0, 10, 8)
        .expect("valid network fault input")
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
    assert_eq!(report.backpressure_last_action, RuntimeBackpressureAction::SlowProducer);
    assert_eq!(report.backpressure_last_reason_code, "runtime_backpressure_slow_producer");
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
    let error = NetworkFaultSimulationInput::new("", "peer-1", 10, 10, 4, 4, 30, 0, 10, 1)
        .expect_err("empty sample id must fail closed");
    assert_eq!(error, NetworkFaultSimulationError::InvalidSampleId);

    let error = NetworkFaultSimulationInput::new("sample-a", "", 10, 10, 4, 4, 30, 0, 10, 1)
        .expect_err("empty peer id must fail closed");
    assert_eq!(error, NetworkFaultSimulationError::InvalidPeerId);

    let error = NetworkFaultSimulationInput::new("sample-a", "peer-1", 10, 10, 4, 4, 30, 0, 0, 1)
        .expect_err("zero queue capacity must fail closed");
    assert_eq!(
        error,
        NetworkFaultSimulationError::InvalidQueueCapacity { capacity: 0 }
    );

    let error = NetworkFaultSimulationInput::new("sample-a", "peer-1", 0, 0, 4, 4, 30, 0, 10, 1)
        .expect_err("invalid watchdog input must fail closed");
    assert_eq!(
        error,
        NetworkFaultSimulationError::Watchdog(WatchdogAnomalyError::InvalidExpectedDeliveries {
            expected_deliveries: 0,
        })
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

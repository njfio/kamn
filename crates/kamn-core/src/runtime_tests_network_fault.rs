use super::super::{
    simulate_daemon_network_fault, DeterministicNetworkFaultSimulator, NetworkFaultSimulationError,
    NetworkFaultSimulationInput, PeerLifecycleState, RuntimeBackpressureAction,
    WatchdogAnomalyKind, WatchdogAnomalySeverity,
};
use std::time::Instant;

#[test]
fn unit_network_fault_simulation_rejects_zero_queue_capacity() {
    let input = NetworkFaultSimulationInput::new(
        "fault-sample-invalid",
        "peer-sim-a",
        100,
        99,
        6,
        6,
        30,
        1,
        0,
        2,
    );
    assert_eq!(
        input,
        Err(NetworkFaultSimulationError::InvalidQueueCapacity { capacity: 0 })
    );
}

#[test]
fn functional_network_fault_simulation_classifies_targeted_packet_loss_as_critical() {
    let simulator = DeterministicNetworkFaultSimulator::default();
    let input = NetworkFaultSimulationInput::new(
        "fault-sample-censorship",
        "peer-sim-a",
        100,
        45,
        8,
        8,
        60,
        3,
        8,
        8,
    )
    .expect("valid simulation input");
    let report = simulator
        .simulate(input)
        .expect("simulation should classify targeted packet loss");

    assert_eq!(report.watchdog_kind, WatchdogAnomalyKind::CensorshipSignal);
    assert_eq!(report.watchdog_severity, WatchdogAnomalySeverity::Critical);
    assert_eq!(report.final_lifecycle_state, PeerLifecycleState::Active);
    assert_eq!(report.queue_overflow_attempts, 0);
    assert_eq!(
        report.backpressure_last_action,
        RuntimeBackpressureAction::SlowProducer
    );
    assert_eq!(
        report.backpressure_last_reason_code,
        "runtime_backpressure_slow_producer"
    );
}

#[test]
fn integration_daemon_network_fault_simulation_reports_overflow_and_degradation() {
    let simulator = DeterministicNetworkFaultSimulator::default();
    let input = NetworkFaultSimulationInput::new(
        "fault-sample-overflow",
        "peer-sim-b",
        120,
        110,
        6,
        4,
        30,
        1,
        2,
        5,
    )
    .expect("valid simulation input");
    let report = simulate_daemon_network_fault(&simulator, input).expect("simulation should pass");

    assert_eq!(report.final_lifecycle_state, PeerLifecycleState::Degraded);
    assert_eq!(report.queue_overflow_attempts, 3);
    assert_eq!(
        report.backpressure_last_action,
        RuntimeBackpressureAction::RejectNewEnqueue
    );
    assert_eq!(
        report.backpressure_last_reason_code,
        "runtime_backpressure_reject_new_enqueue"
    );
    assert_eq!(report.backpressure_rejected_events, 3);
    assert_eq!(report.backpressure_purged_events, 0);
    assert_eq!(
        report.watchdog_kind,
        WatchdogAnomalyKind::LivenessDegradation
    );
}

#[test]
fn integration_network_fault_simulation_purges_stale_disconnected_peer_queue() {
    let simulator = DeterministicNetworkFaultSimulator::default();
    let input = NetworkFaultSimulationInput::new(
        "fault-sample-stale-peer",
        "peer-sim-stale",
        120,
        118,
        6,
        0,
        30,
        1,
        4,
        4,
    )
    .expect("valid simulation input");
    let report = simulate_daemon_network_fault(&simulator, input).expect("simulation should pass");

    assert_eq!(
        report.final_lifecycle_state,
        PeerLifecycleState::Disconnected
    );
    assert_eq!(
        report.backpressure_last_action,
        RuntimeBackpressureAction::PurgeStalePeerQueue
    );
    assert_eq!(
        report.backpressure_last_reason_code,
        "runtime_backpressure_purge_stale_peer_queue"
    );
    assert!(report.backpressure_purged_events > 0);
}

#[test]
fn regression_network_fault_simulation_keeps_censorship_critical_boundary() {
    // Regression: #618
    let simulator = DeterministicNetworkFaultSimulator::default();
    let input = NetworkFaultSimulationInput::new(
        "fault-sample-regression",
        "peer-sim-c",
        200,
        100,
        12,
        12,
        60,
        2,
        4,
        4,
    )
    .expect("valid simulation input");
    let report = simulator
        .simulate(input)
        .expect("simulation should classify censorship boundary");

    assert_eq!(report.watchdog_kind, WatchdogAnomalyKind::CensorshipSignal);
    assert_eq!(report.watchdog_severity, WatchdogAnomalySeverity::Critical);
}

#[test]
fn performance_network_fault_simulation_pr_lane_stays_within_budget() {
    let simulator = DeterministicNetworkFaultSimulator::default();
    let start = Instant::now();
    for sample_index in 0..256 {
        let input = NetworkFaultSimulationInput::new(
            &format!("fault-sample-perf-{sample_index}"),
            "peer-sim-perf",
            100,
            98,
            16,
            16,
            30,
            1,
            8,
            8,
        )
        .expect("valid simulation input");
        assert!(simulator.simulate(input).is_ok());
    }
    let elapsed_millis = start.elapsed().as_millis();
    assert!(
        elapsed_millis < 250,
        "network fault simulation PR lane exceeded budget: {elapsed_millis}ms"
    );
}

#[test]
#[ignore = "scheduled chaos lane"]
fn performance_network_fault_simulation_chaos_lane_stress() {
    let simulator = DeterministicNetworkFaultSimulator::default();
    for sample_index in 0..5000 {
        let targeted_peer_count = if sample_index % 4 == 0 { 3 } else { 1 };
        let delivered = if targeted_peer_count == 3 { 45 } else { 99 };
        let healthy_peers = if sample_index % 3 == 0 { 8 } else { 10 };
        let input = NetworkFaultSimulationInput::new(
            &format!("fault-sample-chaos-{sample_index}"),
            "peer-sim-chaos",
            100,
            delivered,
            10,
            healthy_peers,
            30,
            targeted_peer_count,
            16,
            24,
        )
        .expect("valid simulation input");
        assert!(simulator.simulate(input).is_ok());
    }
}

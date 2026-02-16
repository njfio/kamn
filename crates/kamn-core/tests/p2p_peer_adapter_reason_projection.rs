use kamn_core::{
    deterministic_multi_process_peer_validation_hooks, peer_adapter_reason_taxonomy_version,
    project_live_transport_reconnect_reason, project_peer_adapter_error_reason,
    LiveTransportFaultClass, LiveTransportReconnectPolicy, P2pTransportError,
    PeerAdapterReasonClass,
};
use std::time::{Duration, Instant};

#[test]
fn unit_retry_timeout_reason_projection_is_deterministic() {
    let policy = LiveTransportReconnectPolicy::new(1, 8, 4)
        .expect("policy should build with deterministic bounds");
    let decision = policy.evaluate(LiveTransportFaultClass::DialTimeout, 1);
    let projection = project_live_transport_reconnect_reason(&decision);

    assert_eq!(
        projection.reason_code(),
        "p2p_live_reconnect_retry_dial_timeout"
    );
    assert_eq!(
        projection.reason_class(),
        PeerAdapterReasonClass::RetryTimeout
    );
    assert_eq!(projection.source_marker(), "p2p_live_reconnect_policy");
}

#[test]
fn functional_reason_projection_maps_fail_closed_transport_error() {
    let error = P2pTransportError::LiveSocketSendFailed;
    let projection = project_peer_adapter_error_reason(&error);

    assert_eq!(
        projection.reason_code(),
        "p2p_transport_live_socket_send_failed"
    );
    assert_eq!(
        projection.reason_class(),
        PeerAdapterReasonClass::FailClosed
    );
    assert_eq!(
        projection.source_marker(),
        "p2p_peer_adapter_error_projection"
    );
}

#[test]
fn unit_multi_process_validation_hooks_are_stable_and_ordered() {
    let hooks = deterministic_multi_process_peer_validation_hooks();
    assert_eq!(
        hooks.iter().map(|hook| hook.hook_id()).collect::<Vec<_>>(),
        vec![
            "peer_adapter_process_isolated_validation",
            "peer_adapter_process_isolated_policy",
            "peer_adapter_process_isolated_contract_lane",
        ]
    );
    assert!(hooks
        .iter()
        .all(|hook| hook.reason_taxonomy_version() == peer_adapter_reason_taxonomy_version()));
}

#[test]
fn integration_projection_and_hooks_reason_output_integrity_contract() {
    let policy = LiveTransportReconnectPolicy::new(1, 8, 4)
        .expect("policy should build with deterministic bounds");
    let timeout_projection = project_live_transport_reconnect_reason(
        &policy.evaluate(LiveTransportFaultClass::DialTimeout, 1),
    );
    let budget_projection = project_live_transport_reconnect_reason(
        &policy.evaluate(LiveTransportFaultClass::DialTimeout, 4),
    );
    let fail_closed_projection =
        project_peer_adapter_error_reason(&P2pTransportError::LiveSocketSendFailed);

    assert_eq!(
        timeout_projection.reason_class(),
        PeerAdapterReasonClass::RetryTimeout
    );
    assert_eq!(
        budget_projection.reason_class(),
        PeerAdapterReasonClass::RetryBudgetExhausted
    );
    assert_eq!(
        fail_closed_projection.reason_class(),
        PeerAdapterReasonClass::FailClosed
    );

    let hooks = deterministic_multi_process_peer_validation_hooks();
    assert!(hooks.iter().any(|hook| hook
        .command()
        .contains("validate_libp2p_convergence_process_isolated_live.sh")));
    assert!(hooks.iter().any(|hook| hook
        .command()
        .contains("check_libp2p_convergence_process_isolated_live_policy.sh")));
    assert!(hooks.iter().any(|hook| hook.local_heavy_only()));
}

#[test]
fn regression_retry_timeout_budget_boundary_projection_stays_stable() {
    // Regression: #4320
    let policy = LiveTransportReconnectPolicy::new(1, 8, 4)
        .expect("policy should build with deterministic bounds");

    let pre_budget_projection = project_live_transport_reconnect_reason(
        &policy.evaluate(LiveTransportFaultClass::DialTimeout, 3),
    );
    let budget_projection = project_live_transport_reconnect_reason(
        &policy.evaluate(LiveTransportFaultClass::DialTimeout, 4),
    );

    assert_eq!(
        pre_budget_projection.reason_code(),
        "p2p_live_reconnect_retry_dial_timeout"
    );
    assert_eq!(
        pre_budget_projection.reason_class(),
        PeerAdapterReasonClass::RetryTimeout
    );
    assert_eq!(
        budget_projection.reason_code(),
        "p2p_live_reconnect_retry_budget_exhausted"
    );
    assert_eq!(
        budget_projection.reason_class(),
        PeerAdapterReasonClass::RetryBudgetExhausted
    );
}

#[test]
fn performance_reason_projection_loop_stays_within_local_budget() {
    let policy = LiveTransportReconnectPolicy::new(1, 16, 8)
        .expect("policy should build with deterministic bounds");

    let started = Instant::now();
    for idx in 0..50_000_u32 {
        let attempt = (idx % 7 + 1) as u16;
        let fault = match idx % 3 {
            0 => LiveTransportFaultClass::DialTimeout,
            1 => LiveTransportFaultClass::DiscoveryUnavailable,
            _ => LiveTransportFaultClass::StreamChurn,
        };
        let decision = policy.evaluate(fault, attempt);
        let projection = project_live_transport_reconnect_reason(&decision);
        assert!(
            projection.reason_code().starts_with("p2p_live_reconnect_"),
            "unexpected projected reason code prefix"
        );
    }
    assert!(
        started.elapsed() <= Duration::from_secs(2),
        "reason projection performance exceeded local budget"
    );
}

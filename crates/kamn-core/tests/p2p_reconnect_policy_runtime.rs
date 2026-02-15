use kamn_core::{
    LiveTransportFaultClass, LiveTransportReconnectDecision, LiveTransportReconnectPolicy,
};

#[test]
fn unit_reconnect_policy_backoff_caps_deterministically() {
    let policy = LiveTransportReconnectPolicy::new(1, 4, 3)
        .expect("policy should build with deterministic bounds");

    assert_eq!(
        policy.evaluate(LiveTransportFaultClass::DialTimeout, 1),
        LiveTransportReconnectDecision::Retry {
            backoff_ticks: 1,
            reason_code: "p2p_live_reconnect_retry_dial_timeout",
        }
    );
    assert_eq!(
        policy.evaluate(LiveTransportFaultClass::DialTimeout, 2),
        LiveTransportReconnectDecision::Retry {
            backoff_ticks: 2,
            reason_code: "p2p_live_reconnect_retry_dial_timeout",
        }
    );
    assert_eq!(
        policy.evaluate(LiveTransportFaultClass::DialTimeout, 3),
        LiveTransportReconnectDecision::FailClosed {
            reason_code: "p2p_live_reconnect_retry_budget_exhausted",
        }
    );
}

#[test]
fn functional_reconnect_policy_protocol_violation_fails_closed() {
    let policy = LiveTransportReconnectPolicy::new(2, 8, 4)
        .expect("policy should build with deterministic bounds");

    assert_eq!(
        policy.evaluate(LiveTransportFaultClass::ProtocolViolation, 1),
        LiveTransportReconnectDecision::FailClosed {
            reason_code: "p2p_live_reconnect_protocol_violation",
        }
    );
}

#[test]
fn integration_reconnect_policy_fault_sequence_emits_stable_decisions() {
    let policy = LiveTransportReconnectPolicy::new(1, 4, 4)
        .expect("policy should build with deterministic bounds");
    let faults = vec![
        (LiveTransportFaultClass::DialTimeout, 1),
        (LiveTransportFaultClass::DiscoveryUnavailable, 2),
        (LiveTransportFaultClass::ProtocolViolation, 3),
    ];

    let decisions = faults
        .into_iter()
        .map(|(fault, attempt)| policy.evaluate(fault, attempt))
        .collect::<Vec<_>>();
    assert_eq!(
        decisions,
        vec![
            LiveTransportReconnectDecision::Retry {
                backoff_ticks: 1,
                reason_code: "p2p_live_reconnect_retry_dial_timeout",
            },
            LiveTransportReconnectDecision::Retry {
                backoff_ticks: 2,
                reason_code: "p2p_live_reconnect_retry_discovery_unavailable",
            },
            LiveTransportReconnectDecision::FailClosed {
                reason_code: "p2p_live_reconnect_protocol_violation",
            },
        ]
    );
}

#[test]
fn regression_reconnect_policy_exhaustion_reason_code_is_stable() {
    // Regression: #3576
    let policy = LiveTransportReconnectPolicy::new(1, 8, 2)
        .expect("policy should build with deterministic bounds");
    let first = policy.evaluate(LiveTransportFaultClass::DiscoveryUnavailable, 2);
    let second = policy.evaluate(LiveTransportFaultClass::DiscoveryUnavailable, 2);

    assert_eq!(
        first,
        LiveTransportReconnectDecision::FailClosed {
            reason_code: "p2p_live_reconnect_retry_budget_exhausted",
        }
    );
    assert_eq!(first, second);
}

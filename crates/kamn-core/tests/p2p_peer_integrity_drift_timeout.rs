use kamn_core::{
    InMemoryPeerLifecycleTransport, LiveTransportFaultClass, LiveTransportReconnectDecision,
    LiveTransportReconnectPolicy, NodeRole, P2pTransportError, PeerDiscoveryRecord,
    PeerGossipFrame, PeerLifecycleTransport,
};
use std::time::{Duration, Instant};

fn advertise_peer(transport: &InMemoryPeerLifecycleTransport, peer_id: &str) {
    transport
        .advertise(
            PeerDiscoveryRecord::new(peer_id, NodeRole::Processor, vec!["messages".to_owned()])
                .expect("peer record should build"),
        )
        .expect("peer should advertise");
}

#[test]
fn unit_peer_transport_rejects_sender_integrity_drift_with_reason_code() {
    let transport = InMemoryPeerLifecycleTransport::default();
    advertise_peer(&transport, "peer-recipient");

    let frame = PeerGossipFrame::new(
        "messages",
        "peer-integrity-drift",
        "peer-recipient",
        "tx-integrity-drift",
    )
    .expect("frame should build");
    let error = transport
        .send(frame)
        .expect_err("unknown sender drift must fail closed");

    assert_eq!(
        error,
        P2pTransportError::UnknownSenderPeer("peer-integrity-drift".to_owned())
    );
    assert_eq!(error.reason_code(), "p2p_transport_unknown_sender_peer");
}

#[test]
fn functional_retry_timeout_fault_class_emits_timeout_reason_code() {
    let policy = LiveTransportReconnectPolicy::new(1, 8, 4)
        .expect("policy should build with deterministic bounds");

    assert_eq!(
        policy.evaluate(LiveTransportFaultClass::DialTimeout, 1),
        LiveTransportReconnectDecision::Retry {
            backoff_ticks: 1,
            reason_code: "p2p_live_reconnect_retry_dial_timeout",
        }
    );
}

#[test]
fn integration_retry_fault_matrix_classification_is_stable() {
    let policy = LiveTransportReconnectPolicy::new(1, 8, 6)
        .expect("policy should build with deterministic bounds");
    let faults = [
        (LiveTransportFaultClass::DialTimeout, 1),
        (LiveTransportFaultClass::DiscoveryUnavailable, 2),
        (LiveTransportFaultClass::StreamChurn, 3),
        (LiveTransportFaultClass::DialTimeout, 4),
    ];

    let first = faults
        .iter()
        .copied()
        .map(|(fault, attempt)| policy.evaluate(fault, attempt))
        .collect::<Vec<_>>();
    let second = faults
        .iter()
        .copied()
        .map(|(fault, attempt)| policy.evaluate(fault, attempt))
        .collect::<Vec<_>>();

    assert_eq!(
        first,
        vec![
            LiveTransportReconnectDecision::Retry {
                backoff_ticks: 1,
                reason_code: "p2p_live_reconnect_retry_dial_timeout",
            },
            LiveTransportReconnectDecision::Retry {
                backoff_ticks: 2,
                reason_code: "p2p_live_reconnect_retry_discovery_unavailable",
            },
            LiveTransportReconnectDecision::Retry {
                backoff_ticks: 4,
                reason_code: "p2p_live_reconnect_retry_stream_churn",
            },
            LiveTransportReconnectDecision::Retry {
                backoff_ticks: 8,
                reason_code: "p2p_live_reconnect_retry_dial_timeout",
            },
        ]
    );
    assert_eq!(first, second);
}

#[test]
fn regression_retry_timeout_pre_budget_attempt_remains_timeout_classified() {
    // Regression: #4319
    let policy = LiveTransportReconnectPolicy::new(1, 8, 4)
        .expect("policy should build with deterministic bounds");

    assert_eq!(
        policy.evaluate(LiveTransportFaultClass::DialTimeout, 0),
        LiveTransportReconnectDecision::Retry {
            backoff_ticks: 1,
            reason_code: "p2p_live_reconnect_retry_dial_timeout",
        }
    );
    assert_eq!(
        policy.evaluate(LiveTransportFaultClass::DialTimeout, 3),
        LiveTransportReconnectDecision::Retry {
            backoff_ticks: 4,
            reason_code: "p2p_live_reconnect_retry_dial_timeout",
        }
    );
    assert_eq!(
        policy.evaluate(LiveTransportFaultClass::DialTimeout, 4),
        LiveTransportReconnectDecision::FailClosed {
            reason_code: "p2p_live_reconnect_retry_budget_exhausted",
        }
    );
}

#[test]
fn performance_retry_timeout_classification_stays_within_local_budget() {
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
        let expected_reason = match fault {
            LiveTransportFaultClass::DialTimeout => "p2p_live_reconnect_retry_dial_timeout",
            LiveTransportFaultClass::DiscoveryUnavailable => {
                "p2p_live_reconnect_retry_discovery_unavailable"
            }
            LiveTransportFaultClass::StreamChurn => "p2p_live_reconnect_retry_stream_churn",
            LiveTransportFaultClass::ProtocolViolation => "p2p_live_reconnect_protocol_violation",
        };

        match policy.evaluate(fault, attempt) {
            LiveTransportReconnectDecision::Retry { reason_code, .. } => {
                assert_eq!(reason_code, expected_reason);
            }
            decision => {
                panic!("expected retry decision for bounded timeout class path: {decision:?}")
            }
        }
    }

    assert!(
        started.elapsed() <= Duration::from_secs(2),
        "retry-timeout classification loop exceeded local budget"
    );
}

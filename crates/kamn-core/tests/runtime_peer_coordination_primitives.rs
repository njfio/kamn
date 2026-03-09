use kamn_core::{
    BoundedRuntimeQueue, PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState,
    RuntimeLifecycleError, RuntimeQueueError,
};

fn assert_transition(
    peer: &mut PeerLifecycle,
    event: PeerLifecycleEvent,
    expected: PeerLifecycleState,
) {
    assert_eq!(peer.transition(event), Ok(expected));
}

#[test]
fn integration_runtime_peer_lifecycle_valid_sequence_reaches_expected_states() {
    let mut peer = PeerLifecycle::new("peer-1").expect("peer should construct");

    assert_eq!(peer.state(), PeerLifecycleState::Disconnected);
    assert_transition(
        &mut peer,
        PeerLifecycleEvent::StartConnect,
        PeerLifecycleState::Connecting,
    );
    assert_transition(
        &mut peer,
        PeerLifecycleEvent::HandshakeSucceeded,
        PeerLifecycleState::Active,
    );
    assert_transition(
        &mut peer,
        PeerLifecycleEvent::HeartbeatMissed,
        PeerLifecycleState::Degraded,
    );
    assert_transition(
        &mut peer,
        PeerLifecycleEvent::Disconnect,
        PeerLifecycleState::Disconnected,
    );
}

#[test]
fn integration_runtime_peer_lifecycle_invalid_transition_fails_closed_with_reason_code() {
    let mut peer = PeerLifecycle::new("peer-1").expect("peer should construct");

    let error = peer
        .transition(PeerLifecycleEvent::HeartbeatMissed)
        .expect_err("invalid transition must fail closed");
    assert_eq!(error.reason_code(), "runtime_peer_transition_invalid");
    assert_eq!(
        error,
        RuntimeLifecycleError::InvalidTransition {
            from: PeerLifecycleState::Disconnected,
            event: PeerLifecycleEvent::HeartbeatMissed,
        }
    );
}

#[test]
fn integration_runtime_queue_preserves_fifo_order() {
    let mut queue = BoundedRuntimeQueue::new(2).expect("queue should construct");

    queue
        .enqueue("first")
        .expect("first enqueue should succeed");
    queue
        .enqueue("second")
        .expect("second enqueue should succeed");

    assert_eq!(queue.len(), 2);
    assert_eq!(queue.dequeue(), Some("first"));
    assert_eq!(queue.dequeue(), Some("second"));
    assert!(queue.is_empty());
}

#[test]
fn integration_runtime_queue_invalid_capacity_and_overflow_fail_closed() {
    let error = BoundedRuntimeQueue::<u8>::new(0).expect_err("zero capacity must fail closed");
    assert_eq!(error.reason_code(), "runtime_queue_invalid_capacity");
    assert_eq!(error, RuntimeQueueError::InvalidCapacity { capacity: 0 });

    let mut queue = BoundedRuntimeQueue::new(1).expect("queue should construct");
    queue.enqueue(1).expect("first enqueue should succeed");
    let error = queue.enqueue(2).expect_err("overflow must fail closed");
    assert_eq!(error.reason_code(), "runtime_queue_overflow");
    assert_eq!(
        error,
        RuntimeQueueError::Overflow {
            capacity: 1,
            attempted_len: 2,
        }
    );
}

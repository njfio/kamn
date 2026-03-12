use super::super::*;

#[test]
fn functional_peer_lifecycle_allows_connect_heartbeat_recover_disconnect_flow() {
    let mut lifecycle = PeerLifecycle::new("peer-1").expect("valid peer id");
    assert_eq!(lifecycle.peer_id(), "peer-1");
    assert_eq!(lifecycle.state(), PeerLifecycleState::Disconnected);
    assert!(lifecycle.transition(PeerLifecycleEvent::StartConnect).is_ok());
    assert!(lifecycle
        .transition(PeerLifecycleEvent::HandshakeSucceeded)
        .is_ok());
    assert_eq!(lifecycle.state(), PeerLifecycleState::Active);
    assert!(lifecycle
        .transition(PeerLifecycleEvent::HeartbeatMissed)
        .is_ok());
    assert_eq!(lifecycle.state(), PeerLifecycleState::Degraded);
    assert!(lifecycle
        .transition(PeerLifecycleEvent::HeartbeatRestored)
        .is_ok());
    assert_eq!(lifecycle.state(), PeerLifecycleState::Active);
    assert!(lifecycle.transition(PeerLifecycleEvent::Disconnect).is_ok());
    assert_eq!(lifecycle.state(), PeerLifecycleState::Disconnected);
}

#[test]
fn integration_bounded_runtime_queue_preserves_fifo_until_capacity() {
    let mut queue = BoundedRuntimeQueue::new(2).expect("queue should build");
    assert_eq!(queue.capacity(), 2);
    assert!(queue.is_empty());
    assert!(queue.enqueue("evt-1".to_owned()).is_ok());
    assert!(queue.enqueue("evt-2".to_owned()).is_ok());
    assert_eq!(queue.len(), 2);
    assert_eq!(queue.dequeue(), Some("evt-1".to_owned()));
    assert_eq!(queue.dequeue(), Some("evt-2".to_owned()));
    assert!(queue.dequeue().is_none());
}

#[test]
fn unit_rejects_invalid_peer_lifecycle_transition() {
    let mut lifecycle = PeerLifecycle::new("peer-1").expect("valid peer id");
    let error = lifecycle
        .transition(PeerLifecycleEvent::HandshakeSucceeded)
        .expect_err("handshake cannot complete before connect");
    assert_eq!(
        error,
        RuntimeLifecycleError::InvalidTransition {
            from: PeerLifecycleState::Disconnected,
            event: PeerLifecycleEvent::HandshakeSucceeded
        }
    );
}

#[test]
fn regression_rejoin_without_disconnect_is_rejected() {
    let mut lifecycle = PeerLifecycle::new("peer-1").expect("valid peer id");
    assert!(lifecycle.transition(PeerLifecycleEvent::StartConnect).is_ok());
    assert!(lifecycle
        .transition(PeerLifecycleEvent::HandshakeSucceeded)
        .is_ok());
    let error = lifecycle
        .transition(PeerLifecycleEvent::Rejoin)
        .expect_err("rejoin should require disconnected state");
    assert_eq!(
        error,
        RuntimeLifecycleError::InvalidTransition {
            from: PeerLifecycleState::Active,
            event: PeerLifecycleEvent::Rejoin
        }
    );
}

#[test]
fn regression_queue_overflow_rejects_new_event() {
    let mut queue = BoundedRuntimeQueue::new(1).expect("queue should build");
    assert!(queue.enqueue("evt-1".to_owned()).is_ok());
    let error = queue
        .enqueue("evt-2".to_owned())
        .expect_err("second enqueue must overflow");
    assert_eq!(
        error,
        RuntimeQueueError::Overflow {
            capacity: 1,
            attempted_len: 2
        }
    );
}

#[test]
fn unit_rejects_empty_peer_id() {
    assert_eq!(
        PeerLifecycle::new(""),
        Err(RuntimeLifecycleError::InvalidPeerId)
    );
}

#[test]
fn unit_rejects_zero_queue_capacity() {
    assert_eq!(
        BoundedRuntimeQueue::<String>::new(0),
        Err(RuntimeQueueError::InvalidCapacity { capacity: 0 })
    );
}

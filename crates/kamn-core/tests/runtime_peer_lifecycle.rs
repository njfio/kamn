use kamn_core::{PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, RuntimeLifecycleError};

const PEER_EVENTS: [PeerLifecycleEvent; 6] = [
    PeerLifecycleEvent::StartConnect,
    PeerLifecycleEvent::HandshakeSucceeded,
    PeerLifecycleEvent::HeartbeatMissed,
    PeerLifecycleEvent::HeartbeatRestored,
    PeerLifecycleEvent::Disconnect,
    PeerLifecycleEvent::Rejoin,
];

fn expected_next_state(
    from: PeerLifecycleState,
    event: PeerLifecycleEvent,
) -> Option<PeerLifecycleState> {
    match (from, event) {
        (PeerLifecycleState::Disconnected, PeerLifecycleEvent::StartConnect)
        | (PeerLifecycleState::Disconnected, PeerLifecycleEvent::Rejoin) => {
            Some(PeerLifecycleState::Connecting)
        }
        (PeerLifecycleState::Connecting, PeerLifecycleEvent::HandshakeSucceeded) => {
            Some(PeerLifecycleState::Active)
        }
        (PeerLifecycleState::Connecting, PeerLifecycleEvent::Disconnect)
        | (PeerLifecycleState::Active, PeerLifecycleEvent::Disconnect)
        | (PeerLifecycleState::Degraded, PeerLifecycleEvent::Disconnect) => {
            Some(PeerLifecycleState::Disconnected)
        }
        (PeerLifecycleState::Active, PeerLifecycleEvent::HeartbeatMissed) => {
            Some(PeerLifecycleState::Degraded)
        }
        (PeerLifecycleState::Degraded, PeerLifecycleEvent::HeartbeatRestored) => {
            Some(PeerLifecycleState::Active)
        }
        _ => None,
    }
}

fn for_each_peer_event_sequence(max_len: usize, mut f: impl FnMut(&[PeerLifecycleEvent])) {
    fn recurse(
        target_len: usize,
        current: &mut Vec<PeerLifecycleEvent>,
        f: &mut impl FnMut(&[PeerLifecycleEvent]),
    ) {
        if current.len() == target_len {
            f(current.as_slice());
            return;
        }

        for event in PEER_EVENTS {
            current.push(event);
            recurse(target_len, current, f);
            current.pop();
        }
    }

    let mut current = Vec::new();
    for len in 1..=max_len {
        recurse(len, &mut current, &mut f);
    }
}

#[test]
fn peer_lifecycle_property_generated_event_sequences_match_transition_contract() {
    // Bound sequence depth for fast PR feedback while still exploring broad transition space.
    for_each_peer_event_sequence(5, |sequence| {
        let mut lifecycle =
            PeerLifecycle::new("peer-property").expect("peer lifecycle should initialize");

        for event in sequence {
            let before_state = lifecycle.state();
            let expected = expected_next_state(before_state, *event);
            match (expected, lifecycle.transition(*event)) {
                (Some(next_state), Ok(applied)) => {
                    assert_eq!(applied, next_state);
                    assert_eq!(lifecycle.state(), next_state);
                }
                (
                    None,
                    Err(RuntimeLifecycleError::InvalidTransition {
                        from,
                        event: rejected,
                    }),
                ) => {
                    assert_eq!(from, before_state);
                    assert_eq!(rejected, *event);
                    assert_eq!(lifecycle.state(), before_state);
                }
                (Some(_), Err(error)) => panic!(
                    "expected successful transition from {before_state:?} via {event:?}, got \
                     error: {error:?}"
                ),
                (None, Ok(applied)) => panic!(
                    "expected invalid transition from {before_state:?} via {event:?}, got state \
                     {applied:?}"
                ),
                (None, Err(RuntimeLifecycleError::InvalidPeerId)) => {
                    panic!("peer id is fixed and valid in this property lane");
                }
            }
        }
    });
}

#[test]
fn peer_lifecycle_property_disconnected_state_accepts_only_connect_or_rejoin() {
    for event in PEER_EVENTS {
        let mut lifecycle =
            PeerLifecycle::new("peer-disconnected-check").expect("peer should initialize");
        let outcome = lifecycle.transition(event);
        let should_succeed = matches!(
            event,
            PeerLifecycleEvent::StartConnect | PeerLifecycleEvent::Rejoin
        );
        assert_eq!(
            outcome.is_ok(),
            should_succeed,
            "event {event:?} success mismatch in disconnected state"
        );
    }
}

fn replay_peer_sequence(
    sequence: &[PeerLifecycleEvent],
) -> (
    PeerLifecycleState,
    Vec<Result<PeerLifecycleState, RuntimeLifecycleError>>,
) {
    let mut lifecycle = PeerLifecycle::new("peer-replay").expect("peer should initialize");
    let mut outcomes = Vec::with_capacity(sequence.len());
    for event in sequence {
        outcomes.push(lifecycle.transition(*event));
    }
    (lifecycle.state(), outcomes)
}

#[test]
fn peer_lifecycle_property_sequence_replay_is_deterministic() {
    // Bound sequence depth for fast CI while still exploring broad event permutations.
    for_each_peer_event_sequence(5, |sequence| {
        let (state_a, outcomes_a) = replay_peer_sequence(sequence);
        let (state_b, outcomes_b) = replay_peer_sequence(sequence);

        assert_eq!(
            outcomes_a, outcomes_b,
            "replaying identical peer lifecycle sequence must be deterministic"
        );
        assert_eq!(state_a, state_b);
    });
}

#[test]
fn peer_lifecycle_property_roundtrip_disconnect_recovers_connection_path() {
    for reconnect_event in [PeerLifecycleEvent::StartConnect, PeerLifecycleEvent::Rejoin] {
        let mut lifecycle =
            PeerLifecycle::new("peer-roundtrip").expect("peer roundtrip case should initialize");

        assert_eq!(
            lifecycle.transition(reconnect_event),
            Ok(PeerLifecycleState::Connecting)
        );
        assert_eq!(
            lifecycle.transition(PeerLifecycleEvent::HandshakeSucceeded),
            Ok(PeerLifecycleState::Active)
        );
        assert_eq!(
            lifecycle.transition(PeerLifecycleEvent::HeartbeatMissed),
            Ok(PeerLifecycleState::Degraded)
        );
        assert_eq!(
            lifecycle.transition(PeerLifecycleEvent::HeartbeatRestored),
            Ok(PeerLifecycleState::Active)
        );
        assert_eq!(
            lifecycle.transition(PeerLifecycleEvent::Disconnect),
            Ok(PeerLifecycleState::Disconnected)
        );
        assert_eq!(
            lifecycle.transition(reconnect_event),
            Ok(PeerLifecycleState::Connecting)
        );
        assert_eq!(
            lifecycle.transition(PeerLifecycleEvent::HandshakeSucceeded),
            Ok(PeerLifecycleState::Active)
        );
        assert_eq!(lifecycle.state(), PeerLifecycleState::Active);
    }
}

use kamn_core::{PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, RuntimeLifecycleError};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::{
    Config as ProptestConfig, FileFailurePersistence, RngAlgorithm, RngSeed,
};

const CASES: u32 = 192;
const MAX_SEQUENCE_LEN: usize = 40;
const LEGALITY_SEED: u64 = 0x3533_0000_0000_0001;
const IDEMPOTENCE_SEED: u64 = 0x3533_0000_0000_0002;
const REPLAY_SEED: u64 = 0x3533_0000_0000_0003;
const PROPTEST_SOURCE_PATH: &str = file!();

fn deterministic_config(cases: u32, seed: u64) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::SourceParallel(
            "proptest-regressions",
        ))),
        source_file: Some(PROPTEST_SOURCE_PATH),
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(seed),
        ..ProptestConfig::default()
    }
}

fn peer_event_strategy() -> impl Strategy<Value = PeerLifecycleEvent> {
    prop_oneof![
        Just(PeerLifecycleEvent::StartConnect),
        Just(PeerLifecycleEvent::HandshakeSucceeded),
        Just(PeerLifecycleEvent::HeartbeatMissed),
        Just(PeerLifecycleEvent::HeartbeatRestored),
        Just(PeerLifecycleEvent::Disconnect),
        Just(PeerLifecycleEvent::Rejoin),
    ]
}

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

fn replay_sequence(
    sequence: &[PeerLifecycleEvent],
) -> (
    PeerLifecycleState,
    Vec<Result<PeerLifecycleState, RuntimeLifecycleError>>,
) {
    let mut lifecycle = PeerLifecycle::new("peer-proptest-replay").expect("peer should initialize");
    let mut outcomes = Vec::with_capacity(sequence.len());
    for event in sequence {
        outcomes.push(lifecycle.transition(*event));
    }
    (lifecycle.state(), outcomes)
}

#[test]
fn unit_peer_lifecycle_proptest_config_is_deterministic_and_persistent() {
    let config = deterministic_config(CASES, LEGALITY_SEED);
    assert_eq!(config.cases, CASES);
    assert_eq!(config.rng_algorithm, RngAlgorithm::ChaCha);
    assert_eq!(config.rng_seed, RngSeed::Fixed(LEGALITY_SEED));
    assert_eq!(config.source_file, Some(PROPTEST_SOURCE_PATH));
    assert!(config.failure_persistence.is_some());
}

#[test]
fn regression_peer_lifecycle_seed_corpus_is_tracked() {
    let corpus =
        include_str!("../proptest-regressions/tests/peer_lifecycle_proptest_invariants.txt");
    assert!(corpus.contains("Seeds for failure cases"));
}

proptest! {
    #![proptest_config(deterministic_config(CASES, LEGALITY_SEED))]

    #[test]
    fn functional_peer_lifecycle_proptest_enforces_legal_transition_graph(
        sequence in vec(peer_event_strategy(), 0..(MAX_SEQUENCE_LEN + 1))
    ) {
        let mut lifecycle = PeerLifecycle::new("peer-proptest-legality").expect("peer should initialize");
        for event in sequence {
            let before_state = lifecycle.state();
            let expected = expected_next_state(before_state, event);

            match (expected, lifecycle.transition(event)) {
                (Some(next_state), Ok(applied_state)) => {
                    prop_assert_eq!(applied_state, next_state);
                    prop_assert_eq!(lifecycle.state(), next_state);
                }
                (None, Err(RuntimeLifecycleError::InvalidTransition { from, event: rejected })) => {
                    prop_assert_eq!(from, before_state);
                    prop_assert_eq!(rejected, event);
                    prop_assert_eq!(lifecycle.state(), before_state);
                }
                (Some(next_state), Err(error)) => {
                    prop_assert!(
                        false,
                        "expected legal transition to {next_state:?} from {before_state:?} via {event:?}, got error {error:?}"
                    );
                }
                (None, Ok(applied_state)) => {
                    prop_assert!(
                        false,
                        "expected illegal transition rejection from {before_state:?} via {event:?}, got state {applied_state:?}"
                    );
                }
                (None, Err(RuntimeLifecycleError::InvalidPeerId)) => {
                    prop_assert!(false, "peer id remains valid for the full property lane");
                }
            }
        }
    }
}

proptest! {
    #![proptest_config(deterministic_config(CASES, IDEMPOTENCE_SEED))]

    #[test]
    fn integration_peer_lifecycle_proptest_invalid_event_replays_are_idempotent(
        prefix in vec(peer_event_strategy(), 0..(MAX_SEQUENCE_LEN + 1)),
        repeated_event in peer_event_strategy(),
        repeats in 1_u8..=8_u8
    ) {
        let mut lifecycle = PeerLifecycle::new("peer-proptest-idempotence").expect("peer should initialize");
        for event in prefix {
            let _ = lifecycle.transition(event);
        }

        let baseline_state = lifecycle.state();
        prop_assume!(expected_next_state(baseline_state, repeated_event).is_none());

        for _ in 0..usize::from(repeats) {
            prop_assert_eq!(
                lifecycle.transition(repeated_event),
                Err(RuntimeLifecycleError::InvalidTransition {
                    from: baseline_state,
                    event: repeated_event,
                })
            );
            prop_assert_eq!(lifecycle.state(), baseline_state);
        }
    }
}

proptest! {
    #![proptest_config(deterministic_config(CASES, REPLAY_SEED))]

    #[test]
    fn integration_peer_lifecycle_proptest_sequence_replay_is_deterministic(
        sequence in vec(peer_event_strategy(), 0..(MAX_SEQUENCE_LEN + 1))
    ) {
        let (state_a, outcomes_a) = replay_sequence(&sequence);
        let (state_b, outcomes_b) = replay_sequence(&sequence);
        prop_assert_eq!(state_a, state_b);
        prop_assert_eq!(outcomes_a, outcomes_b);
    }
}

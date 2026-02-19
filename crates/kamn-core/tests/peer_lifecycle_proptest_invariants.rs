use kamn_core::{PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, RuntimeLifecycleError};
#[path = "property_invariant_helpers.rs"]
mod property_invariant_helpers;

use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::{RngAlgorithm, RngSeed};

const CASES: u32 = 192;
const MAX_SEQUENCE_LEN: usize = 40;
const ANTI_CHURN_REPEAT_MAX: u8 = 8;
const LEGALITY_SEED: u64 = 0x3533_0000_0000_0001;
const IDEMPOTENCE_SEED: u64 = 0x3533_0000_0000_0002;
const REPLAY_SEED: u64 = 0x3533_0000_0000_0003;
const PEER_SEED_ENV_KEY: &str = "KAMN_PROPTEST_PEER_LIFECYCLE_SEED";
const PEER_REASON_CODE_SEED_SALT: u64 = 0x00cc_dd11;
const PROPTEST_SOURCE_PATH: &str = file!();

fn base_peer_seed() -> u64 {
    property_invariant_helpers::resolve_seed_from_env(PEER_SEED_ENV_KEY, LEGALITY_SEED)
}

fn deterministic_config(cases: u32, seed: u64) -> proptest::test_runner::Config {
    property_invariant_helpers::deterministic_proptest_config(cases, seed, PROPTEST_SOURCE_PATH)
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
    let seed = base_peer_seed();
    let config = deterministic_config(CASES, seed);
    assert_eq!(config.cases, CASES);
    assert_eq!(config.rng_algorithm, RngAlgorithm::ChaCha);
    assert_eq!(config.rng_seed, RngSeed::Fixed(seed));
    assert_eq!(config.source_file, Some(PROPTEST_SOURCE_PATH));
    assert!(config.failure_persistence.is_some());
}

#[test]
fn regression_peer_lifecycle_seed_corpus_is_tracked() {
    let corpus =
        include_str!("../proptest-regressions/tests/peer_lifecycle_proptest_invariants.txt");
    assert!(corpus.contains("Seeds for failure cases"));
}

#[test]
fn unit_peer_lifecycle_proptest_budget_envelope_is_bounded() {
    let cases = std::hint::black_box(CASES);
    let max_sequence_len = std::hint::black_box(MAX_SEQUENCE_LEN);
    let anti_churn_repeat_max = std::hint::black_box(ANTI_CHURN_REPEAT_MAX);

    assert!(
        cases <= 256,
        "peer lifecycle property case budget must stay bounded for deterministic CI runtime"
    );
    assert!(
        max_sequence_len <= 40,
        "peer lifecycle transition sequence budget must stay bounded for deterministic CI runtime"
    );
    assert!(
        anti_churn_repeat_max <= 8,
        "anti-churn replay repeats must stay bounded for deterministic CI runtime"
    );
}

proptest! {
    #![proptest_config(deterministic_config(CASES, base_peer_seed()))]

    #[test]
    fn functional_peer_lifecycle_proptest_enforces_legal_transition_graph(
        sequence in vec(peer_event_strategy(), 0..(MAX_SEQUENCE_LEN + 1))
    ) {
        let mut lifecycle = PeerLifecycle::new("peer-proptest-legality").expect("peer should initialize");
        for event in sequence {
            let before_state = lifecycle.state();
            let expected = property_invariant_helpers::expected_peer_next_state(before_state, event);

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
    #![proptest_config(deterministic_config(
        CASES,
        property_invariant_helpers::derive_seed(base_peer_seed(), PEER_REASON_CODE_SEED_SALT)
    ))]

    #[test]
    fn functional_peer_lifecycle_proptest_invalid_transition_reason_code_is_stable(
        prefix in vec(peer_event_strategy(), 0..(MAX_SEQUENCE_LEN + 1)),
        invalid_event in peer_event_strategy()
    ) {
        let mut lifecycle = PeerLifecycle::new("peer-proptest-reason-code").expect("peer should initialize");
        for event in prefix {
            let _ = lifecycle.transition(event);
        }

        let before_state = lifecycle.state();
        prop_assume!(
            property_invariant_helpers::expected_peer_next_state(before_state, invalid_event)
                .is_none()
        );

        match lifecycle.transition(invalid_event) {
            Err(error @ RuntimeLifecycleError::InvalidTransition { from, event: rejected }) => {
                prop_assert_eq!(from, before_state);
                prop_assert_eq!(rejected, invalid_event);
                prop_assert_eq!(error.reason_code(), "runtime_peer_transition_invalid");
                prop_assert_eq!(lifecycle.state(), before_state);
            }
            Ok(next_state) => {
                prop_assert!(
                    false,
                    "expected invalid transition rejection from {before_state:?} via {invalid_event:?}, got state {next_state:?}"
                );
            }
            Err(RuntimeLifecycleError::InvalidPeerId) => {
                prop_assert!(false, "peer id is fixed and valid in this property lane");
            }
        }
    }
}

proptest! {
    #![proptest_config(deterministic_config(
        CASES,
        property_invariant_helpers::derive_seed(base_peer_seed(), IDEMPOTENCE_SEED)
    ))]

    #[test]
    fn integration_peer_lifecycle_proptest_invalid_event_replays_are_idempotent(
        prefix in vec(peer_event_strategy(), 0..(MAX_SEQUENCE_LEN + 1)),
        repeated_event in peer_event_strategy(),
        repeats in 1_u8..=ANTI_CHURN_REPEAT_MAX
    ) {
        let mut lifecycle = PeerLifecycle::new("peer-proptest-idempotence").expect("peer should initialize");
        for event in prefix {
            let _ = lifecycle.transition(event);
        }

        let baseline_state = lifecycle.state();
        prop_assume!(
            property_invariant_helpers::expected_peer_next_state(baseline_state, repeated_event)
                .is_none()
        );

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
    #![proptest_config(deterministic_config(
        CASES,
        property_invariant_helpers::derive_seed(base_peer_seed(), REPLAY_SEED)
    ))]

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

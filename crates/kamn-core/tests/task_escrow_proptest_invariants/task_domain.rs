use kamn_core::{TaskLifecycle, TaskLifecycleError, TaskTransition};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::{RngAlgorithm, RngSeed};

use super::shared::property_invariant_helpers;
use super::shared::{
    deterministic_config, history_starts_submitted, task_seed, MAX_SEQUENCE_LEN, TASK_CASES,
    TASK_EVIDENCE_SEED_SALT, TASK_RESTORE_SEED_SALT,
};

fn task_transition_strategy() -> impl Strategy<Value = TaskTransition> {
    prop_oneof![
        Just(TaskTransition::Accept),
        Just(TaskTransition::Delegate),
        Just(TaskTransition::StartWork),
        Just(TaskTransition::RequestInput),
        Just(TaskTransition::Block),
        Just(TaskTransition::Complete),
        Just(TaskTransition::Fail),
        Just(TaskTransition::Cancel),
    ]
}

#[test]
fn unit_task_escrow_proptest_config_is_deterministic_and_persistent() {
    let resolved_task_seed = task_seed();
    let config = deterministic_config(TASK_CASES, resolved_task_seed);
    assert_eq!(config.cases, TASK_CASES);
    assert_eq!(config.rng_algorithm, RngAlgorithm::ChaCha);
    assert_eq!(config.rng_seed, RngSeed::Fixed(resolved_task_seed));
    assert_eq!(
        config.source_file,
        Some("crates/kamn-core/tests/task_escrow_proptest_invariants.rs")
    );
    assert!(config.failure_persistence.is_some());
}

#[test]
fn regression_task_escrow_proptest_seed_corpus_is_tracked() {
    let corpus =
        include_str!("../../proptest-regressions/tests/task_escrow_proptest_invariants.txt");
    assert!(corpus.contains("Seeds for failure cases"));
}

proptest! {
    #![proptest_config(deterministic_config(TASK_CASES, task_seed()))]

    #[test]
    fn functional_task_lifecycle_proptest_sequence_invariants_hold(
        transitions in vec(task_transition_strategy(), 0..(MAX_SEQUENCE_LEN + 1))
    ) {
        let mut lifecycle = TaskLifecycle::new("task-proptest-sequence").expect("task lifecycle must initialize");
        for transition in transitions {
            let before_state = lifecycle.state();
            let before_history = lifecycle.history();

            match lifecycle.transition(transition) {
                Ok(()) => {
                    let after_state = lifecycle.state();
                    prop_assert!(
                        property_invariant_helpers::is_legal_task_state_step(before_state, after_state),
                        "illegal successful step: {before_state:?} -> {after_state:?} via {transition:?}"
                    );
                    prop_assert_eq!(lifecycle.history().len(), before_history.len() + 1);
                }
                Err(TaskLifecycleError::InvalidTransition { from, transition: rejected }) => {
                    prop_assert_eq!(from, before_state);
                    prop_assert_eq!(rejected, transition);
                    prop_assert_eq!(lifecycle.state(), before_state);
                    prop_assert_eq!(lifecycle.history(), before_history);
                }
                Err(TaskLifecycleError::TerminalState(state)) => {
                    prop_assert_eq!(state, before_state);
                    prop_assert_eq!(lifecycle.state(), before_state);
                    prop_assert_eq!(lifecycle.history(), before_history);
                }
                Err(error) => {
                    prop_assert!(false, "unexpected task lifecycle error: {error:?}");
                }
            }

            let history = lifecycle.history();
            prop_assert!(history_starts_submitted(history.as_slice()));
            prop_assert_eq!(history.last().copied(), Some(lifecycle.state()));
        }
    }
}

proptest! {
    #![proptest_config(deterministic_config(
        TASK_CASES,
        property_invariant_helpers::derive_seed(task_seed(), TASK_EVIDENCE_SEED_SALT)
    ))]

    #[test]
    fn functional_task_lifecycle_proptest_transition_evidence_is_legal_and_stable(
        transitions in vec(task_transition_strategy(), 0..(MAX_SEQUENCE_LEN + 1))
    ) {
        let mut lifecycle = TaskLifecycle::new("task-proptest-evidence").expect("task lifecycle must initialize");
        for transition in transitions {
            let before_state = lifecycle.state();
            let before_history = lifecycle.history();

            match lifecycle.transition_with_evidence(transition) {
                Ok(evidence) => {
                    prop_assert_eq!(evidence.from, before_state);
                    prop_assert_eq!(evidence.transition, transition);
                    prop_assert_eq!(evidence.to, lifecycle.state());
                    prop_assert_eq!(evidence.reason_code, "task_transition_allowed");
                    prop_assert!(
                        property_invariant_helpers::is_legal_task_state_step(
                            before_state,
                            lifecycle.state()
                        ),
                        "illegal successful transition with evidence: {before_state:?} -> {:?} via {transition:?}",
                        lifecycle.state()
                    );
                    prop_assert_eq!(lifecycle.history().len(), before_history.len() + 1);
                }
                Err(TaskLifecycleError::InvalidTransition { from, transition: rejected }) => {
                    prop_assert_eq!(from, before_state);
                    prop_assert_eq!(rejected, transition);
                    prop_assert_eq!(lifecycle.state(), before_state);
                    prop_assert_eq!(lifecycle.history(), before_history);
                }
                Err(TaskLifecycleError::TerminalState(state)) => {
                    prop_assert_eq!(state, before_state);
                    prop_assert_eq!(lifecycle.state(), before_state);
                    prop_assert_eq!(lifecycle.history(), before_history);
                }
                Err(error) => {
                    prop_assert!(false, "unexpected task lifecycle evidence error: {error:?}");
                }
            }
        }
    }
}

proptest! {
    #![proptest_config(deterministic_config(
        TASK_CASES,
        property_invariant_helpers::derive_seed(task_seed(), TASK_RESTORE_SEED_SALT)
    ))]

    #[test]
    fn integration_task_lifecycle_proptest_restore_roundtrip_is_stable(
        transitions in vec(task_transition_strategy(), 0..(MAX_SEQUENCE_LEN + 1))
    ) {
        let mut lifecycle = TaskLifecycle::new("task-proptest-restore").expect("task lifecycle must initialize");
        for transition in transitions {
            let _ = lifecycle.transition(transition);
        }

        let history = lifecycle.history();
        let restored = TaskLifecycle::restore("task-proptest-restore-copy", history.clone())
            .expect("history generated by transition replay must restore");
        prop_assert_eq!(restored.state(), lifecycle.state());
        prop_assert_eq!(restored.history(), history);
        prop_assert!(history_starts_submitted(restored.history().as_slice()));
    }
}

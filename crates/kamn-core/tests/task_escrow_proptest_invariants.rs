use kamn_core::{
    EscrowLifecycle, EscrowLifecycleError, EscrowStatus, EscrowTransitionAction, TaskLifecycle,
    TaskLifecycleError, TaskState, TaskTransition,
};
#[path = "property_invariant_helpers.rs"]
mod property_invariant_helpers;

use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::{RngAlgorithm, RngSeed};

const TASK_CASES: u32 = 192;
const ESCROW_CASES: u32 = 192;
const MAX_SEQUENCE_LEN: usize = 32;
const TASK_SEED: u64 = 0x3532_0000_0000_0001;
const ESCROW_SEED: u64 = 0x3532_0000_0000_0002;
const TASK_SEED_ENV_KEY: &str = "KAMN_PROPTEST_TASK_ESCROW_SEED";
const ESCROW_SEED_ENV_KEY: &str = "KAMN_PROPTEST_ESCROW_SEED";
const TASK_EVIDENCE_SEED_SALT: u64 = 0x0aa0_55ff;
const TASK_RESTORE_SEED_SALT: u64 = 0x0f0f_0f0f;
const ESCROW_EVIDENCE_SEED_SALT: u64 = 0x00ff_aacc;
const PROPTASK_SOURCE_PATH: &str = file!();

#[derive(Debug, Clone, Copy)]
enum EscrowAction {
    ReleaseScaled(u8),
    ReleaseRemaining,
    Dispute,
    ResolveHalfSplit,
    RefundRemaining,
}

fn task_seed() -> u64 {
    property_invariant_helpers::resolve_seed_from_env(TASK_SEED_ENV_KEY, TASK_SEED)
}

fn escrow_seed() -> u64 {
    property_invariant_helpers::resolve_seed_from_env(ESCROW_SEED_ENV_KEY, ESCROW_SEED)
}

fn deterministic_config(cases: u32, seed: u64) -> proptest::test_runner::Config {
    property_invariant_helpers::deterministic_proptest_config(cases, seed, PROPTASK_SOURCE_PATH)
}

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

fn escrow_action_strategy() -> impl Strategy<Value = EscrowAction> {
    prop_oneof![
        (1_u8..=16_u8).prop_map(EscrowAction::ReleaseScaled),
        Just(EscrowAction::ReleaseRemaining),
        Just(EscrowAction::Dispute),
        Just(EscrowAction::ResolveHalfSplit),
        Just(EscrowAction::RefundRemaining),
    ]
}

fn escrow_transition_action_strategy() -> impl Strategy<Value = EscrowTransitionAction> {
    prop_oneof![
        (1_u16..=256_u16).prop_map(|amount| EscrowTransitionAction::Release {
            amount: u128::from(amount)
        }),
        Just(EscrowTransitionAction::RefundRemaining),
        Just(EscrowTransitionAction::Dispute),
        ((0_u16..=256_u16), (0_u16..=256_u16)).prop_map(|(release_to_payee, refund_to_payer)| {
            EscrowTransitionAction::Resolve {
                release_to_payee: u128::from(release_to_payee),
                refund_to_payer: u128::from(refund_to_payer),
            }
        }),
    ]
}

fn apply_escrow_action(
    escrow: &mut EscrowLifecycle,
    action: EscrowAction,
) -> Result<(), EscrowLifecycleError> {
    let remaining = escrow.remaining_amount();
    match action {
        EscrowAction::ReleaseScaled(scale) => {
            let amount = if remaining == 0 {
                1
            } else {
                remaining.saturating_mul(u128::from(scale)).div_ceil(16)
            }
            .max(1);
            escrow.release(amount)
        }
        EscrowAction::ReleaseRemaining => escrow.release(remaining.max(1)),
        EscrowAction::Dispute => escrow.dispute(),
        EscrowAction::ResolveHalfSplit => {
            let release_to_payee = remaining / 2;
            let refund_to_payer = remaining.saturating_sub(release_to_payee);
            escrow.resolve(release_to_payee, refund_to_payer)
        }
        EscrowAction::RefundRemaining => escrow.refund_remaining(),
    }
}

fn escrow_invariant_violation(escrow: &EscrowLifecycle, total: u128) -> Option<String> {
    let released = escrow.released_amount();
    let refunded = escrow.refunded_amount();
    let remaining = escrow.remaining_amount();
    let total_projection = released
        .checked_add(refunded)
        .and_then(|value| value.checked_add(remaining));
    if total_projection != Some(total) {
        return Some(format!(
            "amount conservation failed: released={released}, refunded={refunded}, remaining={remaining}, total={total}"
        ));
    }

    match escrow.status() {
        EscrowStatus::Funded => {
            if released != 0 || refunded != 0 || remaining != total {
                return Some(format!(
                    "funded projection mismatch: released={released}, refunded={refunded}, remaining={remaining}, total={total}"
                ));
            }
        }
        EscrowStatus::PartiallyReleased {
            released: status_released,
            remaining: status_remaining,
        } => {
            if status_released != released || status_remaining != remaining || remaining == 0 {
                return Some(format!(
                    "partial projection mismatch: status_released={status_released}, released={released}, status_remaining={status_remaining}, remaining={remaining}"
                ));
            }
        }
        EscrowStatus::Released => {
            if released != total || refunded != 0 || remaining != 0 {
                return Some(format!(
                    "released projection mismatch: released={released}, refunded={refunded}, remaining={remaining}, total={total}"
                ));
            }
        }
        EscrowStatus::Refunded => {
            if refunded == 0 || remaining != 0 {
                return Some(format!(
                    "refunded projection mismatch: released={released}, refunded={refunded}, remaining={remaining}, total={total}"
                ));
            }
        }
        EscrowStatus::Disputed => {
            if remaining == 0 {
                return Some("disputed projection must keep non-zero remaining balance".to_owned());
            }
        }
        EscrowStatus::Resolved {
            released_total,
            refunded_total,
        } => {
            if released_total != released || refunded_total != refunded || remaining != 0 {
                return Some(format!(
                    "resolved projection mismatch: status_released={released_total}, released={released}, status_refunded={refunded_total}, refunded={refunded}, remaining={remaining}"
                ));
            }
        }
    }
    None
}

#[test]
fn unit_task_escrow_proptest_config_is_deterministic_and_persistent() {
    let resolved_task_seed = task_seed();
    let config = deterministic_config(TASK_CASES, resolved_task_seed);
    assert_eq!(config.cases, TASK_CASES);
    assert_eq!(config.rng_algorithm, RngAlgorithm::ChaCha);
    assert_eq!(config.rng_seed, RngSeed::Fixed(resolved_task_seed));
    assert_eq!(config.source_file, Some(PROPTASK_SOURCE_PATH));
    assert!(config.failure_persistence.is_some());
}

#[test]
fn regression_task_escrow_proptest_seed_corpus_is_tracked() {
    let corpus = include_str!("../proptest-regressions/tests/task_escrow_proptest_invariants.txt");
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
            prop_assert_eq!(history.first().copied(), Some(TaskState::Submitted));
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
        prop_assert_eq!(restored.history().first().copied(), Some(TaskState::Submitted));
    }
}

proptest! {
    #![proptest_config(deterministic_config(
        ESCROW_CASES,
        property_invariant_helpers::derive_seed(escrow_seed(), ESCROW_EVIDENCE_SEED_SALT)
    ))]

    #[test]
    fn integration_escrow_proptest_transition_evidence_preserves_invariants(
        total_amount in 1_u128..513_u128,
        actions in vec(escrow_transition_action_strategy(), 0..(MAX_SEQUENCE_LEN + 1))
    ) {
        let mut escrow = EscrowLifecycle::new(total_amount).expect("escrow lifecycle must initialize");
        if let Some(violation) = escrow_invariant_violation(&escrow, total_amount) {
            prop_assert!(false, "{violation}");
        }

        for action in actions {
            let before_status = escrow.status();
            let before_released = escrow.released_amount();
            let before_refunded = escrow.refunded_amount();
            let before_remaining = escrow.remaining_amount();

            match escrow.apply_transition_with_evidence(action.clone()) {
                Ok(evidence) => {
                    prop_assert_eq!(evidence.from, before_status);
                    prop_assert_eq!(evidence.action, action);
                    prop_assert_eq!(evidence.to, escrow.status());
                    prop_assert_eq!(evidence.reason_code, "escrow_transition_allowed");
                }
                Err(error) => {
                    prop_assert_eq!(escrow.status(), before_status);
                    prop_assert_eq!(escrow.released_amount(), before_released);
                    prop_assert_eq!(escrow.refunded_amount(), before_refunded);
                    prop_assert_eq!(escrow.remaining_amount(), before_remaining);
                    prop_assert!(
                        matches!(
                            error.reason_code(),
                            "escrow_amount_zero"
                                | "escrow_amount_invalid"
                                | "escrow_transition_invalid"
                                | "escrow_resolution_mismatch"
                                | "escrow_amount_overflow"
                        ),
                        "unexpected escrow rejection reason code: {}",
                        error.reason_code()
                    );
                }
            }

            if let Some(violation) = escrow_invariant_violation(&escrow, total_amount) {
                prop_assert!(false, "{violation}");
            }
        }
    }
}

proptest! {
    #![proptest_config(deterministic_config(ESCROW_CASES, escrow_seed()))]

    #[test]
    fn integration_escrow_proptest_conserves_amounts_and_status_projections(
        total_amount in 1_u128..513_u128,
        actions in vec(escrow_action_strategy(), 0..(MAX_SEQUENCE_LEN + 1))
    ) {
        let mut escrow = EscrowLifecycle::new(total_amount).expect("escrow lifecycle must initialize");
        if let Some(violation) = escrow_invariant_violation(&escrow, total_amount) {
            prop_assert!(false, "{violation}");
        }

        for action in actions {
            let before_status = escrow.status();
            let before_released = escrow.released_amount();
            let before_refunded = escrow.refunded_amount();
            let before_remaining = escrow.remaining_amount();

            match apply_escrow_action(&mut escrow, action) {
                Ok(()) => {
                    if let Some(violation) = escrow_invariant_violation(&escrow, total_amount) {
                        prop_assert!(false, "{violation}");
                    }
                }
                Err(error) => {
                    prop_assert_eq!(escrow.status(), before_status);
                    prop_assert_eq!(escrow.released_amount(), before_released);
                    prop_assert_eq!(escrow.refunded_amount(), before_refunded);
                    prop_assert_eq!(escrow.remaining_amount(), before_remaining);
                    prop_assert!(
                        matches!(
                            error.reason_code(),
                            "escrow_amount_zero"
                                | "escrow_amount_invalid"
                                | "escrow_transition_invalid"
                                | "escrow_resolution_mismatch"
                                | "escrow_amount_overflow"
                        ),
                        "unexpected escrow rejection reason code: {}",
                        error.reason_code()
                    );
                    if let Some(violation) = escrow_invariant_violation(&escrow, total_amount) {
                        prop_assert!(false, "{violation}");
                    }
                }
            }
        }
    }
}

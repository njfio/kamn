use kamn_core::{
    EscrowLifecycle, EscrowStatus, EscrowTransitionAction, PeerLifecycle, PeerLifecycleEvent,
    PeerLifecycleState, RuntimeLifecycleError, TaskLifecycle, TaskState, TaskTransition,
};

const TASK_TRANSITIONS: [TaskTransition; 8] = [
    TaskTransition::Accept,
    TaskTransition::Delegate,
    TaskTransition::StartWork,
    TaskTransition::RequestInput,
    TaskTransition::Block,
    TaskTransition::Complete,
    TaskTransition::Fail,
    TaskTransition::Cancel,
];

const PEER_EVENTS: [PeerLifecycleEvent; 6] = [
    PeerLifecycleEvent::StartConnect,
    PeerLifecycleEvent::HandshakeSucceeded,
    PeerLifecycleEvent::HeartbeatMissed,
    PeerLifecycleEvent::HeartbeatRestored,
    PeerLifecycleEvent::Disconnect,
    PeerLifecycleEvent::Rejoin,
];

#[derive(Debug, Clone, Copy)]
enum EscrowPropertyAction {
    ReleaseOne,
    ReleaseRemaining,
    Dispute,
    ResolveHalfSplit,
    RefundRemaining,
}

const ESCROW_PROPERTY_ACTIONS: [EscrowPropertyAction; 5] = [
    EscrowPropertyAction::ReleaseOne,
    EscrowPropertyAction::ReleaseRemaining,
    EscrowPropertyAction::Dispute,
    EscrowPropertyAction::ResolveHalfSplit,
    EscrowPropertyAction::RefundRemaining,
];

fn for_each_sequence<T: Copy>(alphabet: &[T], max_len: usize, mut f: impl FnMut(&[T])) {
    fn recurse<T: Copy>(
        alphabet: &[T],
        target_len: usize,
        current: &mut Vec<T>,
        f: &mut impl FnMut(&[T]),
    ) {
        if current.len() == target_len {
            f(current.as_slice());
            return;
        }

        for item in alphabet {
            current.push(*item);
            recurse(alphabet, target_len, current, f);
            current.pop();
        }
    }

    let mut current = Vec::new();
    for len in 1..=max_len {
        recurse(alphabet, len, &mut current, &mut f);
    }
}

fn is_legal_task_state_step(from: TaskState, to: TaskState) -> bool {
    matches!(
        (from, to),
        (TaskState::Submitted, TaskState::Accepted)
            | (TaskState::Submitted, TaskState::Cancelled)
            | (TaskState::Accepted, TaskState::Delegated)
            | (TaskState::Accepted, TaskState::InProgress)
            | (TaskState::Accepted, TaskState::Cancelled)
            | (TaskState::Delegated, TaskState::InProgress)
            | (TaskState::Delegated, TaskState::Cancelled)
            | (TaskState::InProgress, TaskState::Blocked)
            | (TaskState::InProgress, TaskState::InputRequired)
            | (TaskState::InProgress, TaskState::Completed)
            | (TaskState::InProgress, TaskState::Failed)
            | (TaskState::InProgress, TaskState::Cancelled)
            | (TaskState::InputRequired, TaskState::InProgress)
            | (TaskState::InputRequired, TaskState::Failed)
            | (TaskState::InputRequired, TaskState::Cancelled)
            | (TaskState::Blocked, TaskState::InProgress)
            | (TaskState::Blocked, TaskState::Failed)
            | (TaskState::Blocked, TaskState::Cancelled)
    )
}

fn to_escrow_transition_action(
    escrow: &EscrowLifecycle,
    action: EscrowPropertyAction,
) -> EscrowTransitionAction {
    let remaining = escrow.remaining_amount();
    match action {
        EscrowPropertyAction::ReleaseOne => EscrowTransitionAction::Release { amount: 1 },
        EscrowPropertyAction::ReleaseRemaining => EscrowTransitionAction::Release {
            amount: remaining.max(1),
        },
        EscrowPropertyAction::Dispute => EscrowTransitionAction::Dispute,
        EscrowPropertyAction::ResolveHalfSplit => {
            let release_to_payee = remaining / 2;
            let refund_to_payer = remaining.saturating_sub(release_to_payee);
            EscrowTransitionAction::Resolve {
                release_to_payee,
                refund_to_payer,
            }
        }
        EscrowPropertyAction::RefundRemaining => EscrowTransitionAction::RefundRemaining,
    }
}

fn assert_escrow_amount_invariants(escrow: &EscrowLifecycle, total_amount: u128) {
    let released = escrow.released_amount();
    let refunded = escrow.refunded_amount();
    let remaining = escrow.remaining_amount();
    assert_eq!(released + refunded + remaining, total_amount);

    match escrow.status() {
        EscrowStatus::Funded => {
            assert_eq!(released, 0);
            assert_eq!(refunded, 0);
        }
        EscrowStatus::PartiallyReleased {
            released: status_released,
            remaining: status_remaining,
        } => {
            assert_eq!(status_released, released);
            assert_eq!(status_remaining, remaining);
            assert!(remaining > 0);
        }
        EscrowStatus::Released => {
            assert_eq!(remaining, 0);
            assert_eq!(refunded, 0);
        }
        EscrowStatus::Refunded | EscrowStatus::Resolved { .. } => {
            assert_eq!(remaining, 0);
        }
        EscrowStatus::Disputed => {}
    }
}

fn expected_peer_next_state(
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

#[test]
fn lifecycle_property_task_evidence_sequences_preserve_transition_contracts() {
    // Regression: #1526
    // Keep this lane bounded for fast, low-cost CI execution.
    for_each_sequence(&TASK_TRANSITIONS, 4, |sequence| {
        let mut lifecycle =
            TaskLifecycle::new("task-evidence-property").expect("task property case should init");
        for transition in sequence {
            let before_state = lifecycle.state();
            let before_history = lifecycle.history();
            match lifecycle.transition_with_evidence(*transition) {
                Ok(evidence) => {
                    assert_eq!(evidence.from, before_state);
                    assert_eq!(evidence.transition, *transition);
                    assert_eq!(evidence.to, lifecycle.state());
                    assert_eq!(evidence.reason_code, "task_transition_allowed");
                    assert!(
                        is_legal_task_state_step(before_state, evidence.to),
                        "task evidence transition must be legal from {before_state:?} to {:?}",
                        evidence.to
                    );
                    assert_eq!(lifecycle.history().len(), before_history.len() + 1);
                }
                Err(error) => {
                    assert_eq!(lifecycle.state(), before_state);
                    assert_eq!(lifecycle.history(), before_history);
                    assert!(matches!(
                        error.reason_code(),
                        "task_transition_invalid_edge" | "task_transition_terminal_state"
                    ));
                }
            }
        }
    });
}

#[test]
fn lifecycle_property_escrow_evidence_sequences_preserve_amount_and_reason_invariants() {
    // Regression: #1526
    // Keep the matrix bounded so this property lane remains cheap in fast-gate CI.
    for total_amount in [1_u128, 2, 3, 5, 8] {
        for_each_sequence(&ESCROW_PROPERTY_ACTIONS, 4, |sequence| {
            let mut escrow =
                EscrowLifecycle::new(total_amount).expect("escrow property case should init");
            assert_escrow_amount_invariants(&escrow, total_amount);

            for action in sequence {
                let before_status = escrow.status();
                let before_released = escrow.released_amount();
                let before_refunded = escrow.refunded_amount();
                let before_remaining = escrow.remaining_amount();
                let transition_action = to_escrow_transition_action(&escrow, *action);

                match escrow.apply_transition_with_evidence(transition_action.clone()) {
                    Ok(evidence) => {
                        assert_eq!(evidence.from, before_status);
                        assert_eq!(evidence.action, transition_action);
                        assert_eq!(evidence.to, escrow.status());
                        assert_eq!(evidence.reason_code, "escrow_transition_allowed");
                        assert_escrow_amount_invariants(&escrow, total_amount);
                    }
                    Err(error) => {
                        assert_eq!(escrow.status(), before_status);
                        assert_eq!(escrow.released_amount(), before_released);
                        assert_eq!(escrow.refunded_amount(), before_refunded);
                        assert_eq!(escrow.remaining_amount(), before_remaining);
                        assert!(matches!(
                            error.reason_code(),
                            "escrow_transition_invalid"
                                | "escrow_amount_invalid"
                                | "escrow_resolution_mismatch"
                        ));
                    }
                }
            }
        });
    }
}

#[test]
fn lifecycle_property_runtime_peer_sequences_match_transition_contract() {
    // Regression: #1526
    // Sequence depth is intentionally bounded for fast feedback while retaining broad coverage.
    for_each_sequence(&PEER_EVENTS, 4, |sequence| {
        let mut lifecycle = PeerLifecycle::new("peer-property").expect("peer property should init");
        for event in sequence {
            let before_state = lifecycle.state();
            let expected_next = expected_peer_next_state(before_state, *event);
            match (expected_next, lifecycle.transition(*event)) {
                (Some(next_state), Ok(applied_state)) => {
                    assert_eq!(next_state, applied_state);
                    assert_eq!(lifecycle.state(), applied_state);
                }
                (
                    None,
                    Err(RuntimeLifecycleError::InvalidTransition {
                        from,
                        event: rejected_event,
                    }),
                ) => {
                    assert_eq!(from, before_state);
                    assert_eq!(rejected_event, *event);
                    assert_eq!(lifecycle.state(), before_state);
                }
                (Some(_), Err(error)) => panic!(
                    "expected successful runtime transition from {before_state:?} via {event:?}, \
                     got {error:?}"
                ),
                (None, Ok(applied_state)) => panic!(
                    "expected invalid runtime transition from {before_state:?} via {event:?}, \
                     got {applied_state:?}"
                ),
                (None, Err(RuntimeLifecycleError::InvalidPeerId)) => {
                    panic!("peer id is fixed and valid in this property lane")
                }
            }
        }
    });
}

#[test]
fn lifecycle_property_runtime_peer_sequence_replay_is_deterministic() {
    // Regression: #1526
    // Keep this deterministic replay check bounded for fast CI.
    for_each_sequence(&PEER_EVENTS, 4, |sequence| {
        let mut run_a = PeerLifecycle::new("peer-replay-a").expect("peer replay A should init");
        let mut outcomes_a = Vec::with_capacity(sequence.len());
        for event in sequence {
            outcomes_a.push(run_a.transition(*event));
        }

        let mut run_b = PeerLifecycle::new("peer-replay-b").expect("peer replay B should init");
        let mut outcomes_b = Vec::with_capacity(sequence.len());
        for event in sequence {
            outcomes_b.push(run_b.transition(*event));
        }

        assert_eq!(outcomes_a, outcomes_b);
        assert_eq!(run_a.state(), run_b.state());
    });
}

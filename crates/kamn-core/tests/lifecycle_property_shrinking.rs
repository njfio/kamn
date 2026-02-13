use kamn_core::{
    EscrowLifecycle, EscrowStatus, EscrowTransitionAction, PeerLifecycle, PeerLifecycleEvent,
    PeerLifecycleState, RuntimeLifecycleError, TaskLifecycle, TaskState, TaskTransition,
};
use std::time::Instant;

const PROPERTY_SEEDS: [u64; 16] = [
    0x0000_0000_0000_0001,
    0x0000_0000_0000_0042,
    0x0000_0000_0000_01f4,
    0x0000_0000_0000_04d2,
    0x0000_0000_0000_1337,
    0x0000_0000_0000_2710,
    0x0000_0000_0000_cafe,
    0x0000_0000_0000_beef,
    0x0000_0000_0001_0001,
    0x0000_0000_0001_3370,
    0x0000_0000_0002_0002,
    0x0000_0000_0007_7777,
    0x0000_0000_00ab_cdef,
    0x0000_0000_1234_5678,
    0x0000_0000_7fff_ffff,
    0x0000_0000_ffff_ffff,
];

const MAX_SEQUENCE_LEN: usize = 24;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EscrowAction {
    ReleaseOne,
    ReleaseHalf,
    Dispute,
    ResolveHalfSplit,
    RefundRemaining,
}

const ESCROW_ACTIONS: [EscrowAction; 5] = [
    EscrowAction::ReleaseOne,
    EscrowAction::ReleaseHalf,
    EscrowAction::Dispute,
    EscrowAction::ResolveHalfSplit,
    EscrowAction::RefundRemaining,
];

fn xorshift64(mut value: u64) -> u64 {
    if value == 0 {
        value = 0x9e37_79b9_7f4a_7c15;
    }
    value ^= value << 13;
    value ^= value >> 7;
    value ^= value << 17;
    value
}

fn seeded_sequence<T: Copy>(seed: u64, alphabet: &[T], len: usize) -> Vec<T> {
    let mut state = seed;
    let mut sequence = Vec::with_capacity(len);
    for _ in 0..len {
        state = xorshift64(state);
        let index = (state as usize) % alphabet.len();
        sequence.push(alphabet[index]);
    }
    sequence
}

fn shrink_failing_prefix<T: Copy>(sequence: &[T], fails: impl Fn(&[T]) -> bool) -> Vec<T> {
    for end in 1..=sequence.len() {
        let prefix = &sequence[..end];
        if fails(prefix) {
            return prefix.to_vec();
        }
    }
    sequence.to_vec()
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

fn to_escrow_transition_action(
    escrow: &EscrowLifecycle,
    action: EscrowAction,
) -> EscrowTransitionAction {
    let remaining = escrow.remaining_amount();
    match action {
        EscrowAction::ReleaseOne => EscrowTransitionAction::Release { amount: 1 },
        EscrowAction::ReleaseHalf => EscrowTransitionAction::Release {
            amount: remaining / 2,
        },
        EscrowAction::Dispute => EscrowTransitionAction::Dispute,
        EscrowAction::ResolveHalfSplit => {
            let release_to_payee = remaining / 2;
            let refund_to_payer = remaining.saturating_sub(release_to_payee);
            EscrowTransitionAction::Resolve {
                release_to_payee,
                refund_to_payer,
            }
        }
        EscrowAction::RefundRemaining => EscrowTransitionAction::RefundRemaining,
    }
}

fn assert_escrow_amount_invariants(escrow: &EscrowLifecycle, total_amount: u128) -> Option<String> {
    let released = escrow.released_amount();
    let refunded = escrow.refunded_amount();
    let remaining = escrow.remaining_amount();
    if released + refunded + remaining != total_amount {
        return Some(format!(
            "amount conservation failed: released={released}, refunded={refunded}, remaining={remaining}, total={total_amount}"
        ));
    }

    match escrow.status() {
        EscrowStatus::PartiallyReleased {
            released: status_released,
            remaining: status_remaining,
        } => {
            if status_released != released {
                return Some(format!(
                    "partial-release status mismatch: status released {status_released}, ledger released {released}"
                ));
            }
            if status_remaining != remaining {
                return Some(format!(
                    "partial-release remaining mismatch: status remaining {status_remaining}, ledger remaining {remaining}"
                ));
            }
        }
        EscrowStatus::Released | EscrowStatus::Refunded | EscrowStatus::Resolved { .. } => {
            if remaining != 0 {
                return Some(format!(
                    "terminal escrow state must have zero remaining amount, found {remaining}"
                ));
            }
        }
        EscrowStatus::Funded | EscrowStatus::Disputed => {}
    }
    None
}

fn task_sequence_failure(sequence: &[TaskTransition]) -> Option<String> {
    let mut lifecycle =
        TaskLifecycle::new("task-property-seeded").expect("task lifecycle should init");
    for (index, transition) in sequence.iter().enumerate() {
        let before_state = lifecycle.state();
        let before_history = lifecycle.history();
        match lifecycle.transition(*transition) {
            Ok(()) => {
                let after_state = lifecycle.state();
                if !is_legal_task_state_step(before_state, after_state) {
                    return Some(format!(
                        "illegal task transition at index {index}: {before_state:?} -> {after_state:?} via {transition:?}"
                    ));
                }
                let history = lifecycle.history();
                if history.len() != before_history.len() + 1 {
                    return Some(format!(
                        "task history length drift at index {index}: expected {}, found {}",
                        before_history.len() + 1,
                        history.len()
                    ));
                }
                if history.last().copied() != Some(after_state) {
                    return Some(format!("task history tail mismatch at index {index}"));
                }
            }
            Err(error) => {
                if lifecycle.state() != before_state {
                    return Some(format!(
                        "task state mutated on rejected transition at index {index}: {:?} -> {:?}",
                        before_state,
                        lifecycle.state()
                    ));
                }
                if lifecycle.history() != before_history {
                    return Some(format!(
                        "task history mutated on rejection at index {index}"
                    ));
                }
                if !matches!(
                    error.reason_code(),
                    "task_transition_invalid_edge" | "task_transition_terminal_state"
                ) {
                    return Some(format!(
                        "unexpected task rejection reason code at index {index}: {}",
                        error.reason_code()
                    ));
                }
            }
        }
    }
    None
}

fn peer_sequence_failure(sequence: &[PeerLifecycleEvent]) -> Option<String> {
    let mut lifecycle =
        PeerLifecycle::new("peer-property-seeded").expect("peer lifecycle should init");
    for (index, event) in sequence.iter().enumerate() {
        let before_state = lifecycle.state();
        let expected_next = expected_peer_next_state(before_state, *event);
        match (expected_next, lifecycle.transition(*event)) {
            (Some(next_state), Ok(applied_state)) => {
                if next_state != applied_state || lifecycle.state() != applied_state {
                    return Some(format!(
                        "peer transition mismatch at index {index}: expected {next_state:?}, applied {applied_state:?}"
                    ));
                }
            }
            (
                None,
                Err(RuntimeLifecycleError::InvalidTransition {
                    from,
                    event: rejected_event,
                }),
            ) => {
                if from != before_state || rejected_event != *event {
                    return Some(format!(
                        "peer rejection mismatch at index {index}: from {from:?}, event {rejected_event:?}"
                    ));
                }
                if lifecycle.state() != before_state {
                    return Some(format!(
                        "peer state mutated on rejection at index {index}: {:?} -> {:?}",
                        before_state,
                        lifecycle.state()
                    ));
                }
            }
            (Some(_), Err(error)) => {
                return Some(format!(
                    "peer transition unexpectedly failed at index {index}: {error}"
                ));
            }
            (None, Ok(applied_state)) => {
                return Some(format!(
                    "peer transition unexpectedly succeeded at index {index}: {applied_state:?}"
                ));
            }
            (None, Err(RuntimeLifecycleError::InvalidPeerId)) => {
                return Some("peer id should remain valid for seeded lane".to_owned());
            }
        }
    }
    None
}

fn escrow_sequence_failure(sequence: &[EscrowAction], total_amount: u128) -> Option<String> {
    let mut escrow = EscrowLifecycle::new(total_amount).expect("escrow lifecycle should init");
    if let Some(error) = assert_escrow_amount_invariants(&escrow, total_amount) {
        return Some(error);
    }

    for (index, action) in sequence.iter().enumerate() {
        let before_status = escrow.status();
        let before_released = escrow.released_amount();
        let before_refunded = escrow.refunded_amount();
        let before_remaining = escrow.remaining_amount();
        let transition_action = to_escrow_transition_action(&escrow, *action);

        match escrow.apply_transition_with_evidence(transition_action) {
            Ok(_) => {
                if let Some(error) = assert_escrow_amount_invariants(&escrow, total_amount) {
                    return Some(format!("escrow invariant failed at index {index}: {error}"));
                }
            }
            Err(error) => {
                if escrow.status() != before_status
                    || escrow.released_amount() != before_released
                    || escrow.refunded_amount() != before_refunded
                    || escrow.remaining_amount() != before_remaining
                {
                    return Some(format!(
                        "escrow mutated on rejected action at index {index}: reason={}",
                        error.reason_code()
                    ));
                }
                if !matches!(
                    error.reason_code(),
                    "escrow_transition_invalid"
                        | "escrow_amount_zero"
                        | "escrow_amount_invalid"
                        | "escrow_amount_overflow"
                        | "escrow_resolution_mismatch"
                ) {
                    return Some(format!(
                        "unexpected escrow rejection reason at index {index}: {}",
                        error.reason_code()
                    ));
                }
            }
        }
    }
    None
}

#[test]
fn unit_property_shrinker_returns_minimal_failing_prefix() {
    let sequence = [1_i32, 2, 3, 4, 5];
    let shrunk = shrink_failing_prefix(&sequence, |prefix| prefix.iter().sum::<i32>() >= 6);
    assert_eq!(shrunk, vec![1, 2, 3]);
}

#[test]
fn property_seeded_task_lifecycle_invariants_support_shrinking() {
    // Regression: #2692
    for seed in PROPERTY_SEEDS {
        let sequence = seeded_sequence(seed, &TASK_TRANSITIONS, MAX_SEQUENCE_LEN);
        if let Some(error) = task_sequence_failure(&sequence) {
            let shrunk =
                shrink_failing_prefix(&sequence, |prefix| task_sequence_failure(prefix).is_some());
            panic!(
                "task property failure seed={seed} len={} shrunk_len={} error={error} shrunk={shrunk:?}",
                sequence.len(),
                shrunk.len(),
            );
        }
    }
}

#[test]
fn property_seeded_peer_lifecycle_invariants_support_shrinking() {
    // Regression: #2692
    for seed in PROPERTY_SEEDS {
        let sequence =
            seeded_sequence(seed ^ 0xa5a5_5a5a_0000_0001, &PEER_EVENTS, MAX_SEQUENCE_LEN);
        if let Some(error) = peer_sequence_failure(&sequence) {
            let shrunk =
                shrink_failing_prefix(&sequence, |prefix| peer_sequence_failure(prefix).is_some());
            panic!(
                "peer property failure seed={seed} len={} shrunk_len={} error={error} shrunk={shrunk:?}",
                sequence.len(),
                shrunk.len(),
            );
        }
    }
}

#[test]
fn property_seeded_escrow_lifecycle_invariants_support_shrinking() {
    // Regression: #2692
    for seed in PROPERTY_SEEDS {
        let total_amount = (seed % 23 + 1) as u128;
        let sequence = seeded_sequence(
            seed ^ 0x5a5a_a5a5_0000_0002,
            &ESCROW_ACTIONS,
            MAX_SEQUENCE_LEN,
        );
        if let Some(error) = escrow_sequence_failure(&sequence, total_amount) {
            let shrunk = shrink_failing_prefix(&sequence, |prefix| {
                escrow_sequence_failure(prefix, total_amount).is_some()
            });
            panic!(
                "escrow property failure seed={seed} total={total_amount} len={} shrunk_len={} error={error} shrunk={shrunk:?}",
                sequence.len(),
                shrunk.len(),
            );
        }
    }
}

#[test]
fn performance_seeded_lifecycle_property_lane_stays_within_ci_budget() {
    let started = Instant::now();
    for seed in PROPERTY_SEEDS {
        let task_sequence = seeded_sequence(seed, &TASK_TRANSITIONS, MAX_SEQUENCE_LEN);
        assert!(task_sequence_failure(&task_sequence).is_none());

        let peer_sequence =
            seeded_sequence(seed ^ 0xa5a5_5a5a_0000_0001, &PEER_EVENTS, MAX_SEQUENCE_LEN);
        assert!(peer_sequence_failure(&peer_sequence).is_none());

        let total_amount = (seed % 23 + 1) as u128;
        let escrow_sequence = seeded_sequence(
            seed ^ 0x5a5a_a5a5_0000_0002,
            &ESCROW_ACTIONS,
            MAX_SEQUENCE_LEN,
        );
        assert!(escrow_sequence_failure(&escrow_sequence, total_amount).is_none());
    }

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 250,
        "seeded lifecycle property lane exceeded CI budget: {elapsed_millis}ms"
    );
}

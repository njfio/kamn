use kamn_core::{
    EscrowLifecycle, EscrowTransitionAction, PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState,
    RuntimeLifecycleError, TaskOperationEngine, TaskOperationError, TaskState,
};
use std::env;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Instant;

fn concurrency_replay_fixture() -> [&'static [&'static str]; 3] {
    [
        &[
            "kamn:did:agent:worker-1",
            "kamn:did:agent:worker-2",
            "kamn:did:agent:worker-3",
        ],
        &[
            "kamn:did:agent:worker-a",
            "kamn:did:agent:worker-b",
            "kamn:did:agent:worker-c",
            "kamn:did:agent:worker-d",
        ],
        &[
            "kamn:did:agent:worker-11",
            "kamn:did:agent:worker-12",
            "kamn:did:agent:worker-13",
            "kamn:did:agent:worker-14",
            "kamn:did:agent:worker-15",
        ],
    ]
}

fn run_task_accept_race(task_id: &str, contenders: &[&str]) -> (usize, usize, Option<String>) {
    let engine = Arc::new(Mutex::new(TaskOperationEngine::new()));
    engine
        .lock()
        .expect("engine lock should initialize")
        .submit(
            task_id,
            "kamn:did:agent:requester-1",
            "Concurrency accept gate",
        )
        .expect("submit should succeed");

    let barrier = Arc::new(Barrier::new(contenders.len()));
    let mut handles = Vec::new();
    let task_id = task_id.to_owned();

    for contender in contenders {
        let actor = contender.to_string();
        let task_id = task_id.clone();
        let engine = Arc::clone(&engine);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            let result = engine
                .lock()
                .expect("engine lock should acquire")
                .accept(&task_id, &actor);
            (actor, result)
        }));
    }

    let mut success_count = 0_usize;
    let mut unauthorized_count = 0_usize;
    let mut winner: Option<String> = None;

    for handle in handles {
        let (actor, outcome) = handle.join().expect("task accept thread should join");
        match outcome {
            Ok(()) => {
                success_count += 1;
                winner = Some(actor);
            }
            Err(TaskOperationError::UnauthorizedActor {
                actor: rejected_actor,
                required,
            }) => {
                unauthorized_count += 1;
                assert_eq!(rejected_actor, actor);
                assert_eq!(required, "unassigned_or_current_assignee");
            }
            Err(error) => panic!("unexpected task accept concurrency error: {error:?}"),
        }
    }

    let engine = engine.lock().expect("engine lock should acquire");
    let task = engine.task(&task_id).expect("task should exist");
    assert_eq!(task.lifecycle.state(), TaskState::Accepted);
    if let Some(winner) = &winner {
        assert_eq!(task.assignee.as_deref(), Some(winner.as_str()));
    }

    (success_count, unauthorized_count, winner)
}

fn run_peer_lifecycle_race(peer_id: &str) -> ([usize; 3], [usize; 3], PeerLifecycleState) {
    let lifecycle = Arc::new(Mutex::new(
        PeerLifecycle::new(peer_id).expect("peer lifecycle should initialize"),
    ));
    let barrier = Arc::new(Barrier::new(2));
    let mut handles = Vec::new();

    for _ in 0..2 {
        let lifecycle = Arc::clone(&lifecycle);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            let mut outcomes = Vec::new();
            let phases = [
                PeerLifecycleEvent::StartConnect,
                PeerLifecycleEvent::HandshakeSucceeded,
                PeerLifecycleEvent::Disconnect,
            ];

            for event in phases {
                barrier.wait();
                let outcome = lifecycle
                    .lock()
                    .expect("peer lifecycle lock should acquire")
                    .transition(event);
                outcomes.push((event, outcome));
                barrier.wait();
            }
            outcomes
        }));
    }

    let mut success_by_phase = [0_usize; 3];
    let mut invalid_by_phase = [0_usize; 3];

    for handle in handles {
        let outcomes = handle.join().expect("peer lifecycle thread should join");
        for (phase_index, (event, outcome)) in outcomes.into_iter().enumerate() {
            match outcome {
                Ok(next_state) => {
                    success_by_phase[phase_index] += 1;
                    match (phase_index, next_state) {
                        (0, PeerLifecycleState::Connecting)
                        | (1, PeerLifecycleState::Active)
                        | (2, PeerLifecycleState::Disconnected) => {}
                        _ => panic!(
                            "unexpected successful transition in phase {phase_index}: \
                             {next_state:?}"
                        ),
                    }
                }
                Err(RuntimeLifecycleError::InvalidTransition {
                    from,
                    event: rejected_event,
                }) => {
                    invalid_by_phase[phase_index] += 1;
                    assert_eq!(rejected_event, event);
                    match phase_index {
                        0 => assert_eq!(from, PeerLifecycleState::Connecting),
                        1 => assert_eq!(from, PeerLifecycleState::Active),
                        2 => assert_eq!(from, PeerLifecycleState::Disconnected),
                        _ => unreachable!("phase index is bounded by static phases"),
                    }
                }
                Err(RuntimeLifecycleError::InvalidPeerId) => {
                    panic!("peer id is fixed and valid in concurrency lane");
                }
            }
        }
    }

    let final_state = lifecycle
        .lock()
        .expect("peer lifecycle lock should acquire")
        .state();
    (success_by_phase, invalid_by_phase, final_state)
}

fn run_escrow_dispute_refund_race(total_amount: u128) -> (usize, usize, u128, u128, u128) {
    let escrow = Arc::new(Mutex::new(
        EscrowLifecycle::new(total_amount).expect("escrow should initialize"),
    ));
    let barrier = Arc::new(Barrier::new(2));
    let actions = [
        EscrowTransitionAction::Dispute,
        EscrowTransitionAction::RefundRemaining,
    ];
    let mut handles = Vec::new();

    for action in actions {
        let escrow = Arc::clone(&escrow);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            escrow
                .lock()
                .expect("escrow lock should acquire")
                .apply_transition_with_evidence(action)
        }));
    }

    let mut success_count = 0_usize;
    let mut invalid_count = 0_usize;

    for handle in handles {
        match handle
            .join()
            .expect("escrow dispute/refund thread should join")
        {
            Ok(evidence) => {
                success_count += 1;
                assert_eq!(evidence.reason_code, "escrow_transition_allowed");
            }
            Err(error) => {
                invalid_count += 1;
                assert_eq!(error.reason_code(), "escrow_transition_invalid");
            }
        }
    }

    let escrow = escrow.lock().expect("escrow lock should acquire");
    (
        success_count,
        invalid_count,
        escrow.released_amount(),
        escrow.refunded_amount(),
        escrow.remaining_amount(),
    )
}

fn run_escrow_refund_race(
    total_amount: u128,
) -> (usize, usize, Vec<&'static str>, u128, u128, u128) {
    let escrow = Arc::new(Mutex::new(
        EscrowLifecycle::new(total_amount).expect("escrow should initialize"),
    ));
    let barrier = Arc::new(Barrier::new(2));
    let mut handles = Vec::new();

    for _ in 0..2 {
        let escrow = Arc::clone(&escrow);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            escrow
                .lock()
                .expect("escrow lock should acquire")
                .apply_transition_with_evidence(EscrowTransitionAction::RefundRemaining)
        }));
    }

    let mut success_count = 0_usize;
    let mut invalid_count = 0_usize;
    let mut error_reason_codes = Vec::new();

    for handle in handles {
        match handle.join().expect("escrow refund thread should join") {
            Ok(evidence) => {
                success_count += 1;
                assert_eq!(evidence.reason_code, "escrow_transition_allowed");
            }
            Err(error) => {
                invalid_count += 1;
                error_reason_codes.push(error.reason_code());
            }
        }
    }

    let escrow = escrow.lock().expect("escrow lock should acquire");
    (
        success_count,
        invalid_count,
        error_reason_codes,
        escrow.released_amount(),
        escrow.refunded_amount(),
        escrow.remaining_amount(),
    )
}

#[test]
fn task_accept_concurrency_has_single_winner_and_consistent_state() {
    let engine = Arc::new(Mutex::new(TaskOperationEngine::new()));
    engine
        .lock()
        .expect("engine lock should initialize")
        .submit(
            "task-concurrency-accept",
            "kamn:did:agent:requester-1",
            "Concurrency accept gate",
        )
        .expect("submit should succeed");

    let actors = [
        "kamn:did:agent:worker-1".to_owned(),
        "kamn:did:agent:worker-2".to_owned(),
    ];
    let barrier = Arc::new(Barrier::new(2));
    let mut handles = Vec::new();

    for actor in actors {
        let engine = Arc::clone(&engine);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            let result = engine
                .lock()
                .expect("engine lock should acquire")
                .accept("task-concurrency-accept", &actor);
            (actor, result)
        }));
    }

    let mut winning_actor: Option<String> = None;
    for handle in handles {
        let (actor, outcome) = handle.join().expect("task accept thread should join");
        match outcome {
            Ok(()) => {
                assert!(
                    winning_actor.is_none(),
                    "only one actor can win the concurrent accept race"
                );
                winning_actor = Some(actor);
            }
            Err(TaskOperationError::UnauthorizedActor {
                actor: rejected_actor,
                required,
            }) => {
                assert_eq!(rejected_actor, actor);
                assert_eq!(required, "unassigned_or_current_assignee");
            }
            Err(error) => panic!("unexpected task accept concurrency error: {error:?}"),
        }
    }

    let winning_actor = winning_actor.expect("exactly one actor must accept the task");
    let engine = engine.lock().expect("engine lock should acquire");
    let task = engine
        .task("task-concurrency-accept")
        .expect("task should exist");
    assert_eq!(task.lifecycle.state(), TaskState::Accepted);
    assert_eq!(task.assignee.as_deref(), Some(winning_actor.as_str()));
}

#[test]
fn task_submit_concurrency_rejects_duplicate_task_id_deterministically() {
    let engine = Arc::new(Mutex::new(TaskOperationEngine::new()));
    let submitters = [
        "kamn:did:agent:requester-1".to_owned(),
        "kamn:did:agent:requester-2".to_owned(),
    ];
    let barrier = Arc::new(Barrier::new(2));
    let mut handles = Vec::new();

    for requester in submitters {
        let engine = Arc::clone(&engine);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            let result = engine.lock().expect("engine lock should acquire").submit(
                "task-concurrency-submit",
                &requester,
                "Concurrent submit gate",
            );
            (requester, result)
        }));
    }

    let mut success_count = 0_usize;
    let mut duplicate_count = 0_usize;
    let mut winning_requester: Option<String> = None;

    for handle in handles {
        let (requester, outcome) = handle.join().expect("task submit thread should join");
        match outcome {
            Ok(()) => {
                success_count += 1;
                winning_requester = Some(requester);
            }
            Err(TaskOperationError::DuplicateTaskId(task_id)) => {
                duplicate_count += 1;
                assert_eq!(task_id, "task-concurrency-submit");
            }
            Err(error) => panic!("unexpected task submit concurrency error: {error:?}"),
        }
    }

    assert_eq!(success_count, 1);
    assert_eq!(duplicate_count, 1);

    let winning_requester =
        winning_requester.expect("one requester must win concurrent submit for a new task id");
    let engine = engine.lock().expect("engine lock should acquire");
    let task = engine
        .task("task-concurrency-submit")
        .expect("task should exist");
    assert_eq!(task.requester, winning_requester);
    assert_eq!(task.lifecycle.state(), TaskState::Submitted);
}

#[test]
fn peer_lifecycle_concurrency_preserves_transition_contract_across_phases() {
    let (success_by_phase, invalid_by_phase, final_state) =
        run_peer_lifecycle_race("peer-concurrency");
    assert_eq!(success_by_phase, [1, 1, 1]);
    assert_eq!(invalid_by_phase, [1, 1, 1]);
    assert_eq!(final_state, PeerLifecycleState::Disconnected);
}

#[test]
fn unit_concurrency_replay_fixture_entries_are_valid() {
    for contenders in concurrency_replay_fixture() {
        assert!(
            contenders.len() >= 3,
            "concurrency fixtures should include at least three contenders"
        );
        for contender in contenders {
            assert!(contender.starts_with("kamn:did:agent:"));
        }
    }
}

#[test]
fn functional_task_accept_concurrency_replay_fixture_preserves_invariants() {
    for (index, contenders) in concurrency_replay_fixture().iter().enumerate() {
        let task_id = format!("task-concurrency-fixture-{index}");
        let (success_count, unauthorized_count, winner) =
            run_task_accept_race(&task_id, contenders);
        assert_eq!(
            success_count, 1,
            "fixture {index} should produce exactly one successful accept"
        );
        assert_eq!(
            unauthorized_count,
            contenders.len() - 1,
            "fixture {index} should reject all non-winning accept attempts"
        );
        let winner = winner.expect("fixture should produce a winner");
        assert!(
            contenders.iter().any(|contender| contender == &winner),
            "winner should belong to replay fixture contender set"
        );
    }
}

#[test]
fn integration_peer_lifecycle_concurrency_replay_is_deterministic_across_rounds() {
    let mut baseline: Option<([usize; 3], [usize; 3], PeerLifecycleState)> = None;

    for round in 0..6 {
        let summary = run_peer_lifecycle_race(format!("peer-replay-{round}").as_str());
        if let Some(expected) = baseline {
            assert_eq!(
                summary, expected,
                "concurrency replay summary drifted in round {round}"
            );
        } else {
            baseline = Some(summary);
        }
    }
}

#[test]
fn functional_escrow_dispute_refund_concurrency_replay_fixture_preserves_terminal_snapshot() {
    let totals = [3_u128, 5, 8, 13];

    for total_amount in totals {
        let (success_count, invalid_count, released, refunded, remaining) =
            run_escrow_dispute_refund_race(total_amount);

        assert!(success_count >= 1);
        assert!(success_count <= 2);
        assert_eq!(success_count + invalid_count, 2);
        assert_eq!(released, 0);
        assert_eq!(refunded, total_amount);
        assert_eq!(remaining, 0);
    }
}

#[test]
fn integration_escrow_dispute_refund_concurrency_replay_is_deterministic_across_rounds() {
    let mut baseline: Option<(u128, u128, u128)> = None;

    for round in 0..24 {
        let (_success_count, _invalid_count, released, refunded, remaining) =
            run_escrow_dispute_refund_race(21);
        let summary = (released, refunded, remaining);

        if let Some(expected) = baseline {
            assert_eq!(
                summary, expected,
                "escrow dispute/refund race snapshot drifted in round {round}"
            );
        } else {
            baseline = Some(summary);
        }
    }
}

#[test]
fn regression_concurrency_accept_race_never_allows_multiple_winners() {
    // Regression: #844
    let contenders = [
        "kamn:did:agent:worker-reg-1",
        "kamn:did:agent:worker-reg-2",
        "kamn:did:agent:worker-reg-3",
        "kamn:did:agent:worker-reg-4",
    ];

    for round in 0..24 {
        let task_id = format!("task-concurrency-regression-{round}");
        let (success_count, unauthorized_count, winner) =
            run_task_accept_race(&task_id, &contenders);
        assert_eq!(
            success_count, 1,
            "round {round} should have exactly one winner"
        );
        assert_eq!(
            unauthorized_count,
            contenders.len() - 1,
            "round {round} should reject non-winning accept attempts"
        );
        assert!(
            winner.is_some(),
            "round {round} should return winning actor"
        );
    }
}

#[test]
fn regression_escrow_refund_race_never_allows_multiple_refund_winners() {
    // Regression: #904
    for round in 0..32 {
        let (success_count, invalid_count, error_reason_codes, released, refunded, remaining) =
            run_escrow_refund_race(34);
        assert_eq!(
            success_count, 1,
            "round {round} must have one refund winner"
        );
        assert_eq!(
            invalid_count, 1,
            "round {round} must reject exactly one replayed refund attempt"
        );
        assert_eq!(released, 0);
        assert_eq!(refunded, 34);
        assert_eq!(remaining, 0);
        assert_eq!(error_reason_codes, vec!["escrow_transition_invalid"]);
    }
}

#[test]
fn performance_concurrency_state_mutation_contract_lane_stays_within_budget() {
    let started = Instant::now();
    let contenders = [
        "kamn:did:agent:worker-perf-1",
        "kamn:did:agent:worker-perf-2",
        "kamn:did:agent:worker-perf-3",
    ];

    for round in 0..48 {
        let task_id = format!("task-concurrency-performance-{round}");
        let (success_count, unauthorized_count, _) = run_task_accept_race(&task_id, &contenders);
        assert_eq!(success_count, 1);
        assert_eq!(unauthorized_count, contenders.len() - 1);
    }

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 800,
        "concurrency state mutation contract lane exceeded budget: {elapsed_millis}ms"
    );
}

#[test]
fn performance_escrow_dispute_refund_concurrency_lane_stays_within_budget() {
    let started = Instant::now();

    for _ in 0..64 {
        let (success_count, invalid_count, released, refunded, remaining) =
            run_escrow_dispute_refund_race(55);
        assert!(success_count >= 1);
        assert!(success_count <= 2);
        assert_eq!(success_count + invalid_count, 2);
        assert_eq!(released, 0);
        assert_eq!(refunded, 55);
        assert_eq!(remaining, 0);
    }

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 600,
        "escrow dispute/refund concurrency contract lane exceeded budget: {elapsed_millis}ms"
    );
}

#[test]
fn performance_concurrency_state_mutation_deep_lane_stress() {
    if env::var("KAMN_KOLME_LOCAL_HEAVY").ok().as_deref() != Some("1") {
        eprintln!(
            "skipping deep-lane concurrency mutation stress test; set KAMN_KOLME_LOCAL_HEAVY=1 to run"
        );
        return;
    }

    let contenders = [
        "kamn:did:agent:worker-deep-1",
        "kamn:did:agent:worker-deep-2",
        "kamn:did:agent:worker-deep-3",
        "kamn:did:agent:worker-deep-4",
        "kamn:did:agent:worker-deep-5",
    ];

    for round in 0..512 {
        let task_id = format!("task-concurrency-deep-{round}");
        let (success_count, unauthorized_count, _) = run_task_accept_race(&task_id, &contenders);
        assert_eq!(success_count, 1);
        assert_eq!(unauthorized_count, contenders.len() - 1);
    }
}

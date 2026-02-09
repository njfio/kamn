use kamn_core::{
    PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, RuntimeLifecycleError,
    TaskOperationEngine, TaskOperationError, TaskState,
};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

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
    let lifecycle = Arc::new(Mutex::new(
        PeerLifecycle::new("peer-concurrency").expect("peer lifecycle should initialize"),
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

    assert_eq!(success_by_phase, [1, 1, 1]);
    assert_eq!(invalid_by_phase, [1, 1, 1]);
    assert_eq!(
        lifecycle
            .lock()
            .expect("peer lifecycle lock should acquire")
            .state(),
        PeerLifecycleState::Disconnected
    );
}

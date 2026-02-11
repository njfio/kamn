use kamn_core::{TaskOperationEngine, TaskOperationError, TaskOperationNoticeKind, TaskState};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

type TaskTerminalRaceSummary = (
    usize,
    usize,
    TaskState,
    Option<String>,
    Vec<TaskOperationNoticeKind>,
);

fn seed_in_progress_task(
    task_id: &str,
    requester: &str,
    assignee: &str,
) -> Arc<Mutex<TaskOperationEngine>> {
    let engine = Arc::new(Mutex::new(TaskOperationEngine::new()));
    let mut guard = engine.lock().expect("engine lock should initialize");
    guard
        .submit(task_id, requester, "Concurrent terminal transition guard")
        .expect("submit should succeed");
    guard
        .accept(task_id, assignee)
        .expect("accept should succeed");
    guard
        .start_work(task_id, assignee)
        .expect("start_work should succeed");
    drop(guard);
    engine
}

fn run_task_complete_delegate_race(
    task_id: &str,
    requester: &str,
    assignee: &str,
    delegatee: &str,
) -> TaskTerminalRaceSummary {
    let engine = seed_in_progress_task(task_id, requester, assignee);
    let barrier = Arc::new(Barrier::new(2));
    let mut handles = Vec::new();

    {
        let engine = Arc::clone(&engine);
        let barrier = Arc::clone(&barrier);
        let task_id = task_id.to_owned();
        let assignee = assignee.to_owned();
        handles.push(thread::spawn(move || {
            barrier.wait();
            engine
                .lock()
                .expect("engine lock should acquire")
                .complete(&task_id, &assignee)
        }));
    }

    {
        let engine = Arc::clone(&engine);
        let barrier = Arc::clone(&barrier);
        let task_id = task_id.to_owned();
        let assignee = assignee.to_owned();
        let delegatee = delegatee.to_owned();
        handles.push(thread::spawn(move || {
            barrier.wait();
            engine
                .lock()
                .expect("engine lock should acquire")
                .delegate(&task_id, &assignee, &delegatee)
        }));
    }

    let mut success_count = 0_usize;
    let mut lifecycle_error_count = 0_usize;
    for handle in handles {
        match handle
            .join()
            .expect("task terminal race thread should join")
        {
            Ok(()) => success_count += 1,
            Err(TaskOperationError::Lifecycle(error)) => {
                lifecycle_error_count += 1;
                assert!(
                    error.contains("invalid task transition") || error.contains("terminal state"),
                    "lifecycle error should stay in transition/terminal guard surface: {error}"
                );
            }
            Err(error) => panic!("unexpected task terminal race error: {error:?}"),
        }
    }

    let guard = engine.lock().expect("engine lock should acquire");
    let task = guard.task(task_id).expect("task should exist");
    (
        success_count,
        lifecycle_error_count,
        task.lifecycle.state(),
        task.assignee.clone(),
        guard.notices(task_id),
    )
}

#[test]
fn functional_task_complete_delegate_concurrency_race_preserves_terminal_snapshot() {
    let requester = "kamn:did:agent:requester-terminal";
    let assignee = "kamn:did:agent:worker-terminal";
    let delegatee = "kamn:did:agent:worker-shadow";

    let (success_count, lifecycle_error_count, final_state, assignee_after, notices) =
        run_task_complete_delegate_race(
            "task-terminal-race-functional",
            requester,
            assignee,
            delegatee,
        );

    assert_eq!(success_count, 1);
    assert_eq!(lifecycle_error_count, 1);
    assert_eq!(final_state, TaskState::Completed);
    assert_eq!(assignee_after.as_deref(), Some(assignee));
    assert_eq!(
        notices,
        vec![
            TaskOperationNoticeKind::Submitted,
            TaskOperationNoticeKind::Accepted,
            TaskOperationNoticeKind::Started,
            TaskOperationNoticeKind::Completed,
        ]
    );
}

#[test]
fn regression_task_complete_delegate_concurrency_replay_is_deterministic() {
    // Regression: #1527
    let requester = "kamn:did:agent:requester-replay";
    let assignee = "kamn:did:agent:worker-replay";
    let delegatee = "kamn:did:agent:worker-replay-shadow";

    let mut baseline: Option<TaskTerminalRaceSummary> = None;

    for round in 0..32 {
        let task_id = format!("task-terminal-race-replay-{round}");
        let summary =
            run_task_complete_delegate_race(task_id.as_str(), requester, assignee, delegatee);
        if let Some(expected) = baseline.as_ref() {
            assert_eq!(
                &summary, expected,
                "task terminal race summary drifted in round {round}"
            );
        } else {
            baseline = Some(summary);
        }
    }
}

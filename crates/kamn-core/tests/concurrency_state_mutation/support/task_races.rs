use kamn_core::{TaskOperationEngine, TaskOperationError, TaskState};
use std::sync::{Arc, Barrier, Mutex};
use std::thread::{self, JoinHandle};

type AcceptHandle = JoinHandle<(String, Result<(), TaskOperationError>)>;
type SubmitHandle = JoinHandle<(String, Result<(), TaskOperationError>)>;

pub(crate) fn run_task_accept_race(
    task_id: &str,
    contenders: &[&str],
) -> (usize, usize, Option<String>) {
    let engine = submitted_task_engine(task_id);
    let handles = spawn_accept_handles(&engine, task_id, contenders);
    let (success_count, unauthorized_count, winner) = collect_accept_outcomes(handles);
    assert_accepted_task_state(&engine, task_id, winner.as_deref());
    (success_count, unauthorized_count, winner)
}

pub(crate) fn run_task_submit_race(task_id: &str, requesters: &[&str]) -> (usize, usize, String) {
    let engine = Arc::new(Mutex::new(TaskOperationEngine::new()));
    let handles = spawn_submit_handles(&engine, task_id, requesters);
    let (success_count, duplicate_count, winner) = collect_submit_outcomes(handles);
    assert_submitted_task_state(&engine, task_id, &winner);
    (success_count, duplicate_count, winner)
}

fn submitted_task_engine(task_id: &str) -> Arc<Mutex<TaskOperationEngine>> {
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
    engine
}

fn spawn_accept_handles(
    engine: &Arc<Mutex<TaskOperationEngine>>,
    task_id: &str,
    contenders: &[&str],
) -> Vec<AcceptHandle> {
    let barrier = Arc::new(Barrier::new(contenders.len()));
    contenders
        .iter()
        .map(|contender| spawn_accept_handle(engine, &barrier, task_id, contender))
        .collect()
}

fn spawn_accept_handle(
    engine: &Arc<Mutex<TaskOperationEngine>>,
    barrier: &Arc<Barrier>,
    task_id: &str,
    contender: &str,
) -> AcceptHandle {
    let actor = contender.to_owned();
    let task_id = task_id.to_owned();
    let engine = Arc::clone(engine);
    let barrier = Arc::clone(barrier);
    thread::spawn(move || {
        barrier.wait();
        let result = engine
            .lock()
            .expect("engine lock should acquire")
            .accept(&task_id, &actor);
        (actor, result)
    })
}

fn collect_accept_outcomes(handles: Vec<AcceptHandle>) -> (usize, usize, Option<String>) {
    let mut success_count = 0;
    let mut unauthorized_count = 0;
    let mut winner = None;
    for handle in handles {
        let (actor, outcome) = handle.join().expect("task accept thread should join");
        match outcome {
            Ok(()) => record_accept_success(&mut success_count, &mut winner, actor),
            Err(TaskOperationError::UnauthorizedActor {
                actor: rejected,
                required,
            }) => {
                unauthorized_count += 1;
                assert_eq!(rejected, actor);
                assert_eq!(required, "unassigned_or_current_assignee");
            }
            Err(error) => panic!("unexpected task accept concurrency error: {error:?}"),
        }
    }
    (success_count, unauthorized_count, winner)
}

fn record_accept_success(success_count: &mut usize, winner: &mut Option<String>, actor: String) {
    *success_count += 1;
    *winner = Some(actor);
}

fn assert_accepted_task_state(
    engine: &Arc<Mutex<TaskOperationEngine>>,
    task_id: &str,
    winner: Option<&str>,
) {
    let engine = engine.lock().expect("engine lock should acquire");
    let task = engine.task(task_id).expect("task should exist");
    assert_eq!(task.lifecycle.state(), TaskState::Accepted);
    assert_eq!(task.assignee.as_deref(), winner);
}

fn spawn_submit_handles(
    engine: &Arc<Mutex<TaskOperationEngine>>,
    task_id: &str,
    requesters: &[&str],
) -> Vec<SubmitHandle> {
    let barrier = Arc::new(Barrier::new(requesters.len()));
    requesters
        .iter()
        .map(|requester| spawn_submit_handle(engine, &barrier, task_id, requester))
        .collect()
}

fn spawn_submit_handle(
    engine: &Arc<Mutex<TaskOperationEngine>>,
    barrier: &Arc<Barrier>,
    task_id: &str,
    requester: &str,
) -> SubmitHandle {
    let requester = requester.to_owned();
    let task_id = task_id.to_owned();
    let engine = Arc::clone(engine);
    let barrier = Arc::clone(barrier);
    thread::spawn(move || {
        barrier.wait();
        let result = engine.lock().expect("engine lock should acquire").submit(
            &task_id,
            &requester,
            "Concurrent submit gate",
        );
        (requester, result)
    })
}

fn collect_submit_outcomes(handles: Vec<SubmitHandle>) -> (usize, usize, String) {
    let mut success_count = 0;
    let mut duplicate_count = 0;
    let mut winner = None;
    for handle in handles {
        let (requester, outcome) = handle.join().expect("task submit thread should join");
        match outcome {
            Ok(()) => record_submit_success(&mut success_count, &mut winner, requester),
            Err(TaskOperationError::DuplicateTaskId(task_id)) => {
                duplicate_count += 1;
                assert_eq!(task_id, "task-concurrency-submit");
            }
            Err(error) => panic!("unexpected task submit concurrency error: {error:?}"),
        }
    }
    (
        success_count,
        duplicate_count,
        winner.expect("one requester must win"),
    )
}

fn record_submit_success(
    success_count: &mut usize,
    winner: &mut Option<String>,
    requester: String,
) {
    *success_count += 1;
    *winner = Some(requester);
}

fn assert_submitted_task_state(
    engine: &Arc<Mutex<TaskOperationEngine>>,
    task_id: &str,
    winner: &str,
) {
    let engine = engine.lock().expect("engine lock should acquire");
    let task = engine.task(task_id).expect("task should exist");
    assert_eq!(task.requester, winner);
    assert_eq!(task.lifecycle.state(), TaskState::Submitted);
}

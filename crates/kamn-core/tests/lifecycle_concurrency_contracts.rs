use kamn_core::{
    EscrowLifecycle, PeerLifecycle, PeerLifecycleEvent, TaskLifecycle, TaskState, TaskTransition,
};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

fn in_progress_task(task_id: &str) -> TaskLifecycle {
    let mut lifecycle =
        TaskLifecycle::new(task_id).expect("task lifecycle should initialize for concurrency test");
    lifecycle
        .transition(TaskTransition::Accept)
        .expect("task should accept");
    lifecycle
        .transition(TaskTransition::StartWork)
        .expect("task should move in-progress");
    lifecycle
}

#[test]
fn regression_task_complete_transition_stays_fail_closed_under_concurrency() {
    // Regression: #1597
    let thread_count = 8;
    let lifecycle = Arc::new(Mutex::new(in_progress_task("task-concurrency-1")));
    let barrier = Arc::new(Barrier::new(thread_count));

    let handles: Vec<_> = (0..thread_count)
        .map(|_| {
            let lifecycle = Arc::clone(&lifecycle);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                lifecycle
                    .lock()
                    .expect("task lifecycle mutex poisoned")
                    .transition(TaskTransition::Complete)
            })
        })
        .collect();

    let mut success_count = 0usize;
    let mut terminal_error_count = 0usize;

    for handle in handles {
        match handle.join().expect("task transition thread should join") {
            Ok(()) => success_count += 1,
            Err(error) => {
                if error.reason_code() == "task_transition_terminal_state" {
                    terminal_error_count += 1;
                } else {
                    panic!("unexpected task transition error: {error}");
                }
            }
        }
    }

    assert_eq!(success_count, 1);
    assert_eq!(terminal_error_count, thread_count - 1);
    assert_eq!(
        lifecycle
            .lock()
            .expect("task lifecycle mutex poisoned")
            .state(),
        TaskState::Completed
    );
}

#[test]
fn regression_escrow_dispute_transition_stays_fail_closed_under_concurrency() {
    // Regression: #1597
    let thread_count = 8;
    let lifecycle = Arc::new(Mutex::new(
        EscrowLifecycle::new(42).expect("escrow lifecycle should initialize for concurrency test"),
    ));
    let barrier = Arc::new(Barrier::new(thread_count));

    let handles: Vec<_> = (0..thread_count)
        .map(|_| {
            let lifecycle = Arc::clone(&lifecycle);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                lifecycle
                    .lock()
                    .expect("escrow lifecycle mutex poisoned")
                    .dispute()
            })
        })
        .collect();

    let mut success_count = 0usize;
    let mut invalid_error_count = 0usize;

    for handle in handles {
        match handle.join().expect("escrow transition thread should join") {
            Ok(()) => success_count += 1,
            Err(error) => {
                if error.reason_code() == "escrow_transition_invalid" {
                    invalid_error_count += 1;
                } else {
                    panic!("unexpected escrow transition error: {error}");
                }
            }
        }
    }

    assert_eq!(success_count, 1);
    assert_eq!(invalid_error_count, thread_count - 1);
}

#[test]
fn regression_peer_handshake_transition_rejects_parallel_invalid_edges() {
    // Regression: #1597
    let thread_count = 8;
    let mut seed_lifecycle =
        PeerLifecycle::new("peer-concurrency-1").expect("peer lifecycle should initialize");
    seed_lifecycle
        .transition(PeerLifecycleEvent::StartConnect)
        .expect("peer should enter connecting state");

    let lifecycle = Arc::new(Mutex::new(seed_lifecycle));
    let barrier = Arc::new(Barrier::new(thread_count));

    let handles: Vec<_> = (0..thread_count)
        .map(|_| {
            let lifecycle = Arc::clone(&lifecycle);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                lifecycle
                    .lock()
                    .expect("peer lifecycle mutex poisoned")
                    .transition(PeerLifecycleEvent::HandshakeSucceeded)
            })
        })
        .collect();

    let mut success_count = 0usize;
    let mut invalid_error_count = 0usize;

    for handle in handles {
        match handle.join().expect("peer transition thread should join") {
            Ok(_) => success_count += 1,
            Err(error) => {
                if error.reason_code() == "runtime_peer_transition_invalid" {
                    invalid_error_count += 1;
                } else {
                    panic!("unexpected peer lifecycle transition error: {error}");
                }
            }
        }
    }

    assert_eq!(success_count, 1);
    assert_eq!(invalid_error_count, thread_count - 1);
}

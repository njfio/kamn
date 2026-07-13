use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

static ACTIVE_GATE: Mutex<Option<Arc<GateState>>> = Mutex::new(None);

struct GateState {
    target_path: String,
    state: Mutex<GateProgress>,
    changed: Condvar,
}

#[derive(Default)]
struct GateProgress {
    arrivals: usize,
    released: bool,
}

pub(crate) struct TestPostAuthGate {
    state: Arc<GateState>,
}

pub(crate) fn set_test_post_auth_gate(target_path: &str) -> TestPostAuthGate {
    let state = Arc::new(GateState {
        target_path: target_path.to_owned(),
        state: Mutex::new(GateProgress::default()),
        changed: Condvar::new(),
    });
    *active_gate()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(state.clone());
    TestPostAuthGate { state }
}

pub(crate) fn wait_at_test_post_auth_gate(request_path: &str) {
    let state = active_gate()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    if let Some(state) = state.filter(|gate| gate.target_path == request_path) {
        state.arrive_and_wait();
    }
}

impl TestPostAuthGate {
    pub(crate) fn expect_arrivals(&self, expected: usize) {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut progress = self.lock_progress();
        while progress.arrivals < expected {
            let timeout = deadline.saturating_duration_since(Instant::now());
            let waited = self
                .state
                .changed
                .wait_timeout(progress, timeout)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            progress = waited.0;
            assert!(
                !waited.1.timed_out(),
                "post-auth gate expected {expected} arrivals"
            );
        }
    }

    pub(crate) fn release(&self) {
        let mut progress = self.lock_progress();
        progress.released = true;
        self.state.changed.notify_all();
    }

    fn lock_progress(&self) -> std::sync::MutexGuard<'_, GateProgress> {
        self.state
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl GateState {
    fn arrive_and_wait(&self) {
        let mut progress = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        progress.arrivals += 1;
        self.changed.notify_all();
        while !progress.released {
            progress = self
                .changed
                .wait(progress)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }
}

impl Drop for TestPostAuthGate {
    fn drop(&mut self) {
        self.release();
        *active_gate()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }
}

fn active_gate() -> &'static Mutex<Option<Arc<GateState>>> {
    &ACTIVE_GATE
}

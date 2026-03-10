use kamn_core::{PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, RuntimeLifecycleError};
use std::sync::{Arc, Barrier, Mutex};
use std::thread::{self, JoinHandle};

type PeerOutcome = (PeerLifecycleEvent, Result<PeerLifecycleState, RuntimeLifecycleError>);
type PeerHandle = JoinHandle<Vec<PeerOutcome>>;

pub(crate) fn run_peer_lifecycle_race(peer_id: &str) -> ([usize; 3], [usize; 3], PeerLifecycleState) {
    let lifecycle = Arc::new(Mutex::new(PeerLifecycle::new(peer_id).expect("peer lifecycle should initialize")));
    let handles = spawn_peer_handles(&lifecycle);
    let (success_by_phase, invalid_by_phase) = collect_peer_outcomes(handles);
    let final_state = lifecycle.lock().expect("peer lifecycle lock should acquire").state();
    (success_by_phase, invalid_by_phase, final_state)
}

fn spawn_peer_handles(lifecycle: &Arc<Mutex<PeerLifecycle>>) -> Vec<PeerHandle> {
    let barrier = Arc::new(Barrier::new(2));
    (0..2)
        .map(|_| spawn_peer_handle(lifecycle, &barrier))
        .collect()
}

fn spawn_peer_handle(lifecycle: &Arc<Mutex<PeerLifecycle>>, barrier: &Arc<Barrier>) -> PeerHandle {
    let lifecycle = Arc::clone(lifecycle);
    let barrier = Arc::clone(barrier);
    thread::spawn(move || run_peer_sequence(&lifecycle, &barrier))
}

fn run_peer_sequence(lifecycle: &Arc<Mutex<PeerLifecycle>>, barrier: &Arc<Barrier>) -> Vec<PeerOutcome> {
    let mut outcomes = Vec::new();
    for event in [
        PeerLifecycleEvent::StartConnect,
        PeerLifecycleEvent::HandshakeSucceeded,
        PeerLifecycleEvent::Disconnect,
    ] {
        barrier.wait();
        let outcome = lifecycle.lock().expect("peer lifecycle lock should acquire").transition(event);
        outcomes.push((event, outcome));
        barrier.wait();
    }
    outcomes
}

fn collect_peer_outcomes(handles: Vec<PeerHandle>) -> ([usize; 3], [usize; 3]) {
    let mut success_by_phase = [0; 3];
    let mut invalid_by_phase = [0; 3];
    for handle in handles {
        let outcomes = handle.join().expect("peer lifecycle thread should join");
        for (phase_index, outcome) in outcomes.into_iter().enumerate() {
            tally_peer_outcome(phase_index, outcome, &mut success_by_phase, &mut invalid_by_phase);
        }
    }
    (success_by_phase, invalid_by_phase)
}

fn tally_peer_outcome(
    phase_index: usize,
    (event, outcome): PeerOutcome,
    success_by_phase: &mut [usize; 3],
    invalid_by_phase: &mut [usize; 3],
) {
    match outcome {
        Ok(next_state) => record_peer_success(phase_index, next_state, success_by_phase),
        Err(RuntimeLifecycleError::InvalidTransition { from, event: rejected }) => {
            invalid_by_phase[phase_index] += 1;
            assert_eq!(rejected, event);
            assert_invalid_transition_state(phase_index, from);
        }
        Err(RuntimeLifecycleError::InvalidPeerId) => panic!("peer id is fixed and valid in concurrency lane"),
    }
}

fn record_peer_success(
    phase_index: usize,
    next_state: PeerLifecycleState,
    success_by_phase: &mut [usize; 3],
) {
    success_by_phase[phase_index] += 1;
    match (phase_index, next_state) {
        (0, PeerLifecycleState::Connecting)
        | (1, PeerLifecycleState::Active)
        | (2, PeerLifecycleState::Disconnected) => {}
        _ => panic!("unexpected successful transition in phase {phase_index}: {next_state:?}"),
    }
}

fn assert_invalid_transition_state(phase_index: usize, from: PeerLifecycleState) {
    match phase_index {
        0 => assert_eq!(from, PeerLifecycleState::Connecting),
        1 => assert_eq!(from, PeerLifecycleState::Active),
        2 => assert_eq!(from, PeerLifecycleState::Disconnected),
        _ => unreachable!("phase index is bounded by static phases"),
    }
}

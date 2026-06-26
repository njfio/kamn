use kamn_core::{
    EscrowLifecycle, EscrowLifecycleError, EscrowTransitionAction, EscrowTransitionEvidence,
};
use std::sync::{Arc, Barrier, Mutex};
use std::thread::{self, JoinHandle};

type EscrowHandle = JoinHandle<Result<EscrowTransitionEvidence, EscrowLifecycleError>>;

pub(crate) fn run_escrow_dispute_refund_race(
    total_amount: u128,
) -> (usize, usize, u128, u128, u128) {
    let escrow = Arc::new(Mutex::new(
        EscrowLifecycle::new(total_amount).expect("escrow should initialize"),
    ));
    let handles = spawn_action_handles(
        &escrow,
        [
            EscrowTransitionAction::Dispute,
            EscrowTransitionAction::RefundRemaining,
        ],
    );
    let (success_count, invalid_count) = collect_transition_counts(handles);
    let escrow = escrow.lock().expect("escrow lock should acquire");
    (
        success_count,
        invalid_count,
        escrow.released_amount(),
        escrow.refunded_amount(),
        escrow.remaining_amount(),
    )
}

pub(crate) fn run_escrow_refund_race(
    total_amount: u128,
) -> (usize, usize, Vec<&'static str>, u128, u128, u128) {
    let escrow = Arc::new(Mutex::new(
        EscrowLifecycle::new(total_amount).expect("escrow should initialize"),
    ));
    let handles = spawn_action_handles(
        &escrow,
        [
            EscrowTransitionAction::RefundRemaining,
            EscrowTransitionAction::RefundRemaining,
        ],
    );
    let (success_count, invalid_count, error_reason_codes) = collect_refund_outcomes(handles);
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

fn spawn_action_handles(
    escrow: &Arc<Mutex<EscrowLifecycle>>,
    actions: [EscrowTransitionAction; 2],
) -> Vec<EscrowHandle> {
    let barrier = Arc::new(Barrier::new(actions.len()));
    actions
        .into_iter()
        .map(|action| spawn_action_handle(escrow, &barrier, action))
        .collect()
}

fn spawn_action_handle(
    escrow: &Arc<Mutex<EscrowLifecycle>>,
    barrier: &Arc<Barrier>,
    action: EscrowTransitionAction,
) -> EscrowHandle {
    let escrow = Arc::clone(escrow);
    let barrier = Arc::clone(barrier);
    thread::spawn(move || {
        barrier.wait();
        escrow
            .lock()
            .expect("escrow lock should acquire")
            .apply_transition_with_evidence(action)
    })
}

fn collect_transition_counts(handles: Vec<EscrowHandle>) -> (usize, usize) {
    let mut success_count = 0;
    let mut invalid_count = 0;
    for handle in handles {
        match handle
            .join()
            .expect("escrow dispute/refund thread should join")
        {
            Ok(evidence) => record_escrow_success(&mut success_count, evidence),
            Err(error) => record_escrow_invalid(&mut invalid_count, error),
        }
    }
    (success_count, invalid_count)
}

fn collect_refund_outcomes(handles: Vec<EscrowHandle>) -> (usize, usize, Vec<&'static str>) {
    let mut success_count = 0;
    let mut invalid_count = 0;
    let mut reason_codes = Vec::new();
    for handle in handles {
        match handle.join().expect("escrow refund thread should join") {
            Ok(evidence) => record_escrow_success(&mut success_count, evidence),
            Err(error) => record_refund_invalid(&mut invalid_count, &mut reason_codes, error),
        }
    }
    (success_count, invalid_count, reason_codes)
}

fn record_escrow_success(success_count: &mut usize, evidence: EscrowTransitionEvidence) {
    *success_count += 1;
    assert_eq!(evidence.reason_code, "escrow_transition_allowed");
}

fn record_escrow_invalid(invalid_count: &mut usize, error: EscrowLifecycleError) {
    *invalid_count += 1;
    assert_eq!(error.reason_code(), "escrow_transition_invalid");
}

fn record_refund_invalid(
    invalid_count: &mut usize,
    reason_codes: &mut Vec<&'static str>,
    error: EscrowLifecycleError,
) {
    *invalid_count += 1;
    reason_codes.push(error.reason_code());
}

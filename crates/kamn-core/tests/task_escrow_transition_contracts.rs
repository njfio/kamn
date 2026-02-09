use kamn_core::{
    EscrowLifecycle, EscrowReceiptFinality, EscrowSettlementAction, EscrowSettlementOutcome,
    EscrowStatus, EscrowTransitionAction, TaskLifecycle, TaskState, TaskTransition,
};
use std::time::Instant;

#[test]
fn unit_task_transition_error_reason_codes_are_deterministic() {
    let empty_error = TaskLifecycle::new("").expect_err("empty task id must fail");
    assert_eq!(empty_error.reason_code(), "task_id_empty");

    let mut lifecycle = TaskLifecycle::new("task-reason-invalid")
        .expect("task lifecycle should initialize for invalid-transition reason code");
    let invalid_error = lifecycle
        .transition(TaskTransition::Complete)
        .expect_err("submitted->complete must be rejected");
    assert_eq!(invalid_error.reason_code(), "task_transition_invalid_edge");

    lifecycle
        .transition(TaskTransition::Accept)
        .expect("accept should succeed");
    lifecycle
        .transition(TaskTransition::StartWork)
        .expect("start work should succeed");
    lifecycle
        .transition(TaskTransition::Complete)
        .expect("complete should succeed");
    let terminal_error = lifecycle
        .transition(TaskTransition::Fail)
        .expect_err("terminal state must reject follow-up transitions");
    assert_eq!(
        terminal_error.reason_code(),
        "task_transition_terminal_state"
    );
}

#[test]
fn functional_task_transition_with_evidence_reports_allowed_reason_code() {
    let mut lifecycle = TaskLifecycle::new("task-evidence-flow")
        .expect("task lifecycle should initialize for evidence flow");

    let accept_evidence = lifecycle
        .transition_with_evidence(TaskTransition::Accept)
        .expect("submitted->accept should succeed");
    assert_eq!(accept_evidence.from, TaskState::Submitted);
    assert_eq!(accept_evidence.transition, TaskTransition::Accept);
    assert_eq!(accept_evidence.to, TaskState::Accepted);
    assert_eq!(accept_evidence.reason_code, "task_transition_allowed");

    let start_evidence = lifecycle
        .transition_with_evidence(TaskTransition::StartWork)
        .expect("accepted->in_progress should succeed");
    assert_eq!(start_evidence.from, TaskState::Accepted);
    assert_eq!(start_evidence.to, TaskState::InProgress);
    assert_eq!(start_evidence.reason_code, "task_transition_allowed");
}

#[test]
fn unit_escrow_transition_error_reason_codes_are_deterministic() {
    let mut escrow = EscrowLifecycle::new(25).expect("escrow should initialize");
    let zero_release_error = escrow.release(0).expect_err("zero release must fail");
    assert_eq!(zero_release_error.reason_code(), "escrow_amount_zero");

    escrow.release(25).expect("release should succeed");
    let dispute_after_release_error = escrow
        .dispute()
        .expect_err("released escrow must reject dispute");
    assert_eq!(
        dispute_after_release_error.reason_code(),
        "escrow_transition_invalid"
    );
}

#[test]
fn functional_escrow_transition_with_evidence_reports_allowed_reason_code() {
    let mut escrow = EscrowLifecycle::new(40).expect("escrow should initialize");
    let release_evidence = escrow
        .apply_transition_with_evidence(EscrowTransitionAction::Release { amount: 15 })
        .expect("release should succeed");

    assert_eq!(release_evidence.reason_code, "escrow_transition_allowed");
    assert_eq!(release_evidence.from, EscrowStatus::Funded);
    assert_eq!(
        release_evidence.action,
        EscrowTransitionAction::Release { amount: 15 }
    );
    assert!(matches!(
        release_evidence.to,
        EscrowStatus::PartiallyReleased {
            released: 15,
            remaining: 25
        }
    ));
}

#[test]
fn integration_task_completion_and_escrow_settlement_emit_expected_evidence() {
    let mut lifecycle =
        TaskLifecycle::new("task-escrow-integration").expect("task lifecycle should initialize");
    lifecycle
        .transition_with_evidence(TaskTransition::Accept)
        .expect("accept should succeed");
    lifecycle
        .transition_with_evidence(TaskTransition::StartWork)
        .expect("start work should succeed");
    let completion_evidence = lifecycle
        .transition_with_evidence(TaskTransition::Complete)
        .expect("complete should succeed");
    assert_eq!(completion_evidence.to, TaskState::Completed);
    assert_eq!(completion_evidence.reason_code, "task_transition_allowed");

    let mut escrow = EscrowLifecycle::new(80).expect("escrow should initialize");
    let settlement_outcome = escrow
        .reconcile_receipt_finality(
            "receipt-escrow-integration",
            EscrowReceiptFinality::Final,
            EscrowSettlementAction::Release { amount: 80 },
        )
        .expect("final receipt should settle release action");
    assert!(matches!(
        settlement_outcome,
        EscrowSettlementOutcome::Settled {
            status: EscrowStatus::Released
        }
    ));
    assert_eq!(
        settlement_outcome.reason_code(),
        "escrow_settlement_finalized"
    );
}

#[test]
fn regression_rejects_dispute_after_release_with_stable_reason_code() {
    // Regression: #903
    let mut escrow = EscrowLifecycle::new(10).expect("escrow should initialize");
    escrow
        .apply_transition_with_evidence(EscrowTransitionAction::Release { amount: 10 })
        .expect("release should succeed");

    let error = escrow
        .apply_transition_with_evidence(EscrowTransitionAction::Dispute)
        .expect_err("released escrow must reject dispute action");
    assert_eq!(error.reason_code(), "escrow_transition_invalid");
}

#[test]
fn performance_transition_evidence_checks_stay_within_budget() {
    let start = Instant::now();
    for _ in 0..256 {
        let mut task = TaskLifecycle::new("task-perf").expect("task lifecycle should initialize");
        task.transition_with_evidence(TaskTransition::Accept)
            .expect("accept should succeed");
        task.transition_with_evidence(TaskTransition::StartWork)
            .expect("start should succeed");
        task.transition_with_evidence(TaskTransition::Complete)
            .expect("complete should succeed");

        let mut escrow = EscrowLifecycle::new(12).expect("escrow should initialize");
        escrow
            .apply_transition_with_evidence(EscrowTransitionAction::Release { amount: 12 })
            .expect("release should succeed");
    }

    let elapsed_millis = start.elapsed().as_millis();
    assert!(
        elapsed_millis < 900,
        "task/escrow transition evidence contract lane exceeded budget: {elapsed_millis}ms"
    );
}

use kamn_core::{
    EscrowLifecycle, EscrowLifecycleError, EscrowStatus, EscrowTransitionAction, TaskLifecycle,
    TaskState, TaskTransition,
};
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
enum DisputeRefundAction {
    ReleaseOne,
    Dispute,
    ResolveHalfSplit,
    RefundRemaining,
}

const DISPUTE_REFUND_ACTIONS: [DisputeRefundAction; 4] = [
    DisputeRefundAction::ReleaseOne,
    DisputeRefundAction::Dispute,
    DisputeRefundAction::ResolveHalfSplit,
    DisputeRefundAction::RefundRemaining,
];

fn for_each_dispute_refund_sequence(max_len: usize, mut f: impl FnMut(&[DisputeRefundAction])) {
    fn recurse(
        target_len: usize,
        current: &mut Vec<DisputeRefundAction>,
        f: &mut impl FnMut(&[DisputeRefundAction]),
    ) {
        if current.len() == target_len {
            f(current.as_slice());
            return;
        }

        for action in DISPUTE_REFUND_ACTIONS {
            current.push(action);
            recurse(target_len, current, f);
            current.pop();
        }
    }

    let mut current = Vec::new();
    for len in 1..=max_len {
        recurse(len, &mut current, &mut f);
    }
}

fn assert_escrow_amount_contract(escrow: &EscrowLifecycle, total_amount: u128) {
    let released = escrow.released_amount();
    let refunded = escrow.refunded_amount();
    let remaining = escrow.remaining_amount();
    assert_eq!(released + refunded + remaining, total_amount);
    assert!(released <= total_amount);
    assert!(refunded <= total_amount);
    assert!(remaining <= total_amount);
}

fn apply_action(
    escrow: &mut EscrowLifecycle,
    action: DisputeRefundAction,
) -> Result<&'static str, EscrowLifecycleError> {
    let transition_action = match action {
        DisputeRefundAction::ReleaseOne => EscrowTransitionAction::Release { amount: 1 },
        DisputeRefundAction::Dispute => EscrowTransitionAction::Dispute,
        DisputeRefundAction::ResolveHalfSplit => {
            let remaining = escrow.remaining_amount();
            let release_to_payee = remaining / 2;
            let refund_to_payer = remaining.saturating_sub(release_to_payee);
            EscrowTransitionAction::Resolve {
                release_to_payee,
                refund_to_payer,
            }
        }
        DisputeRefundAction::RefundRemaining => EscrowTransitionAction::RefundRemaining,
    };

    let evidence = escrow.apply_transition_with_evidence(transition_action)?;
    assert_eq!(evidence.reason_code, "escrow_transition_allowed");
    Ok(evidence.reason_code)
}

fn run_sequence_trace(
    total_amount: u128,
    sequence: &[DisputeRefundAction],
) -> (EscrowStatus, u128, u128, u128, Vec<String>) {
    let mut escrow = EscrowLifecycle::new(total_amount).expect("escrow should initialize");
    let mut trace = Vec::new();

    for action in sequence {
        let before_status = escrow.status();
        let before_released = escrow.released_amount();
        let before_refunded = escrow.refunded_amount();
        let before_remaining = escrow.remaining_amount();

        match apply_action(&mut escrow, *action) {
            Ok(reason_code) => {
                trace.push(format!(
                    "ok:{reason_code}:{before_status:?}->{:?}",
                    escrow.status()
                ));
            }
            Err(error) => {
                trace.push(format!("err:{}:{before_status:?}", error.reason_code()));
                assert_eq!(escrow.status(), before_status);
                assert_eq!(escrow.released_amount(), before_released);
                assert_eq!(escrow.refunded_amount(), before_refunded);
                assert_eq!(escrow.remaining_amount(), before_remaining);
            }
        }

        assert_escrow_amount_contract(&escrow, total_amount);
    }

    (
        escrow.status(),
        escrow.released_amount(),
        escrow.refunded_amount(),
        escrow.remaining_amount(),
        trace,
    )
}

#[test]
fn unit_dispute_refund_error_reason_codes_remain_stable() {
    let mut refunded = EscrowLifecycle::new(10).expect("escrow should initialize");
    refunded
        .refund_remaining()
        .expect("funded->refunded transition should succeed");
    let dispute_after_refund = refunded.dispute().expect_err("dispute replay should fail");
    assert_eq!(
        dispute_after_refund.reason_code(),
        "escrow_transition_invalid"
    );

    let mut disputed = EscrowLifecycle::new(11).expect("escrow should initialize");
    disputed
        .dispute()
        .expect("funded->disputed transition should succeed");
    let mismatch_error = disputed
        .resolve(3, 3)
        .expect_err("resolution split mismatch should fail");
    assert_eq!(mismatch_error.reason_code(), "escrow_resolution_mismatch");
}

#[test]
fn functional_dispute_then_refund_remaining_emits_allowed_evidence() {
    let mut escrow = EscrowLifecycle::new(13).expect("escrow should initialize");

    let dispute_evidence = escrow
        .apply_transition_with_evidence(EscrowTransitionAction::Dispute)
        .expect("funded->disputed transition should succeed");
    assert_eq!(dispute_evidence.reason_code, "escrow_transition_allowed");
    assert_eq!(dispute_evidence.from, EscrowStatus::Funded);
    assert_eq!(dispute_evidence.to, EscrowStatus::Disputed);

    let refund_evidence = escrow
        .apply_transition_with_evidence(EscrowTransitionAction::RefundRemaining)
        .expect("disputed->refunded transition should succeed");
    assert_eq!(refund_evidence.reason_code, "escrow_transition_allowed");
    assert_eq!(refund_evidence.from, EscrowStatus::Disputed);
    assert_eq!(refund_evidence.to, EscrowStatus::Refunded);
    assert_eq!(escrow.refunded_amount(), 13);
    assert_eq!(escrow.remaining_amount(), 0);
}

#[test]
fn functional_property_dispute_refund_sequences_preserve_contracts() {
    let totals = [3_u128, 5, 8];

    for total_amount in totals {
        for_each_dispute_refund_sequence(4, |sequence| {
            let (_status, _released, _refunded, _remaining, _trace) =
                run_sequence_trace(total_amount, sequence);
        });
    }
}

#[test]
fn integration_dispute_refund_replay_traces_are_deterministic() {
    let totals = [3_u128, 5, 8];

    for total_amount in totals {
        for_each_dispute_refund_sequence(3, |sequence| {
            let first = run_sequence_trace(total_amount, sequence);
            let second = run_sequence_trace(total_amount, sequence);
            assert_eq!(first, second);
        });
    }
}

#[test]
fn integration_task_completion_and_refund_resolution_contracts_stay_coherent() {
    let mut lifecycle =
        TaskLifecycle::new("task-904-integration").expect("task lifecycle should initialize");
    let accept = lifecycle
        .transition_with_evidence(TaskTransition::Accept)
        .expect("submitted->accepted should succeed");
    let start = lifecycle
        .transition_with_evidence(TaskTransition::StartWork)
        .expect("accepted->in_progress should succeed");
    let complete = lifecycle
        .transition_with_evidence(TaskTransition::Complete)
        .expect("in_progress->completed should succeed");

    assert_eq!(accept.reason_code, "task_transition_allowed");
    assert_eq!(start.reason_code, "task_transition_allowed");
    assert_eq!(complete.reason_code, "task_transition_allowed");
    assert_eq!(lifecycle.state(), TaskState::Completed);

    let mut escrow = EscrowLifecycle::new(17).expect("escrow should initialize");
    escrow
        .apply_transition_with_evidence(EscrowTransitionAction::Dispute)
        .expect("funded->disputed should succeed");
    escrow
        .apply_transition_with_evidence(EscrowTransitionAction::Resolve {
            release_to_payee: 8,
            refund_to_payer: 9,
        })
        .expect("disputed->resolved should succeed");

    assert_eq!(
        escrow.status(),
        EscrowStatus::Resolved {
            released_total: 8,
            refunded_total: 9,
        }
    );
    assert_eq!(escrow.remaining_amount(), 0);
}

#[test]
fn regression_replay_dispute_after_refund_fails_closed_with_reason_code() {
    // Regression: #904
    let mut escrow = EscrowLifecycle::new(9).expect("escrow should initialize");
    escrow
        .apply_transition_with_evidence(EscrowTransitionAction::RefundRemaining)
        .expect("initial refund should succeed");

    let replay_error = escrow
        .apply_transition_with_evidence(EscrowTransitionAction::Dispute)
        .expect_err("dispute replay after refund must fail");
    assert_eq!(replay_error.reason_code(), "escrow_transition_invalid");
    assert_eq!(escrow.status(), EscrowStatus::Refunded);
    assert_eq!(escrow.refunded_amount(), 9);
    assert_eq!(escrow.remaining_amount(), 0);
}

#[test]
fn regression_dispute_resolution_split_mismatch_preserves_disputed_state() {
    // Regression: #904
    let mut escrow = EscrowLifecycle::new(12).expect("escrow should initialize");
    escrow
        .apply_transition_with_evidence(EscrowTransitionAction::Dispute)
        .expect("dispute transition should succeed");

    let before_status = escrow.status();
    let before_released = escrow.released_amount();
    let before_refunded = escrow.refunded_amount();
    let before_remaining = escrow.remaining_amount();

    let mismatch_error = escrow
        .apply_transition_with_evidence(EscrowTransitionAction::Resolve {
            release_to_payee: 6,
            refund_to_payer: 3,
        })
        .expect_err("invalid split should fail closed");
    assert_eq!(mismatch_error.reason_code(), "escrow_resolution_mismatch");
    assert_eq!(escrow.status(), before_status);
    assert_eq!(escrow.released_amount(), before_released);
    assert_eq!(escrow.refunded_amount(), before_refunded);
    assert_eq!(escrow.remaining_amount(), before_remaining);
}

#[test]
fn performance_dispute_refund_property_contract_lane_stays_within_budget() {
    let started = Instant::now();
    let totals = [2_u128, 3, 5];

    for total_amount in totals {
        for_each_dispute_refund_sequence(4, |sequence| {
            let _ = run_sequence_trace(total_amount, sequence);
        });
    }

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 700,
        "dispute/refund property contract lane exceeded budget: {elapsed_millis}ms"
    );
}

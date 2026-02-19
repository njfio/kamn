use kamn_core::{
    EscrowLifecycle, EscrowStatus, PaymentConfirm, PaymentOffer, TaskOperationEngine,
    TaskPaymentError, TaskPaymentWorkflow, TaskState,
};

fn completed_task_engine(task_id: &str) -> TaskOperationEngine {
    let mut tasks = TaskOperationEngine::new();
    tasks
        .submit(
            task_id,
            "kamn:did:agent:requester-1",
            "Deliver integration work",
        )
        .expect("submit should succeed");
    tasks
        .accept(task_id, "kamn:did:agent:worker-1")
        .expect("accept should succeed");
    tasks
        .start_work(task_id, "kamn:did:agent:worker-1")
        .expect("start work should succeed");
    tasks
        .complete(task_id, "kamn:did:agent:worker-1")
        .expect("complete should succeed");
    tasks
}

#[test]
fn offer_rejects_task_not_completed() {
    let mut workflow = TaskPaymentWorkflow::new();
    let mut tasks = TaskOperationEngine::new();
    tasks
        .submit(
            "task-pay-1",
            "kamn:did:agent:requester-1",
            "Prepare payment terms",
        )
        .expect("submit should succeed");
    let escrow = EscrowLifecycle::new(100).expect("escrow should initialize");

    let offer = PaymentOffer {
        task_id: "task-pay-1".to_owned(),
        escrow_id: "escrow-1".to_owned(),
        payer_did: "kamn:did:agent:requester-1".to_owned(),
        payee_did: "kamn:did:agent:worker-1".to_owned(),
        amount: 60,
    };

    assert_eq!(
        workflow.submit_offer(offer, &tasks, &escrow),
        Err(TaskPaymentError::TaskNotCompleted {
            task_id: "task-pay-1".to_owned(),
            state: TaskState::Submitted,
        })
    );
}

#[test]
fn completed_task_offer_confirm_releases_escrow() {
    let mut workflow = TaskPaymentWorkflow::new();
    let tasks = completed_task_engine("task-pay-2");
    let mut escrow = EscrowLifecycle::new(100).expect("escrow should initialize");

    workflow
        .submit_offer(
            PaymentOffer {
                task_id: "task-pay-2".to_owned(),
                escrow_id: "escrow-2".to_owned(),
                payer_did: "kamn:did:agent:requester-1".to_owned(),
                payee_did: "kamn:did:agent:worker-1".to_owned(),
                amount: 60,
            },
            &tasks,
            &escrow,
        )
        .expect("offer should be accepted");
    workflow
        .confirm_offer(
            PaymentConfirm {
                task_id: "task-pay-2".to_owned(),
                escrow_id: "escrow-2".to_owned(),
                confirmer_did: "kamn:did:agent:requester-1".to_owned(),
            },
            &mut escrow,
        )
        .expect("confirmation should release escrow");

    assert_eq!(
        escrow.status(),
        EscrowStatus::PartiallyReleased {
            released: 60,
            remaining: 40,
        }
    );
}

#[test]
fn regression_rejects_offer_with_payer_mismatch() {
    // Regression: #558
    let mut workflow = TaskPaymentWorkflow::new();
    let tasks = completed_task_engine("task-pay-payer-mismatch");
    let escrow = EscrowLifecycle::new(100).expect("escrow should initialize");

    let result = workflow.submit_offer(
        PaymentOffer {
            task_id: "task-pay-payer-mismatch".to_owned(),
            escrow_id: "escrow-payer-mismatch".to_owned(),
            payer_did: "kamn:did:agent:observer-9".to_owned(),
            payee_did: "kamn:did:agent:worker-1".to_owned(),
            amount: 60,
        },
        &tasks,
        &escrow,
    );

    assert_eq!(
        result,
        Err(TaskPaymentError::PayerRequesterMismatch {
            expected: "kamn:did:agent:requester-1".to_owned(),
            found: "kamn:did:agent:observer-9".to_owned(),
        })
    );
}

#[test]
fn regression_rejects_offer_with_payee_mismatch() {
    // Regression: #558
    let mut workflow = TaskPaymentWorkflow::new();
    let tasks = completed_task_engine("task-pay-payee-mismatch");
    let escrow = EscrowLifecycle::new(100).expect("escrow should initialize");

    let result = workflow.submit_offer(
        PaymentOffer {
            task_id: "task-pay-payee-mismatch".to_owned(),
            escrow_id: "escrow-payee-mismatch".to_owned(),
            payer_did: "kamn:did:agent:requester-1".to_owned(),
            payee_did: "kamn:did:agent:observer-9".to_owned(),
            amount: 60,
        },
        &tasks,
        &escrow,
    );

    assert_eq!(
        result,
        Err(TaskPaymentError::PayeeAssigneeMismatch {
            expected: "kamn:did:agent:worker-1".to_owned(),
            found: "kamn:did:agent:observer-9".to_owned(),
        })
    );
}

#[test]
fn confirm_rejects_unauthorized_confirmer() {
    let mut workflow = TaskPaymentWorkflow::new();
    let tasks = completed_task_engine("task-pay-3");
    let mut escrow = EscrowLifecycle::new(100).expect("escrow should initialize");

    workflow
        .submit_offer(
            PaymentOffer {
                task_id: "task-pay-3".to_owned(),
                escrow_id: "escrow-3".to_owned(),
                payer_did: "kamn:did:agent:requester-1".to_owned(),
                payee_did: "kamn:did:agent:worker-1".to_owned(),
                amount: 75,
            },
            &tasks,
            &escrow,
        )
        .expect("offer should be accepted");

    assert_eq!(
        workflow.confirm_offer(
            PaymentConfirm {
                task_id: "task-pay-3".to_owned(),
                escrow_id: "escrow-3".to_owned(),
                confirmer_did: "kamn:did:agent:observer-9".to_owned(),
            },
            &mut escrow,
        ),
        Err(TaskPaymentError::UnauthorizedConfirmer {
            expected: "kamn:did:agent:requester-1".to_owned(),
            found: "kamn:did:agent:observer-9".to_owned(),
        })
    );
}

#[test]
fn regression_duplicate_confirm_is_rejected() {
    // Regression: #216
    let mut workflow = TaskPaymentWorkflow::new();
    let tasks = completed_task_engine("task-pay-4");
    let mut escrow = EscrowLifecycle::new(100).expect("escrow should initialize");

    workflow
        .submit_offer(
            PaymentOffer {
                task_id: "task-pay-4".to_owned(),
                escrow_id: "escrow-4".to_owned(),
                payer_did: "kamn:did:agent:requester-1".to_owned(),
                payee_did: "kamn:did:agent:worker-1".to_owned(),
                amount: 50,
            },
            &tasks,
            &escrow,
        )
        .expect("offer should be accepted");

    let confirm = PaymentConfirm {
        task_id: "task-pay-4".to_owned(),
        escrow_id: "escrow-4".to_owned(),
        confirmer_did: "kamn:did:agent:requester-1".to_owned(),
    };
    workflow
        .confirm_offer(confirm.clone(), &mut escrow)
        .expect("first confirm should succeed");

    assert_eq!(
        workflow.confirm_offer(confirm, &mut escrow),
        Err(TaskPaymentError::DuplicateConfirm("task-pay-4".to_owned()))
    );
}

#[test]
fn integration_timeout_refund_recovers_remaining_escrow_after_confirm() {
    let mut workflow = TaskPaymentWorkflow::new();
    let tasks = completed_task_engine("task-pay-5");
    let mut escrow = EscrowLifecycle::new(100).expect("escrow should initialize");

    workflow
        .submit_offer(
            PaymentOffer {
                task_id: "task-pay-5".to_owned(),
                escrow_id: "escrow-5".to_owned(),
                payer_did: "kamn:did:agent:requester-1".to_owned(),
                payee_did: "kamn:did:agent:worker-1".to_owned(),
                amount: 60,
            },
            &tasks,
            &escrow,
        )
        .expect("offer should be accepted");
    workflow
        .confirm_offer(
            PaymentConfirm {
                task_id: "task-pay-5".to_owned(),
                escrow_id: "escrow-5".to_owned(),
                confirmer_did: "kamn:did:agent:requester-1".to_owned(),
            },
            &mut escrow,
        )
        .expect("confirmation should release escrow");
    escrow
        .refund_after_timeout(1_716_620_500, 1_716_620_100)
        .expect("timeout refund should recover remaining funds");

    assert_eq!(escrow.status(), EscrowStatus::Refunded);
    assert_eq!(escrow.released_amount(), 60);
    assert_eq!(escrow.refunded_amount(), 40);
}

#[test]
fn invalid_payer_did_surfaces_reason_code_contract() {
    let mut workflow = TaskPaymentWorkflow::new();
    let tasks = completed_task_engine("task-pay-invalid-payer-did");
    let escrow = EscrowLifecycle::new(100).expect("escrow should initialize");

    let result = workflow.submit_offer(
        PaymentOffer {
            task_id: "task-pay-invalid-payer-did".to_owned(),
            escrow_id: "escrow-invalid-payer-did".to_owned(),
            payer_did: "bad-did".to_owned(),
            payee_did: "kamn:did:agent:worker-1".to_owned(),
            amount: 60,
        },
        &tasks,
        &escrow,
    );

    assert_eq!(
        result,
        Err(TaskPaymentError::InvalidDid {
            field: "payer_did",
            reason_code: "task_payment_invalid_payer_did",
            detail: "invalid agent did prefix: bad-did".to_owned(),
        })
    );
}

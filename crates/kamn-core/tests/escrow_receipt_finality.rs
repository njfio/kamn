use kamn_core::{
    EscrowLifecycle, EscrowLifecycleError, EscrowReceiptFinality, EscrowSettlementAction,
    EscrowSettlementOutcome, EscrowStatus,
};

#[test]
fn unit_receipt_finality_parser_rejects_unknown_value() {
    assert_eq!(
        EscrowReceiptFinality::parse("UNKNOWN_FINALITY"),
        Err(EscrowLifecycleError::InvalidReceiptFinality {
            found: "UNKNOWN_FINALITY".to_owned(),
        })
    );
}

#[test]
fn functional_pending_finality_defers_settlement_action() {
    let mut escrow = EscrowLifecycle::new(100).expect("escrow should initialize");
    let outcome = escrow
        .reconcile_receipt_finality(
            "receipt-pending-001",
            EscrowReceiptFinality::Pending,
            EscrowSettlementAction::Release { amount: 40 },
        )
        .expect("pending finality should not hard-fail");

    assert_eq!(
        outcome,
        EscrowSettlementOutcome::Pending {
            reason: "receipt finality pending",
        }
    );
    assert_eq!(escrow.status(), EscrowStatus::Funded);
}

#[test]
fn functional_failed_finality_rejects_settlement_action() {
    let mut escrow = EscrowLifecycle::new(90).expect("escrow should initialize");
    let outcome = escrow
        .reconcile_receipt_finality(
            "receipt-failed-001",
            EscrowReceiptFinality::Failed,
            EscrowSettlementAction::Release { amount: 25 },
        )
        .expect("failed finality should surface typed rejection");

    assert_eq!(
        outcome,
        EscrowSettlementOutcome::Rejected {
            reason: "receipt finality failed",
        }
    );
    assert_eq!(escrow.status(), EscrowStatus::Funded);
}

#[test]
fn integration_final_finality_applies_release_transition() {
    let mut escrow = EscrowLifecycle::new(100).expect("escrow should initialize");
    let outcome = escrow
        .reconcile_receipt_finality(
            "receipt-final-001",
            EscrowReceiptFinality::Final,
            EscrowSettlementAction::Release { amount: 100 },
        )
        .expect("final receipt should allow release");

    assert_eq!(
        outcome,
        EscrowSettlementOutcome::Settled {
            status: EscrowStatus::Released,
        }
    );
    assert_eq!(escrow.status(), EscrowStatus::Released);
}

#[test]
fn regression_missing_receipt_id_is_rejected() {
    // Regression: #678
    let mut escrow = EscrowLifecycle::new(100).expect("escrow should initialize");
    assert_eq!(
        escrow.reconcile_receipt_finality(
            "   ",
            EscrowReceiptFinality::Final,
            EscrowSettlementAction::RefundRemaining,
        ),
        Err(EscrowLifecycleError::MissingReceiptEvidence)
    );
}

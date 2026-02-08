use kamn_core::{EscrowLifecycle, EscrowLifecycleError, EscrowStatus};

#[test]
fn funded_to_partial_to_released_flow_is_valid() {
    let mut escrow = EscrowLifecycle::new(100).expect("escrow must initialize");
    assert_eq!(escrow.status(), EscrowStatus::Funded);

    escrow
        .release(30)
        .expect("first partial release must succeed");
    assert_eq!(
        escrow.status(),
        EscrowStatus::PartiallyReleased {
            released: 30,
            remaining: 70,
        }
    );

    escrow
        .release(70)
        .expect("final release to zero remainder must succeed");
    assert_eq!(escrow.status(), EscrowStatus::Released);
    assert_eq!(escrow.remaining_amount(), 0);
}

#[test]
fn funded_to_refunded_flow_is_valid() {
    let mut escrow = EscrowLifecycle::new(50).expect("escrow must initialize");
    escrow
        .refund_remaining()
        .expect("refund should succeed from funded state");
    assert_eq!(escrow.status(), EscrowStatus::Refunded);
    assert_eq!(escrow.refunded_amount(), 50);
}

#[test]
fn dispute_then_resolve_sets_resolved_state() {
    let mut escrow = EscrowLifecycle::new(200).expect("escrow must initialize");
    escrow
        .release(40)
        .expect("initial partial release must succeed");
    escrow
        .dispute()
        .expect("dispute should succeed from partial state");
    assert_eq!(escrow.status(), EscrowStatus::Disputed);

    escrow
        .resolve(100, 60)
        .expect("resolution split must match remaining amount");
    assert_eq!(
        escrow.status(),
        EscrowStatus::Resolved {
            released_total: 140,
            refunded_total: 60,
        }
    );
    assert_eq!(escrow.remaining_amount(), 0);
}

#[test]
fn invalid_transition_released_to_disputed_is_rejected() {
    let mut escrow = EscrowLifecycle::new(25).expect("escrow must initialize");
    escrow.release(25).expect("release must succeed");

    assert_eq!(
        escrow.dispute(),
        Err(EscrowLifecycleError::InvalidTransition {
            from: EscrowStatus::Released,
            action: "dispute",
        })
    );
}

#[test]
fn resolve_rejects_amount_mismatch() {
    let mut escrow = EscrowLifecycle::new(90).expect("escrow must initialize");
    escrow.dispute().expect("dispute should succeed");

    assert_eq!(
        escrow.resolve(10, 10),
        Err(EscrowLifecycleError::ResolutionMismatch {
            expected_remaining: 90,
            actual_split: 20,
        })
    );
}

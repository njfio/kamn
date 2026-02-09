use kamn_core::{EscrowLifecycle, EscrowLifecycleError, EscrowStatus};

#[derive(Debug, Clone, Copy)]
enum EscrowPropertyAction {
    ReleaseOne,
    ReleaseRemaining,
    Dispute,
    ResolveHalfSplit,
    RefundRemaining,
}

const ESCROW_PROPERTY_ACTIONS: [EscrowPropertyAction; 5] = [
    EscrowPropertyAction::ReleaseOne,
    EscrowPropertyAction::ReleaseRemaining,
    EscrowPropertyAction::Dispute,
    EscrowPropertyAction::ResolveHalfSplit,
    EscrowPropertyAction::RefundRemaining,
];

fn for_each_escrow_action_sequence(max_len: usize, mut f: impl FnMut(&[EscrowPropertyAction])) {
    fn recurse(
        target_len: usize,
        current: &mut Vec<EscrowPropertyAction>,
        f: &mut impl FnMut(&[EscrowPropertyAction]),
    ) {
        if current.len() == target_len {
            f(current.as_slice());
            return;
        }

        for action in ESCROW_PROPERTY_ACTIONS {
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

fn assert_escrow_invariants(escrow: &EscrowLifecycle, total_amount: u128) {
    let released = escrow.released_amount();
    let refunded = escrow.refunded_amount();
    let remaining = escrow.remaining_amount();
    assert_eq!(released + refunded + remaining, total_amount);
    assert!(released <= total_amount);
    assert!(refunded <= total_amount);
    assert!(remaining <= total_amount);

    match escrow.status() {
        EscrowStatus::Funded => {
            assert_eq!(released, 0);
            assert_eq!(refunded, 0);
            assert_eq!(remaining, total_amount);
        }
        EscrowStatus::PartiallyReleased {
            released: status_released,
            remaining: status_remaining,
        } => {
            assert_eq!(status_released, released);
            assert_eq!(status_remaining, remaining);
            assert!(remaining > 0);
        }
        EscrowStatus::Released => {
            assert_eq!(remaining, 0);
            assert_eq!(refunded, 0);
        }
        EscrowStatus::Refunded => {
            assert_eq!(remaining, 0);
        }
        EscrowStatus::Disputed => {}
        EscrowStatus::Resolved {
            released_total,
            refunded_total,
        } => {
            assert_eq!(released_total, released);
            assert_eq!(refunded_total, refunded);
            assert_eq!(remaining, 0);
        }
    }
}

fn apply_escrow_action(
    escrow: &mut EscrowLifecycle,
    action: EscrowPropertyAction,
) -> Result<(), EscrowLifecycleError> {
    let remaining = escrow.remaining_amount();
    match action {
        EscrowPropertyAction::ReleaseOne => escrow.release(1),
        EscrowPropertyAction::ReleaseRemaining => escrow.release(remaining.max(1)),
        EscrowPropertyAction::Dispute => escrow.dispute(),
        EscrowPropertyAction::ResolveHalfSplit => {
            let release_to_payee = remaining / 2;
            let refund_to_payer = remaining.saturating_sub(release_to_payee);
            escrow.resolve(release_to_payee, refund_to_payer)
        }
        EscrowPropertyAction::RefundRemaining => escrow.refund_remaining(),
    }
}

#[test]
fn escrow_property_generated_action_sequences_preserve_amount_and_status_invariants() {
    let totals = [1_u128, 2, 3, 5, 8, 21];

    for total_amount in totals {
        // Bound sequence depth to keep the property lane fast and CI-cost efficient.
        for_each_escrow_action_sequence(4, |sequence| {
            let mut escrow =
                EscrowLifecycle::new(total_amount).expect("escrow property case should initialize");
            assert_escrow_invariants(&escrow, total_amount);

            for action in sequence {
                let before_status = escrow.status();
                let before_released = escrow.released_amount();
                let before_refunded = escrow.refunded_amount();
                let before_remaining = escrow.remaining_amount();

                match apply_escrow_action(&mut escrow, *action) {
                    Ok(()) => {
                        assert_escrow_invariants(&escrow, total_amount);
                    }
                    Err(_error) => {
                        assert_eq!(escrow.status(), before_status);
                        assert_eq!(escrow.released_amount(), before_released);
                        assert_eq!(escrow.refunded_amount(), before_refunded);
                        assert_eq!(escrow.remaining_amount(), before_remaining);
                    }
                }
            }
        });
    }
}

fn build_terminal_escrow_variants() -> Vec<EscrowLifecycle> {
    let mut released = EscrowLifecycle::new(9).expect("released escrow should initialize");
    released
        .release(9)
        .expect("funded->released path should succeed");

    let mut refunded = EscrowLifecycle::new(9).expect("refunded escrow should initialize");
    refunded
        .refund_remaining()
        .expect("funded->refunded path should succeed");

    let mut resolved = EscrowLifecycle::new(10).expect("resolved escrow should initialize");
    resolved
        .dispute()
        .expect("funded->disputed path should succeed");
    resolved
        .resolve(5, 5)
        .expect("disputed->resolved path should succeed");

    vec![released, refunded, resolved]
}

#[test]
fn escrow_property_terminal_statuses_reject_all_mutating_actions() {
    for mut escrow in build_terminal_escrow_variants() {
        let terminal_status = escrow.status();
        let before_released = escrow.released_amount();
        let before_refunded = escrow.refunded_amount();
        let before_remaining = escrow.remaining_amount();

        for action in ESCROW_PROPERTY_ACTIONS {
            let result = apply_escrow_action(&mut escrow, action);
            assert!(
                result.is_err(),
                "terminal escrow state should reject action {action:?}"
            );
            assert_eq!(escrow.status(), terminal_status);
            assert_eq!(escrow.released_amount(), before_released);
            assert_eq!(escrow.refunded_amount(), before_refunded);
            assert_eq!(escrow.remaining_amount(), before_remaining);
        }
    }
}

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

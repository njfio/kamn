use kamn_core::{EscrowLifecycle, EscrowLifecycleError, EscrowTransitionAction};
use proptest::collection::vec;
use proptest::prelude::*;

use super::shared::property_invariant_helpers;
use super::shared::{
    deterministic_config, escrow_invariant_violation, escrow_seed, ESCROW_CASES,
    ESCROW_EVIDENCE_SEED_SALT, MAX_SEQUENCE_LEN,
};

#[derive(Debug, Clone, Copy)]
enum EscrowAction {
    ReleaseScaled(u8),
    ReleaseRemaining,
    Dispute,
    ResolveHalfSplit,
    RefundRemaining,
}

fn escrow_action_strategy() -> impl Strategy<Value = EscrowAction> {
    prop_oneof![
        (1_u8..=16_u8).prop_map(EscrowAction::ReleaseScaled),
        Just(EscrowAction::ReleaseRemaining),
        Just(EscrowAction::Dispute),
        Just(EscrowAction::ResolveHalfSplit),
        Just(EscrowAction::RefundRemaining),
    ]
}

fn escrow_transition_action_strategy() -> impl Strategy<Value = EscrowTransitionAction> {
    prop_oneof![
        (1_u16..=256_u16).prop_map(|amount| EscrowTransitionAction::Release {
            amount: u128::from(amount)
        }),
        Just(EscrowTransitionAction::RefundRemaining),
        Just(EscrowTransitionAction::Dispute),
        ((0_u16..=256_u16), (0_u16..=256_u16)).prop_map(|(release_to_payee, refund_to_payer)| {
            EscrowTransitionAction::Resolve {
                release_to_payee: u128::from(release_to_payee),
                refund_to_payer: u128::from(refund_to_payer),
            }
        }),
    ]
}

fn apply_escrow_action(
    escrow: &mut EscrowLifecycle,
    action: EscrowAction,
) -> Result<(), EscrowLifecycleError> {
    let remaining = escrow.remaining_amount();
    match action {
        EscrowAction::ReleaseScaled(scale) => {
            let amount = if remaining == 0 {
                1
            } else {
                remaining.saturating_mul(u128::from(scale)).div_ceil(16)
            }
            .max(1);
            escrow.release(amount)
        }
        EscrowAction::ReleaseRemaining => escrow.release(remaining.max(1)),
        EscrowAction::Dispute => escrow.dispute(),
        EscrowAction::ResolveHalfSplit => {
            let release_to_payee = remaining / 2;
            let refund_to_payer = remaining.saturating_sub(release_to_payee);
            escrow.resolve(release_to_payee, refund_to_payer)
        }
        EscrowAction::RefundRemaining => escrow.refund_remaining(),
    }
}

proptest! {
    #![proptest_config(deterministic_config(
        ESCROW_CASES,
        property_invariant_helpers::derive_seed(escrow_seed(), ESCROW_EVIDENCE_SEED_SALT)
    ))]

    #[test]
    fn integration_escrow_proptest_transition_evidence_preserves_invariants(
        total_amount in 1_u128..513_u128,
        actions in vec(escrow_transition_action_strategy(), 0..(MAX_SEQUENCE_LEN + 1))
    ) {
        let mut escrow = EscrowLifecycle::new(total_amount).expect("escrow lifecycle must initialize");
        if let Some(violation) = escrow_invariant_violation(&escrow, total_amount) {
            prop_assert!(false, "{violation}");
        }

        for action in actions {
            let before_status = escrow.status();
            let before_released = escrow.released_amount();
            let before_refunded = escrow.refunded_amount();
            let before_remaining = escrow.remaining_amount();

            match escrow.apply_transition_with_evidence(action.clone()) {
                Ok(evidence) => {
                    prop_assert_eq!(evidence.from, before_status);
                    prop_assert_eq!(evidence.action, action);
                    prop_assert_eq!(evidence.to, escrow.status());
                    prop_assert_eq!(evidence.reason_code, "escrow_transition_allowed");
                }
                Err(error) => {
                    prop_assert_eq!(escrow.status(), before_status);
                    prop_assert_eq!(escrow.released_amount(), before_released);
                    prop_assert_eq!(escrow.refunded_amount(), before_refunded);
                    prop_assert_eq!(escrow.remaining_amount(), before_remaining);
                    prop_assert!(
                        matches!(
                            error.reason_code(),
                            "escrow_amount_zero"
                                | "escrow_amount_invalid"
                                | "escrow_transition_invalid"
                                | "escrow_resolution_mismatch"
                                | "escrow_amount_overflow"
                        ),
                        "unexpected escrow rejection reason code: {}",
                        error.reason_code()
                    );
                }
            }

            if let Some(violation) = escrow_invariant_violation(&escrow, total_amount) {
                prop_assert!(false, "{violation}");
            }
        }
    }
}

proptest! {
    #![proptest_config(deterministic_config(ESCROW_CASES, escrow_seed()))]

    #[test]
    fn integration_escrow_proptest_conserves_amounts_and_status_projections(
        total_amount in 1_u128..513_u128,
        actions in vec(escrow_action_strategy(), 0..(MAX_SEQUENCE_LEN + 1))
    ) {
        let mut escrow = EscrowLifecycle::new(total_amount).expect("escrow lifecycle must initialize");
        if let Some(violation) = escrow_invariant_violation(&escrow, total_amount) {
            prop_assert!(false, "{violation}");
        }

        for action in actions {
            let before_status = escrow.status();
            let before_released = escrow.released_amount();
            let before_refunded = escrow.refunded_amount();
            let before_remaining = escrow.remaining_amount();

            match apply_escrow_action(&mut escrow, action) {
                Ok(()) => {
                    if let Some(violation) = escrow_invariant_violation(&escrow, total_amount) {
                        prop_assert!(false, "{violation}");
                    }
                }
                Err(error) => {
                    prop_assert_eq!(escrow.status(), before_status);
                    prop_assert_eq!(escrow.released_amount(), before_released);
                    prop_assert_eq!(escrow.refunded_amount(), before_refunded);
                    prop_assert_eq!(escrow.remaining_amount(), before_remaining);
                    prop_assert!(
                        matches!(
                            error.reason_code(),
                            "escrow_amount_zero"
                                | "escrow_amount_invalid"
                                | "escrow_transition_invalid"
                                | "escrow_resolution_mismatch"
                                | "escrow_amount_overflow"
                        ),
                        "unexpected escrow rejection reason code: {}",
                        error.reason_code()
                    );
                    if let Some(violation) = escrow_invariant_violation(&escrow, total_amount) {
                        prop_assert!(false, "{violation}");
                    }
                }
            }
        }
    }
}

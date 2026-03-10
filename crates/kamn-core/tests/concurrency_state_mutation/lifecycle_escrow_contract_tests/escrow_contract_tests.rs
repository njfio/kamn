use crate::support::{run_escrow_dispute_refund_race, run_escrow_refund_race};

#[test]
fn functional_escrow_dispute_refund_concurrency_replay_fixture_preserves_terminal_snapshot() {
    for total_amount in [3_u128, 5, 8, 13] {
        assert_dispute_refund_outcome(total_amount);
    }
}

#[test]
fn integration_escrow_dispute_refund_concurrency_replay_is_deterministic_across_rounds() {
    let mut baseline = None;
    for round in 0..24 {
        let (_, _, released, refunded, remaining) = run_escrow_dispute_refund_race(21);
        let summary = (released, refunded, remaining);
        if let Some(expected) = baseline {
            assert_eq!(summary, expected, "escrow dispute/refund race snapshot drifted in round {round}");
        } else {
            baseline = Some(summary);
        }
    }
}

#[test]
fn regression_escrow_refund_race_never_allows_multiple_refund_winners() {
    for round in 0..32 {
        let (success_count, invalid_count, reason_codes, released, refunded, remaining) =
            run_escrow_refund_race(34);
        assert_eq!(success_count, 1, "round {round} must have one refund winner");
        assert_eq!(invalid_count, 1, "round {round} must reject one replayed refund attempt");
        assert_eq!(reason_codes, vec!["escrow_transition_invalid"]);
        assert_eq!((released, refunded, remaining), (0, 34, 0));
    }
}

fn assert_dispute_refund_outcome(total_amount: u128) {
    let (success_count, invalid_count, released, refunded, remaining) =
        run_escrow_dispute_refund_race(total_amount);
    assert!(success_count >= 1);
    assert!(success_count <= 2);
    assert_eq!(success_count + invalid_count, 2);
    assert_eq!((released, refunded, remaining), (0, total_amount, 0));
}

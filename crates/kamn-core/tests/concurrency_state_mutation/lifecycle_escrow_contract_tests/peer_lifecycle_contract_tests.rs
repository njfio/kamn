use crate::support::run_peer_lifecycle_race;
use kamn_core::PeerLifecycleState;

#[test]
fn peer_lifecycle_concurrency_preserves_transition_contract_across_phases() {
    let (success_by_phase, invalid_by_phase, final_state) =
        run_peer_lifecycle_race("peer-concurrency");
    assert_eq!(success_by_phase, [1, 1, 1]);
    assert_eq!(invalid_by_phase, [1, 1, 1]);
    assert_eq!(final_state, PeerLifecycleState::Disconnected);
}

#[test]
fn integration_peer_lifecycle_concurrency_replay_is_deterministic_across_rounds() {
    let mut baseline = None;
    for round in 0..6 {
        let peer_id = format!("peer-replay-{round}");
        let summary = run_peer_lifecycle_race(peer_id.as_str());
        assert_peer_summary(round, summary, &mut baseline);
    }
}

fn assert_peer_summary(
    round: usize,
    summary: ([usize; 3], [usize; 3], PeerLifecycleState),
    baseline: &mut Option<([usize; 3], [usize; 3], PeerLifecycleState)>,
) {
    if let Some(expected) = *baseline {
        assert_eq!(summary, expected, "concurrency replay summary drifted in round {round}");
    } else {
        *baseline = Some(summary);
    }
}

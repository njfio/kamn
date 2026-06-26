use super::runtime_wiring::LIBP2P_LIVE_TRANSPORT_FEATURE_NAME;
use super::*;

#[test]
fn runtime_transport_profile_markers_remain_stable() {
    assert_eq!(
        RuntimeTransportProfile::InMemoryDeterministic.marker_component(),
        "p2p-transport-profile:in-memory-deterministic"
    );
    assert_eq!(
        RuntimeTransportProfile::Libp2pLive.marker_component(),
        "p2p-transport-profile:libp2p-live"
    );
}

#[test]
fn libp2p_feature_gate_name_matches_constant() {
    assert_eq!(
        libp2p_feature_gate_name(),
        LIBP2P_LIVE_TRANSPORT_FEATURE_NAME
    );
}

#[test]
fn deterministic_proposal_planner_orders_candidate_ids() {
    let planner = DeterministicProposalPlanner::new("state-hash");
    let candidates = vec![
        ProposalCandidate::new("b", "did:sender:b", 2, "state-hash").unwrap(),
        ProposalCandidate::new("a", "did:sender:a", 1, "state-hash").unwrap(),
    ];
    let plan = planner.plan(candidates).unwrap();
    assert_eq!(plan.ordered_candidate_ids(), vec!["a", "b"]);
}

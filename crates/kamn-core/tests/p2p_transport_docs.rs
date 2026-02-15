const DOC: &str = include_str!("../../../docs/architecture/p2p-transport.md");
const ROADMAP: &str = include_str!("../../../docs/plans/2026-02-08-production-service-roadmap.md");

#[test]
fn architecture_doc_contains_p2p_transport_core_components() {
    assert!(DOC.contains("PeerLifecycleTransport"));
    assert!(DOC.contains("InMemoryPeerLifecycleTransport"));
    assert!(DOC.contains("PeerDiscoveryRecord"));
    assert!(DOC.contains("PeerGossipFrame"));
    assert!(DOC.contains("PeerLifecycleTransportCoordinator"));
    assert!(DOC.contains("P2pSwarmDeterministicConfig"));
    assert!(DOC.contains("P2pSwarmBehaviorStack"));
    assert!(DOC.contains("P2pSwarmHarnessTask"));
    assert!(DOC.contains("KademliaBootstrapSeedSet"));
    assert!(DOC.contains("KademliaDiscoveryBootstrapPlan"));
}

#[test]
fn architecture_doc_contains_runtime_wiring_and_guardrails() {
    assert!(DOC.contains("p2p-discovery"));
    assert!(DOC.contains("p2p-gossip-transport"));
    assert!(DOC.contains("p2p-libp2p-swarm-stack"));
    assert!(DOC.contains("p2p-libp2p-harness-ready"));
    assert!(DOC.contains("gossip-transport-disabled"));
    assert!(DOC.contains("P2pTransportError::InvalidPeerId"));
    assert!(DOC.contains("P2pTransportError::InvalidTopic"));
    assert!(DOC.contains("P2pTransportError::InactivePeerLifecycleState"));
    assert!(DOC.contains("P2pTransportError::InvalidSwarmListenAddress"));
    assert!(DOC.contains("P2pTransportError::InvalidSwarmBootstrapPeerAddress"));
    assert!(DOC.contains("P2pTransportError::InvalidSwarmHarnessTickBudget"));
    assert!(DOC.contains("P2pTransportError::GossipTransportDisabled"));
    assert!(DOC.contains("P2pTransportError::MissingKademliaBootstrapSeeds"));
    assert!(DOC.contains("validate_p2p_transport_live.sh"));
    assert!(DOC.contains("test_validate_p2p_transport_live.sh"));
    assert!(DOC.contains("cargo test -p kamn-core --test p2p_kademlia_bootstrap"));
}

#[test]
fn roadmap_references_phase_31_initial_p2p_slice() {
    assert!(ROADMAP.contains("Phase 3.1 initial slice delivered"));
    assert!(ROADMAP.contains("Task #2921, Subtask #2922"));
    assert!(ROADMAP.contains("docs/architecture/p2p-transport.md"));
    assert!(ROADMAP.contains("Phase 3.1 live validation delivered"));
    assert!(ROADMAP.contains("scripts/runtime/validate_p2p_transport_live.sh"));
    assert!(ROADMAP.contains("fail_closed_reason_code=p2p_transport_inactive_lifecycle_state"));
}

#[test]
fn regression_doc_tracks_disconnected_broadcast_guard() {
    // Regression: #2922
    assert!(DOC.contains("Regression: #2922"));
}

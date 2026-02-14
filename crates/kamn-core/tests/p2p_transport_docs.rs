const DOC: &str = include_str!("../../../docs/architecture/p2p-transport.md");
const ROADMAP: &str = include_str!("../../../docs/plans/2026-02-08-production-service-roadmap.md");

#[test]
fn architecture_doc_contains_p2p_transport_core_components() {
    assert!(DOC.contains("PeerLifecycleTransport"));
    assert!(DOC.contains("InMemoryPeerLifecycleTransport"));
    assert!(DOC.contains("PeerDiscoveryRecord"));
    assert!(DOC.contains("PeerGossipFrame"));
    assert!(DOC.contains("PeerLifecycleTransportCoordinator"));
}

#[test]
fn architecture_doc_contains_runtime_wiring_and_guardrails() {
    assert!(DOC.contains("p2p-discovery"));
    assert!(DOC.contains("p2p-gossip-transport"));
    assert!(DOC.contains("gossip-transport-disabled"));
    assert!(DOC.contains("P2pTransportError::InvalidPeerId"));
    assert!(DOC.contains("P2pTransportError::InvalidTopic"));
    assert!(DOC.contains("P2pTransportError::InactivePeerLifecycleState"));
}

#[test]
fn roadmap_references_phase_31_initial_p2p_slice() {
    assert!(ROADMAP.contains("Phase 3.1 initial slice delivered"));
    assert!(ROADMAP.contains("Task #2921, Subtask #2922"));
    assert!(ROADMAP.contains("docs/architecture/p2p-transport.md"));
}

#[test]
fn regression_doc_tracks_disconnected_broadcast_guard() {
    // Regression: #2922
    assert!(DOC.contains("Regression: #2922"));
}

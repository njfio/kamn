const DOC: &str = include_str!("../../../docs/architecture/p2p-transport.md");
const ROADMAP: &str = include_str!("../../../docs/plans/2026-02-08-production-service-roadmap.md");

#[test]
fn architecture_doc_contains_p2p_transport_core_components() {
    assert!(DOC.contains("PeerLifecycleTransport"));
    assert!(DOC.contains("InMemoryPeerLifecycleTransport"));
    assert!(DOC.contains("Libp2pLivePeerLifecycleTransport"));
    assert!(DOC.contains("PeerDiscoveryRecord"));
    assert!(DOC.contains("PeerGossipFrame"));
    assert!(DOC.contains("PeerLifecycleTransportCoordinator"));
    assert!(DOC.contains("apply_live_transport_signal"));
    assert!(DOC.contains("live_data_plane_network_id()"));
    assert!(DOC.contains("P2pSwarmDeterministicConfig"));
    assert!(DOC.contains("P2pSwarmBehaviorStack"));
    assert!(DOC.contains("P2pSwarmHarnessTask"));
    assert!(DOC.contains("KademliaBootstrapSeedSet"));
    assert!(DOC.contains("KademliaDiscoveryBootstrapPlan"));
    assert!(DOC.contains("PeerLifecycleRegressionCase"));
    assert!(DOC.contains("PeerLifecycleRegressionOutcome"));
}

#[test]
fn architecture_doc_contains_runtime_wiring_and_guardrails() {
    assert!(DOC.contains("p2p-discovery"));
    assert!(DOC.contains("p2p-gossip-transport"));
    assert!(DOC.contains("p2p-libp2p-swarm-stack"));
    assert!(DOC.contains("p2p-libp2p-harness-ready"));
    assert!(DOC.contains("p2p-transport-profile:in-memory-deterministic"));
    assert!(DOC.contains("p2p-in-memory-transport-fallback"));
    assert!(DOC.contains("p2p-transport-profile:libp2p-live"));
    assert!(DOC.contains("p2p-live-libp2p-provider"));
    assert!(DOC.contains("no `InMemoryPeerLifecycleTransport` delegate fallback path"));
    assert!(DOC.contains("P2pTransportError::reason_code()"));
    assert!(DOC.contains("gossip-transport-disabled"));
    assert!(DOC.contains("P2pTransportError::InvalidPeerId"));
    assert!(DOC.contains("P2pTransportError::InvalidTopic"));
    assert!(DOC.contains("P2pTransportError::InactivePeerLifecycleState"));
    assert!(DOC.contains("P2pTransportError::InvalidSwarmListenAddress"));
    assert!(DOC.contains("P2pTransportError::InvalidSwarmBootstrapPeerAddress"));
    assert!(DOC.contains("P2pTransportError::InvalidSwarmHarnessTickBudget"));
    assert!(DOC.contains("P2pTransportError::GossipTransportDisabled"));
    assert!(DOC.contains("P2pTransportError::MissingKademliaBootstrapSeeds"));
    assert!(DOC.contains("runtime_peer_transition_invalid"));
    assert!(DOC.contains("validate_p2p_transport_live.sh"));
    assert!(DOC.contains("test_validate_p2p_transport_live.sh"));
    assert!(DOC.contains("cargo test -p kamn-core --test p2p_kademlia_bootstrap"));
    assert!(DOC.contains("cargo test -p kamn-core --test p2p_lifecycle_regression_corpus"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test p2p_live_transport_runtime integration_live_transport_data_plane_supports_independent_adapter_exchange -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test p2p_live_transport_runtime integration_live_transport_invalid_event_retries_are_idempotent -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test p2p_live_transport_runtime regression_live_transport_invalid_transition_reason_code_stable -- --exact"
    ));
}

#[test]
fn architecture_doc_contains_peer_lifecycle_proptest_invariant_catalog() {
    assert!(DOC.contains("## Peer Lifecycle Proptest Invariants"));
    assert!(DOC.contains("cargo test -p kamn-core --test peer_lifecycle_proptest_invariants"));
    assert!(DOC.contains("LEGALITY_SEED"));
    assert!(DOC.contains("IDEMPOTENCE_SEED"));
    assert!(DOC.contains("REPLAY_SEED"));
    assert!(DOC.contains("FileFailurePersistence::SourceParallel(\"proptest-regressions\")"));
    assert!(DOC.contains(
        "crates/kamn-core/proptest-regressions/tests/peer_lifecycle_proptest_invariants.txt"
    ));
    assert!(DOC.contains("invalid transition retries must remain idempotent"));
    assert!(DOC.contains("runtime_peer_transition_invalid"));
    assert!(DOC.contains("lifecycle state remains unchanged across retries"));
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

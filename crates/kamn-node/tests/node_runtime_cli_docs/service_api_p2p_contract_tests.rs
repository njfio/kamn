use crate::support::{assert_doc_markers, DOC};

const SERVICE_API_INGRESS_MARKERS: &[&str] = &[
    "--api-body-limit-bytes",
    "--api-concurrency-limit",
    "--api-rate-limit-per-second",
    "sender window limit: `3` messages over `5` seconds",
    "suspension trigger: `2` consecutive sender rate-limit violations",
    "suspension duration: `60` seconds",
    "service_api_ingress_body_size_limit_exceeded",
    "service_api_ingress_concurrency_limit_exceeded",
    "service_api_ingress_rate_limit_exceeded",
    "service_api_ingress_sender_rate_limit_exceeded",
    "service_api_ingress_sender_suspended",
];
const P2P_SWARM_MARKERS: &[&str] = &[
    "## P2P Swarm Harness Contracts",
    "build_p2p_swarm_deterministic_config",
    "compose_libp2p_swarm_behavior_stack",
    "compose_kademlia_discovery_bootstrap",
    "build_runtime_wiring_with_transport_profile",
    "RuntimeTransportProfile::Libp2pLive",
    "Libp2pLivePeerLifecycleTransport",
    "apply_live_transport_signal",
    "build_libp2p_lifecycle_regression_corpus",
    "run_libp2p_lifecycle_regression_case",
    "run_libp2p_lifecycle_regression_corpus",
    "P2pSwarmHarnessTask::start",
    "p2p-libp2p-swarm-stack",
    "p2p-libp2p-harness-ready",
    "p2p-transport-profile:in-memory-deterministic",
    "p2p-in-memory-transport-fallback",
    "p2p-transport-profile:libp2p-live",
    "p2p-live-libp2p-provider",
    "P2pTransportError::InvalidSwarmListenAddress",
    "P2pTransportError::InvalidSwarmBootstrapPeerAddress",
    "P2pTransportError::InvalidSwarmHarnessTickBudget",
    "P2pTransportError::GossipTransportDisabled",
    "P2pTransportError::MissingKademliaBootstrapSeeds",
    "discovery backend marker remains deterministic: `kademlia`.",
    "runtime_peer_transition_invalid",
];
const DECOMPOSITION_GUARDRAIL_MARKERS: &[&str] = &[
    "## Decomposition Guardrails",
    "main.rs` orchestrates only",
    "docs/architecture/kamn-node-module-map.md",
    "src/cli.rs",
    "src/runtime_kolme_live.rs",
    "src/signer.rs",
    "src/wire_payload.rs",
    "Regression: #2606",
];
const PROCESSOR_HA_REFERENCE_MARKERS: &[&str] = &[
    "## Processor HA Runtime References",
    "docs/foundation/runtime-processor-ha.md",
];

#[test]
fn doc_contains_service_api_ingress_limiter_matrix_rules() {
    assert_doc_markers(
        DOC,
        SERVICE_API_INGRESS_MARKERS,
        "node runtime CLI service-api ingress rules",
    );
}

#[test]
fn doc_contains_p2p_swarm_harness_contracts() {
    assert_doc_markers(
        DOC,
        P2P_SWARM_MARKERS,
        "node runtime CLI p2p swarm harness rules",
    );
}

#[test]
fn doc_contains_decomposition_guardrails() {
    assert_doc_markers(
        DOC,
        DECOMPOSITION_GUARDRAIL_MARKERS,
        "node runtime CLI decomposition guardrails",
    );
}

#[test]
fn doc_contains_processor_ha_reference_section() {
    assert_doc_markers(
        DOC,
        PROCESSOR_HA_REFERENCE_MARKERS,
        "node runtime CLI processor ha references",
    );
}

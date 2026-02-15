# P2P Transport Architecture

This document captures Phase 3.1 initial core delivery for peer discovery and
gossip lifecycle integration (Task #2921, Subtask #2922).

## Scope

- Add deterministic peer discovery and gossip transport abstractions in
  `kamn-core`.
- Integrate peer lifecycle transitions with transport advertisement and fan-out
  behavior.
- Wire bootstrap/runtime component planning so gossip-enabled nodes explicitly
  include p2p transport surfaces.

This slice is intentionally dependency-light and does not yet bind to live
libp2p transport networking. Live validation/rehearsal is tracked separately in
Task #2923 and Subtask #2924.

Subtask #3356 extends this slice with deterministic libp2p swarm-composition
contracts (configuration validation, behavior-stack composition, and bounded
runtime harness startup) without introducing heavyweight network dependencies in
the default fast test lane.

## Core Components

- `PeerLifecycleTransport`
  - transport adapter contract for advertise/discover/send/drain operations.
- `InMemoryPeerLifecycleTransport`
  - shared deterministic adapter for low-cost local tests and smoke lanes.
- `Libp2pLivePeerLifecycleTransport`
  - deterministic live-adapter surface that boots swarm harness startup and
    provides a concrete transport profile for runtime wiring.
  - executes advertise/discover/send/drain through a dedicated live data-plane
    state channel (no `InMemoryPeerLifecycleTransport` delegate fallback path).
- `PeerDiscoveryRecord`
  - normalized peer advertisement payload with role and gossip topic set.
- `PeerGossipFrame`
  - deterministic topic/sender/recipient/payload frame contract.
- `PeerLifecycleTransportCoordinator`
  - lifecycle-aware coordinator that:
    - transitions through connect/handshake state
    - advertises local discovery metadata
    - discovers peers by topic
    - broadcasts fan-out gossip frames
    - drains inbound queue frames
    - maps live transport lifecycle events through
      `apply_live_transport_signal(...)` with fail-closed transition handling
- `P2pSwarmDeterministicConfig`
  - validates deterministic listen multiaddr, bootstrap peers, gossip topics,
    and bounded harness tick budgets.
- `P2pSwarmBehaviorStack`
  - canonical libp2p behavior ordering:
    - `tcp`
    - `noise`
    - `yamux`
    - `identify`
    - `kad`
    - `gossipsub`
- `P2pSwarmHarnessTask`
  - controlled runtime-harness startup surface for deterministic `DryRun` / `Run`
    execution modes used by local integration tests.
- `KademliaBootstrapSeedSet`
  - deterministic seed-set normalization for discovery bootstrap startup.
- `KademliaDiscoveryBootstrapPlan`
  - deterministic Kademlia backend marker + canonical seed-peer ordering.
- `PeerLifecycleRegressionCase`
  - deterministic transition replay case across connect/drop/heartbeat/rejoin scenarios.
- `PeerLifecycleRegressionOutcome`
  - deterministic replay result with explicit final-state or fail-closed reason-code markers.

## Runtime Wiring Integration

`build_runtime_wiring(...)` now adds deterministic p2p components:

- with `enable_gossip=true`:
  - `p2p-discovery`
  - `p2p-gossip-transport`
  - `p2p-libp2p-swarm-stack`
  - `p2p-libp2p-harness-ready`
  - `p2p-transport-profile:in-memory-deterministic`
  - `p2p-in-memory-transport-fallback`
- with `enable_gossip=false`:
  - `gossip-transport-disabled`

This makes bootstrap planning explicitly reflect whether gossip transport is
enabled for a node profile.

`build_runtime_wiring_with_transport_profile(...)` additionally supports live
provider marker contracts for local-heavy runtime rehearsal:

- `RuntimeTransportProfile::Libp2pLive`
  - `p2p-transport-profile:libp2p-live`
  - `p2p-live-libp2p-provider`

## Live Data-Plane Execution

Subtask #3574 moves the live adapter execution path off the in-memory delegate
wrapper and onto a dedicated live data-plane state registry keyed by
deterministic network identity.

- `Libp2pLivePeerLifecycleTransport::live_data_plane_network_id()`
  - exposes the deterministic network identifier used for adapter mesh routing.
- Independent live adapter instances with matching deterministic network inputs
  can discover and exchange gossip frames without cloning one shared adapter.
- Unsupported lifecycle transitions still fail closed through
  `P2pTransportError::Lifecycle(RuntimeLifecycleError::InvalidTransition { ... })`.
- `P2pTransportError::reason_code()`
  - exposes deterministic reason-code taxonomy for transport policy checks and
    repeated invalid-event idempotence guards.

## Deterministic Guardrails

- Empty peer IDs fail closed with `P2pTransportError::InvalidPeerId`.
- Empty/malformed topics fail closed with `P2pTransportError::InvalidTopic`.
- Empty payloads fail closed with `P2pTransportError::EmptyPayload`.
- Unadvertised recipients fail closed with
  `P2pTransportError::UnknownRecipientPeer`.
- Transport I/O requires active lifecycle state:
  `P2pTransportError::InactivePeerLifecycleState`.
- Invalid swarm listen addresses fail closed with
  `P2pTransportError::InvalidSwarmListenAddress`.
- Invalid swarm bootstrap peer addresses fail closed with
  `P2pTransportError::InvalidSwarmBootstrapPeerAddress`.
- Zero swarm harness budgets fail closed with
  `P2pTransportError::InvalidSwarmHarnessTickBudget`.
- Swarm config requests with `enable_gossip=false` fail closed with
  `P2pTransportError::GossipTransportDisabled`.
- Empty Kademlia seed sets fail closed with
  `P2pTransportError::MissingKademliaBootstrapSeeds`.
- Invalid lifecycle transition replay remains fail closed with reason code
  `runtime_peer_transition_invalid`.
- Repeated invalid lifecycle events under live transport must remain idempotent:
  - lifecycle state remains unchanged across retries.
  - `P2pTransportError::reason_code()` remains stable at
    `runtime_peer_transition_invalid` for invalid transition retries.

## Peer Lifecycle Proptest Invariants

- Proptest target:
  - `cargo test -p kamn-core --test peer_lifecycle_proptest_invariants`
- Deterministic runner contract:
  - fixed runner seeds are used for legality/idempotence/replay checks
    (`LEGALITY_SEED`, `IDEMPOTENCE_SEED`, `REPLAY_SEED`).
  - runner persistence is enabled with
    `FileFailurePersistence::SourceParallel("proptest-regressions")`.
  - tracked seed corpus path:
    `crates/kamn-core/proptest-regressions/tests/peer_lifecycle_proptest_invariants.txt`.
- Invariant inventory:
  - legal transitions must match the peer lifecycle state graph.
  - invalid transition retries must remain idempotent and preserve lifecycle state.
  - replaying an identical event sequence must produce identical outcomes.
- Rejection guardrail:
  - invalid transition retries must continue returning reason code
    `runtime_peer_transition_invalid`.

Regression marker:
- `Regression: #2922` ensures disconnected peers cannot broadcast gossip frames.

## Validation Commands

```bash
cargo test -p kamn-core --test p2p_transport_runtime
cargo test -p kamn-core --test p2p_swarm_stack_runtime
cargo test -p kamn-core --test p2p_kademlia_bootstrap
cargo test -p kamn-core --test p2p_lifecycle_regression_corpus
cargo test -p kamn-core --test p2p_live_transport_runtime integration_live_transport_data_plane_supports_independent_adapter_exchange -- --exact
cargo test -p kamn-core --test p2p_live_transport_runtime integration_live_transport_invalid_event_retries_are_idempotent -- --exact
cargo test -p kamn-core --test p2p_live_transport_runtime regression_live_transport_invalid_transition_reason_code_stable -- --exact
cargo test -p kamn-core --test peer_lifecycle_proptest_invariants unit_peer_lifecycle_proptest_config_is_deterministic_and_persistent -- --exact
cargo test -p kamn-core --test peer_lifecycle_proptest_invariants functional_peer_lifecycle_proptest_enforces_legal_transition_graph -- --exact
cargo test -p kamn-core --test peer_lifecycle_proptest_invariants integration_peer_lifecycle_proptest_invalid_event_replays_are_idempotent -- --exact
cargo test -p kamn-core --test peer_lifecycle_proptest_invariants integration_peer_lifecycle_proptest_sequence_replay_is_deterministic -- --exact
cargo test -p kamn-core p2p_transport
cargo clippy -p kamn-core -- -D warnings
cargo fmt --check
```

## Live Validation Follow-Up

Live transport rehearsal and evidence bundles are tracked by:

- Task #2923
- Subtask #2924

Expected live-validation outcomes:

- deterministic `status=pass` / `final_decision=GO` markers
- explicit fail-closed injected fault markers
- bounded runtime budget reporting for low-cost local execution

Live validation lane commands:

```bash
bash scripts/runtime/validate_p2p_transport_live.sh --output-json /tmp/p2p-transport-live-validation-report.json
bash scripts/runtime/test_validate_p2p_transport_live.sh
```

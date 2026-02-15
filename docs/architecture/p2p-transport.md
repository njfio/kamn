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

This slice is intentionally dependency-light by default and does not yet bind to
live libp2p transport networking. Live validation/rehearsal is tracked
separately in Task #2923 and Subtask #2924.

Subtask #3356 extends this slice with deterministic libp2p swarm-composition
contracts (configuration validation, behavior-stack composition, and bounded
runtime harness startup) without introducing heavyweight network dependencies in
the default fast test lane.

Subtask #3651 adds explicit compile-time feature-gate wiring for future native
libp2p I/O integration while preserving low-cost default CI behavior.

## Dependency And Feature-Gate Map

- `kamn-core` optional dependency:
  - `libp2p` (disabled by default)
  - `tokio` (enabled only with native live transport feature)
- Cargo feature gate:
  - `libp2p-live-transport` enables `dep:libp2p` + `dep:tokio`
- Compile-mode hooks:
  - `libp2p_feature_gate_name()` returns `libp2p-live-transport`
  - `resolve_libp2p_compile_mode()` returns:
    - `Libp2pCompileMode::ContractOnly` (default fast lane)
    - `Libp2pCompileMode::NativeLibp2p` (feature-enabled lane)
- Runtime wiring marker:
  - `p2p-live-libp2p-provider:contract-only` when feature disabled
  - `p2p-live-libp2p-provider:native` when feature enabled

## Core Components

- `PeerLifecycleTransport`
  - transport adapter contract for advertise/discover/send/drain operations.
- `InMemoryPeerLifecycleTransport`
  - shared deterministic adapter for low-cost local tests and smoke lanes.
- `Libp2pLivePeerLifecycleTransport`
  - deterministic live-adapter surface that boots swarm harness startup and
    provides a concrete transport profile for runtime wiring.
  - compile-mode backend execution:
    - contract mode executes advertise/discover/send/drain through a dedicated
      live data-plane state channel (no `InMemoryPeerLifecycleTransport`
      delegate fallback path).
    - no `InMemoryPeerLifecycleTransport` delegate fallback path.
    - `libp2p-live-transport` mode executes over native socket-backed
      transport.
- `Libp2pLiveRuntimeBackend`
  - deterministic backend marker contract:
    - `contract-data-plane`
    - `native-libp2p-swarm`
- `resolve_libp2p_live_runtime_backend()`
  - compile-time resolver used by runtime/reporting contracts.
- `UdpPeerLifecycleTransport`
  - UDP socket-backed transport adapter for local live convergence drills and
    feature-enabled native runtime delivery paths.
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
  - canonical protocol-id/topic-namespace metadata:
    - `canonical_libp2p_identify_protocol_id()`
    - `canonical_libp2p_topic_id(...)`
- `P2pSwarmHarnessTask`
  - controlled runtime-harness startup surface for deterministic `DryRun` / `Run`
    execution modes used by local integration tests.
  - feature-enabled `Run` mode validates native libp2p stack composition and
    appends `libp2p-runtime-swarm` to harness behavior markers.
- `Libp2pRuntimeEvent`
  - normalized runtime event payload schema:
    - `kamn.libp2p.runtime-event.v1`
    - `PeerAdvertised`
    - `PeerDiscovered`
    - `GossipPublished`
    - `GossipReceived`
    - `BehaviorFailure`
- `Libp2pBehaviorFailureClass`
  - typed behavior-failure taxonomy mapped to deterministic reason codes.
- `LiveTransportReconnectPolicy`
  - deterministic reconnect/backoff evaluator for live transport faults.
- `LiveTransportReconnectDecision`
  - deterministic retry/fail-closed decision contract with reason-code output.
- `LiveTransportFaultClass`
  - canonical fault-class taxonomy for live reconnect/discovery policy mapping.
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
  - compile-mode provider marker:
    - `p2p-live-libp2p-provider:contract-only` (feature disabled)
    - `p2p-live-libp2p-provider:native` (`libp2p-live-transport` enabled)

## Live Data-Plane Execution

Subtask #3574 moves the live adapter execution path off the in-memory delegate
wrapper and onto a dedicated live data-plane state registry keyed by
deterministic network identity.

- `Libp2pLivePeerLifecycleTransport::live_data_plane_network_id()`
  - exposes the deterministic network identifier used for adapter mesh routing.
- `Libp2pLivePeerLifecycleTransport::drain_runtime_events()`
  - drains normalized runtime events emitted by advertise/discover/send paths for
    deterministic adapter-policy validation.
- Independent live adapter instances with matching deterministic network inputs
  can discover and exchange gossip frames without cloning one shared adapter.
- Unsupported lifecycle transitions still fail closed through
  `P2pTransportError::Lifecycle(RuntimeLifecycleError::InvalidTransition { ... })`.
- `P2pTransportError::reason_code()`
  - exposes deterministic reason-code taxonomy for transport policy checks and
    repeated invalid-event idempotence guards.

Task #3633 adds a feature-enabled native runtime path:

- `resolve_libp2p_live_runtime_backend()`
  - returns `Libp2pLiveRuntimeBackend::NativeSocket` when
    `libp2p-live-transport` is enabled.
- Feature-enabled live adapter construction validates native libp2p runtime
  inputs and then routes send/receive/discovery over socket-backed transport.
- Feature-enabled harness startup validates:
  - live listen multiaddr
  - bootstrap peer multiaddrs
  - gossipsub topic subscription composition

## Reconnect Taxonomy

Subtask #3576 adds deterministic reconnect/backoff policy contracts for live
libp2p transport hardening:

- `LiveTransportReconnectPolicy::new(base_backoff_ticks, max_backoff_ticks, max_retry_attempts)`
  - validates deterministic policy bounds.
- `LiveTransportReconnectPolicy::evaluate(fault_class, attempt)`
  - maps fault class + attempt to deterministic decision output.
- `LiveTransportReconnectDecision::Retry { backoff_ticks, reason_code }`
  - emitted for retryable faults before budget exhaustion.
- `LiveTransportReconnectDecision::FailClosed { reason_code }`
  - emitted for non-retryable protocol faults or exhausted retry budget.

Deterministic reason-code markers:

- `p2p_live_reconnect_retry_dial_timeout`
- `p2p_live_reconnect_retry_discovery_unavailable`
- `p2p_live_reconnect_retry_stream_churn`
- `p2p_live_reconnect_protocol_violation`
- `p2p_live_reconnect_retry_budget_exhausted`

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
- Zero reconnect retry budget fails closed with
  `P2pTransportError::InvalidReconnectRetryBudget`.
- Invalid reconnect backoff window (`base/max` bounds) fails closed with
  `P2pTransportError::InvalidReconnectBackoffWindow`.
- Swarm config requests with `enable_gossip=false` fail closed with
  `P2pTransportError::GossipTransportDisabled`.
- Feature-enabled native runtime config validation failures fail closed with
  `P2pTransportError::Libp2pRuntimeConfigInvalid`.
- Empty Kademlia seed sets fail closed with
  `P2pTransportError::MissingKademliaBootstrapSeeds`.
- Invalid lifecycle transition replay remains fail closed with reason code
  `runtime_peer_transition_invalid`.
- Repeated invalid lifecycle events under live transport must remain idempotent:
  - lifecycle state remains unchanged across retries.
  - `P2pTransportError::reason_code()` remains stable at
    `runtime_peer_transition_invalid` for invalid transition retries.
- Runtime event normalization emits deterministic reason-code contracts:
  - `p2p_libp2p_event_peer_advertised`
  - `p2p_libp2p_event_peer_discovered`
  - `p2p_libp2p_event_gossip_published`
  - `p2p_libp2p_event_gossip_received`
  - `p2p_transport_unknown_sender_peer`
  - `p2p_transport_unknown_recipient_peer`
- Native adapter command-bridge channel failures emit `BehaviorFailure`
  runtime events with deterministic reason codes:
  - `p2p_libp2p_runtime_connect_channel_closed`
  - `p2p_libp2p_runtime_discover_channel_closed`
  - `p2p_libp2p_runtime_publish_channel_closed`
  - `p2p_libp2p_runtime_receive_channel_closed`
  - `p2p_libp2p_runtime_event_drain_channel_closed`
- Production transport-profile policy failures remain fail-closed with remediation:
  - `runtime_transport_profile_gossip_disabled_for_production`
    - remediation: remove `--disable-gossip` or switch to non-production runtime mode
  - `runtime_transport_profile_in_memory_fallback_forbidden`
    - remediation: remove in-memory fallback markers and enforce live profile markers

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
cargo test -p kamn-core --test p2p_transport_feature_gates
cargo test -p kamn-core --test p2p_transport_feature_gates --features libp2p-live-transport
cargo test -p kamn-core --test p2p_libp2p_native_adapter_runtime --features libp2p-live-transport
cargo test -p kamn-core --test p2p_live_transport_runtime integration_live_transport_data_plane_supports_independent_adapter_exchange -- --exact
cargo test -p kamn-core --test p2p_live_transport_runtime functional_live_transport_emits_normalized_runtime_events -- --exact
cargo test -p kamn-core --test p2p_live_transport_runtime integration_live_transport_invalid_event_retries_are_idempotent -- --exact
cargo test -p kamn-core --test p2p_live_transport_runtime regression_live_transport_invalid_transition_reason_code_stable -- --exact
cargo test -p kamn-core --test p2p_live_transport_runtime unit_libp2p_runtime_protocol_and_topic_ids_are_deterministic -- --exact
cargo test -p kamn-core --test p2p_reconnect_policy_runtime
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

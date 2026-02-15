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

## Runtime Wiring Integration

`build_runtime_wiring(...)` now adds deterministic p2p components:

- with `enable_gossip=true`:
  - `p2p-discovery`
  - `p2p-gossip-transport`
  - `p2p-libp2p-swarm-stack`
  - `p2p-libp2p-harness-ready`
- with `enable_gossip=false`:
  - `gossip-transport-disabled`

This makes bootstrap planning explicitly reflect whether gossip transport is
enabled for a node profile.

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

Regression marker:
- `Regression: #2922` ensures disconnected peers cannot broadcast gossip frames.

## Validation Commands

```bash
cargo test -p kamn-core --test p2p_transport_runtime
cargo test -p kamn-core --test p2p_swarm_stack_runtime
cargo test -p kamn-core --test p2p_kademlia_bootstrap
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

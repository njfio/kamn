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

## Runtime Wiring Integration

`build_runtime_wiring(...)` now adds deterministic p2p components:

- with `enable_gossip=true`:
  - `p2p-discovery`
  - `p2p-gossip-transport`
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

Regression marker:
- `Regression: #2922` ensures disconnected peers cannot broadcast gossip frames.

## Validation Commands

```bash
cargo test -p kamn-core --test p2p_transport_runtime
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

# Runtime Network Contracts (Issues #315 / #317 / #320 / #324 / #319 / #323 / #321 / #322 / #333 / #336)

This document captures the initial runtime-network foundation slice for peer lifecycle and bounded queue behavior in `kamn-core`.

## Scope Delivered
- Added peer lifecycle state machine primitives in `crates/kamn-core/src/runtime.rs`:
  - `PeerLifecycleState`
  - `PeerLifecycleEvent`
  - `PeerLifecycle`
  - `RuntimeLifecycleError`
- Added bounded FIFO queue primitives in `crates/kamn-core/src/runtime.rs`:
  - `BoundedRuntimeQueue<T>`
  - `RuntimeQueueError`
- Added deterministic proposal-planner primitives in `crates/kamn-core/src/runtime.rs`:
  - `ProposalCandidate`
  - `DeterministicProposalPlanner`
  - `ProposalPlan`
  - `ProposalPlannerError`
- Added recovery/rejoin guard primitives in `crates/kamn-core/src/runtime.rs`:
  - `RejoinAttempt`
  - `RecoveryRejoinGuard`
  - `RecoveryStatus`
  - `RecoveryGuardError`
- Added deterministic fault simulation primitives in `crates/kamn-core/src/runtime.rs`:
  - `NetworkFaultSimulationInput`
  - `NetworkFaultSimulationReport`
  - `NetworkFaultSimulationError`
  - `DeterministicNetworkFaultSimulator`
  - `simulate_daemon_network_fault(...)`
- Kept runtime role wiring behavior unchanged and covered by existing tests.

## Node CLI Recovery-Check Mapping
- `kamn-node --runtime-mode recovery-check` maps directly to `RecoveryRejoinGuard` evaluation flow.
- Command example:
  - `kamn-node --role processor --runtime-mode recovery-check`
  - `kamn-node --role processor --runtime-mode recovery-check --expected-state-version 42 --expected-state-hash state-42 --rejoin-attempt node-a|42|state-42|resume-1`
- CLI argument mapping:
  - `--expected-state-version` -> `RecoveryRejoinGuard::new(expected_state_version, ...)`
  - `--expected-state-hash` -> `RecoveryRejoinGuard::new(..., expected_state_hash)`
  - `--rejoin-attempt <node-id|state-version|state-hash|resume-token>` -> `RejoinAttempt::new(...)`
- Deterministic output mapping:
  - `RecoveryStatus::RejoinAccepted` -> `rejoin-accepted`
  - `RecoveryStatus::CatchUpRequired { from_version, to_version }` -> `catch-up-required:<from_version>-><to_version>`
- Error mapping:
  - malformed rejoin-attempt argument -> `ConfigError::InvalidRejoinAttemptArgument`
  - replay/version/hash mismatch from guard evaluation -> `ConfigError::RuntimeRecovery`

## Node CLI Daemon Lifecycle Mapping
- `kamn-node --runtime-mode daemon` can optionally evaluate peer lifecycle transitions.
- CLI argument mapping:
  - `--daemon-peer-id` -> `PeerLifecycle::new(peer_id)`
  - `--daemon-lifecycle-event <event>` -> `PeerLifecycle::transition(event)` in input order
- Supported daemon lifecycle events:
  - `start-connect`
  - `handshake-succeeded`
  - `heartbeat-missed`
  - `heartbeat-restored`
  - `disconnect`
  - `rejoin`
- Deterministic daemon lifecycle outputs:
  - `daemon_peer_id`
  - `daemon_peer_lifecycle_final_state`
  - `daemon_peer_lifecycle_applied_events`
- Error mapping:
  - invalid lifecycle event argument -> `ConfigError::InvalidDaemonLifecycleEvent`
  - invalid transition from lifecycle state machine -> `ConfigError::RuntimeDaemonLifecycle`

## Bridge Quorum Runtime Mapping
- Listener and approver bridge quorum runtime contracts are documented in:
  - `docs/foundation/bridge-quorum-runtime.md`
- Inbound bridge event handling maps listener attestation normalization and quorum evaluation before acceptance.
- Outbound bridge authorization maps approver attestation threshold validation before action dispatch.
- Bridge runtime guard outcomes are deterministic and include replay, malformed payload, and under-quorum rejection semantics.

## Peer Lifecycle Rules
- `PeerLifecycle` starts in `Disconnected`.
- Valid transitions:
  - `Disconnected` + `StartConnect` -> `Connecting`
  - `Disconnected` + `Rejoin` -> `Connecting`
  - `Connecting` + `HandshakeSucceeded` -> `Active`
  - `Active` + `HeartbeatMissed` -> `Degraded`
  - `Degraded` + `HeartbeatRestored` -> `Active`
  - `Connecting|Active|Degraded` + `Disconnect` -> `Disconnected`
- Invalid transitions return `RuntimeLifecycleError::InvalidTransition`.
- Empty peer IDs are rejected with `RuntimeLifecycleError::InvalidPeerId`.

## Queue Guard Rules
- `BoundedRuntimeQueue<T>` is FIFO and preserves insertion order.
- Capacity must be greater than zero.
- Overflow does not evict existing entries; new enqueue attempts fail with:
  - `RuntimeQueueError::Overflow { capacity, attempted_len }`
- Zero-capacity queues fail with:
  - `RuntimeQueueError::InvalidCapacity { capacity: 0 }`

## Scheduler Determinism Rules
- `ProposalCandidate` requires non-empty:
  - candidate ID
  - sender DID
  - state hash
- Candidate nonce must be positive.
- `DeterministicProposalPlanner` rejects candidate sets when:
  - duplicate candidate IDs are present (`ProposalPlannerError::DuplicateCandidateId`)
  - candidate state hash differs from planner expectation (`ProposalPlannerError::StaleStateHash`)
- Valid candidate sets are ordered deterministically by:
  - nonce ascending
  - sender DID ascending
  - candidate ID ascending

## Recovery and Rejoin Guard Rules
- `RejoinAttempt` requires non-empty:
  - node ID
  - state hash
  - resume token
- Rejoin state version must be positive.
- `RecoveryRejoinGuard` behavior:
  - lagging state versions produce `CatchUpRequired { from_version, to_version }`
  - newer-than-expected state versions are rejected (`StateVersionMismatch`)
  - matching version with mismatched state hash is rejected (`StateHashMismatch`)
  - replayed resume tokens are rejected (`ReplayResumeToken`)
  - matching version/hash with unique resume token is accepted (`RejoinAccepted`)

## Deterministic Fault Simulation Harness Rules
- `NetworkFaultSimulationInput` requires:
  - non-empty `sample_id`
  - non-empty `peer_id`
  - queue capacity greater than zero
  - watchdog-compatible delivery/peer sample values
- `DeterministicNetworkFaultSimulator` executes deterministic simulation flow:
  - lifecycle bootstrap: `StartConnect` -> `HandshakeSucceeded`
  - degraded lifecycle projection when `healthy_peers < active_peers`
  - bounded queue saturation accounting via `queue_overflow_attempts`
  - watchdog anomaly classification via `WatchdogAnomalyEvaluator`
- `simulate_daemon_network_fault(...)` provides daemon wrapper parity for simulator execution.
- Output contract fields:
  - `final_lifecycle_state`
  - `queue_overflow_attempts`
  - `watchdog_kind`
  - `watchdog_severity`
  - delivery/liveness per-mille ratios

## Test Coverage Mapping
- Unit:
  - invalid transition checks
  - empty peer ID and zero-capacity queue rejection
  - empty proposal candidate ID rejection
  - empty resume-token rejoin attempt rejection
- Functional:
  - peer lifecycle connect/degrade/recover/disconnect flow
  - planner deterministic ordering contract
  - rejoin acceptance with matching snapshot
- Integration:
  - bounded FIFO queue behavior under capacity
  - queue-drain to planner ordering preservation
  - lagging-node catch-up guidance output
  - daemon network-fault simulation with queue-overflow/degradation reporting
- Regression:
  - rejoin without disconnect is rejected (`Regression: #324`)
  - queue overflow rejects new event (`Regression: #324`)
  - duplicate candidate ID is rejected (`Regression: #323`)
  - stale state hash is rejected (`Regression: #323`)
  - rejoin replay token is rejected (`Regression: #322`)
  - rejoin state hash mismatch is rejected (`Regression: #322`)
  - CLI recovery-check replay/version/hash mismatch rejection (`Regression: #336`)
  - daemon lifecycle invalid transition rejection (`Regression: #349`)
  - listener attestation replay rejection (`Regression: #371`)
  - outbound under-quorum rejection (`Regression: #372`)
  - network fault simulation censorship critical-boundary guard (`Regression: #618`)
- Performance:
  - bounded PR-lane deterministic fault simulation budget check
  - scheduled chaos lane stress hook (`--ignored`)

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core runtime::tests::
cargo test -p kamn-core network_fault_simulation
cargo test -p kamn-core --test runtime_network_docs
cargo test -p kamn-core --test bridge_quorum_runtime_docs
cargo test -p kamn-node --test node_runtime_cli_docs
cargo test -p kamn-node regression_runtime_daemon_rejects_invalid_lifecycle_transition
```

Scheduled deep-lane command:

```bash
cargo test -p kamn-core performance_network_fault_simulation_chaos_lane_stress -- --ignored
```

Then run strict lint/format gates:

```bash
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

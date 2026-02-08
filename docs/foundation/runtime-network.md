# Runtime Network Contracts (Issues #315 / #317 / #320 / #324 / #319 / #323)

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
- Kept runtime role wiring behavior unchanged and covered by existing tests.

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

## Test Coverage Mapping
- Unit:
  - invalid transition checks
  - empty peer ID and zero-capacity queue rejection
  - empty proposal candidate ID rejection
- Functional:
  - peer lifecycle connect/degrade/recover/disconnect flow
  - planner deterministic ordering contract
- Integration:
  - bounded FIFO queue behavior under capacity
  - queue-drain to planner ordering preservation
- Regression:
  - rejoin without disconnect is rejected (`Regression: #324`)
  - queue overflow rejects new event (`Regression: #324`)
  - duplicate candidate ID is rejected (`Regression: #323`)
  - stale state hash is rejected (`Regression: #323`)

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core runtime::tests::
cargo test -p kamn-core --test runtime_network_docs
```

Then run strict lint/format gates:

```bash
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

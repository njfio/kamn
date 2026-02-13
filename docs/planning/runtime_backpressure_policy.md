# Runtime Backpressure Policy

This document defines deterministic backpressure enforcement for runtime queue mutation paths.

## Scope

Applies to queue mutation in:

- `BoundedRuntimeQueue::enqueue_with_backpressure`
- `DeterministicNetworkFaultSimulator::simulate`

## Contract Markers

- `runtime_backpressure_policy_schema=kamn.runtime.backpressure.v1`
- `enforcement_path=queue_mutation_precheck`
- `decision_source=DeterministicBackpressureController`
- `stale_disconnected_peer_policy=purge_then_reject`
- `reason_codes=deterministic_and_stable`

## Enforcement Semantics

For each enqueue mutation attempt:

1. Build deterministic input from peer id, queue depth, queue capacity, and lifecycle state.
2. Evaluate controller policy.
3. Apply action:
   - `Accept`: enqueue succeeds.
   - `SlowProducer`: enqueue succeeds with slow marker.
   - `RejectNewEnqueue`: enqueue is rejected fail-closed.
   - `PurgeStalePeerQueue`: existing queue entries are purged and enqueue is rejected fail-closed.

## Deterministic Reason Codes

- `runtime_backpressure_accept`
- `runtime_backpressure_slow_producer`
- `runtime_backpressure_reject_new_enqueue`
- `runtime_backpressure_purge_stale_peer_queue`

Queue errors also expose deterministic reason codes via `RuntimeQueueError::reason_code`.

## Runtime Report Markers

`NetworkFaultSimulationReport` includes:

- `backpressure_last_action`
- `backpressure_last_reason_code`
- `backpressure_rejected_events`
- `backpressure_purged_events`
- `backpressure_slow_events`

## Evidence

- Backpressure contract lane:
  - `cargo test -p kamn-core --lib backpressure`
- Queue/runtime enforcement lane:
  - `cargo test -p kamn-core --lib runtime::tests::`
- Network-fault integration lane:
  - `cargo test -p kamn-core --lib network_fault_simulation`

## Regression

- Regression: #2691

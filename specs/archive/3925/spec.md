# Issue #3925 Spec

- Title: `Subtask: wire runtime backpressure and queue-shedding into live transport dispatch`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r27-24-async-api-concurrency-and-admission-backpressure-governance/index.md`

## Problem Statement
Deterministic runtime backpressure policy exists, but live transport dispatch enqueue paths still push directly into inbox queues. This leaves production dispatch behavior unbounded and bypasses deterministic reject/purge reason codes.

## Scope
In:
- Apply deterministic runtime backpressure evaluation to live transport inbox enqueue paths used by dispatch.
- Fail closed with stable reason codes when saturation rejects or stale queue purge triggers.
- Emit normalized runtime behavior-failure events for reject/purge outcomes.
- Add regression tests that exercise dispatch entrypoints and reason-code stability.

Out:
- New dependency introduction.
- Protocol/wire-format changes.
- Redesign of swarm runtime orchestration.

## Acceptance Criteria
- AC-1: Given contract data-plane dispatch enqueue, when inbox utilization reaches reject threshold, then enqueue fails closed with `runtime_backpressure_reject_new_enqueue`.
- AC-2: Given native runtime swarm receive dispatch enqueue, when backpressure rejects/purges, then runtime emits behavior-failure events with deterministic backpressure reason codes.
- AC-3: Given transport error projection, when backpressure validation/rejection/purge occurs, then `P2pTransportError::reason_code()` stays deterministic and regression-tested.
- AC-4: Given scoped kamn-core transport tests, when run, then dispatch behavior remains green for existing publish/discover flows.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration/Conformance | `cargo test -p kamn-core --test p2p_live_transport_runtime functional_live_transport_dispatch_backpressure_rejects_saturated_inbox -- --exact --nocapture` | send path fails closed with reject reason code once saturated |
| C-02 | AC-2 | Unit/Regression | `cargo test -p kamn-core live_transport_dispatch_backpressure -- --nocapture` | dispatch enqueue helper emits reject/purge behavior-failure reason codes |
| C-03 | AC-3 | Regression | `cargo test -p kamn-core --test p2p_live_transport_runtime regression_live_transport_dispatch_backpressure_reason_codes_stay_stable -- --exact --nocapture` | transport reason-code markers remain stable |
| C-04 | AC-4 | Integration | `cargo test -p kamn-core --test p2p_live_transport_runtime -- --nocapture` | existing live transport behavior remains green |

## Test Mapping
- `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs`
- `crates/kamn-core/src/p2p_transport/error.rs`
- `crates/kamn-core/src/p2p_transport/runtime_event.rs`
- `crates/kamn-core/tests/p2p_live_transport_runtime.rs`

## Success Metrics
- Production dispatch enqueue paths no longer bypass deterministic backpressure policy.
- Reject/purge reason codes are observable through transport errors and runtime events.
- Existing transport tests remain stable.

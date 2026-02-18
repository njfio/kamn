# Issue #3921 Spec

- Title: Task: implement runtime backpressure enforcement path from policy decisions to queue actions
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-24-async-api-concurrency-and-admission-backpressure-governance/index.md

## Problem Statement
Runtime backpressure policy decisions must deterministically control dispatch queue behavior (accept, slow/suspend alias, reject, purge) in live transport paths, with stable reason-code telemetry and regression protection.

## Acceptance Criteria
- AC-1: Runtime dispatch enforces Accept/Reject/Suspend semantics via concrete queue actions in live transport paths.
- AC-2: Enforcement outcomes emit deterministic reason codes and runtime event markers.
- AC-3: Regression coverage fails closed if action-to-reason mapping drifts.
- AC-4: Scoped unit/functional/integration/regression suites are green.

## Scope
In scope:
- Runtime dispatch backpressure enforcement wiring across live transport enqueue paths.
- Deterministic reason-code projection through transport errors/runtime events.
- Regression tests and docs-contract coverage for decision marker parity.

Out of scope:
- External traffic shaping (L4/L7).
- Protocol/wire-format changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional/Integration | `cargo test -p kamn-core --test p2p_live_transport_runtime functional_live_transport_dispatch_backpressure_rejects_saturated_inbox -- --exact --nocapture` | saturated dispatch queue fails closed with deterministic reject action |
| C-02 | AC-1, AC-2 | Functional/Integration | `cargo test -p kamn-core --test p2p_live_transport_runtime functional_live_transport_dispatch_slow_producer_suspend_alias_stays_fail_closed -- --exact --nocapture` | slow-producer range remains accepted while reject threshold fails closed |
| C-03 | AC-2, AC-3 | Unit/Regression | `cargo test -p kamn-core runtime::tests::regression_runtime_backpressure_action_reason_matrix_remains_stable -- --exact --nocapture` | decision/action/reason matrix stays deterministic |
| C-04 | AC-4 | Regression | `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture` + archive-policy checks | docs/policy contract surfaces remain green after integration |

## Test Mapping
- `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs`
- `crates/kamn-core/src/p2p_transport/error.rs`
- `crates/kamn-core/src/p2p_transport/runtime_event.rs`
- `crates/kamn-core/src/runtime_tests.rs`
- `crates/kamn-core/tests/p2p_live_transport_runtime.rs`
- `docs/foundation/runtime-network.md`

## Success Metrics
- Runtime dispatch no longer bypasses deterministic backpressure decisions.
- Reason-code taxonomy for backpressure actions is stable and regression-enforced.
- Backpressure task-level scope is completed and ready for closure.

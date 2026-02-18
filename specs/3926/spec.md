# Issue #3926 Spec

- Title: `Subtask: add regression tests for Accept/Reject/Suspend backpressure decisions and reason codes`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r27-24-async-api-concurrency-and-admission-backpressure-governance/index.md`

## Problem Statement
Runtime backpressure decisions and reason-code markers can drift without an explicit decision-matrix regression contract that includes Accept, Reject, and Suspend (represented by `SlowProducer`) semantics.

## Scope
In:
- Add deterministic regression matrix tests for backpressure decisions and reason codes.
- Add dispatch-path regression checks for accept/suspend-before-reject behavior.
- Add docs-contract assertions for backpressure decision marker parity in runtime docs.
- Update `docs/foundation/runtime-network.md` with explicit decision/reason marker mapping.

Out:
- New runtime behavior beyond test/documentation contracts.
- Queue policy threshold changes.
- External traffic-shaping controls.

## Acceptance Criteria
- AC-1: Given deterministic backpressure inputs, when evaluating decision matrix, then Accept/SlowProducer/Reject/Purge action-to-reason mapping remains stable.
- AC-2: Given live transport contract data-plane dispatch, when below reject threshold, then enqueue succeeds for Accept and Suspend/SlowProducer ranges.
- AC-3: Given runtime docs and docs-contract tests, when marker drift occurs, then tests fail closed.
- AC-4: Given scoped runtime and docs suites, when executed, then regression tests pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | `cargo test -p kamn-core runtime::tests::regression_runtime_backpressure_action_reason_matrix_remains_stable -- --exact --nocapture` | Accept/SlowProducer/Reject/Purge actions map to deterministic reason markers |
| C-02 | AC-2 | Functional/Integration | `cargo test -p kamn-core --test p2p_live_transport_runtime functional_live_transport_dispatch_slow_producer_suspend_alias_stays_fail_closed -- --exact --nocapture` | dispatch remains accepting in SlowProducer range and fails closed only at reject threshold |
| C-03 | AC-3 | Conformance/Docs | `cargo test -p kamn-core --test runtime_network_docs doc_contains_backpressure_decision_reason_marker_matrix -- --exact --nocapture` | runtime-network docs contain decision/reason marker matrix |
| C-04 | AC-4 | Regression | `cargo test -p kamn-core runtime_backpressure -- --nocapture` + `cargo test -p kamn-core --test p2p_live_transport_runtime -- --nocapture` | scoped suites remain green |

## Test Mapping
- `crates/kamn-core/src/runtime_tests.rs`
- `crates/kamn-core/tests/p2p_live_transport_runtime.rs`
- `crates/kamn-core/tests/runtime_network_docs.rs`
- `docs/foundation/runtime-network.md`

## Success Metrics
- Backpressure decision/reason drift fails closed in tests.
- Dispatch-path accept/suspend/reject parity is regression-covered.
- Runtime docs explicitly encode marker mapping used by tests.

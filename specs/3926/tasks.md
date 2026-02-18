# Issue #3926 Tasks

- Issue: `#3926`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add failing regression/docs tests for decision-marker matrix and dispatch SlowProducer/Suspend alias parity.
- T2 (Green): add deterministic runtime backpressure action/reason matrix regression test.
- T3 (Green): add dispatch-path regression covering SlowProducer-range success and reject-threshold fail-closed behavior.
- T4 (Docs): update `docs/foundation/runtime-network.md` with explicit backpressure decision marker mapping and enforce via docs-contract tests.
- T5 (Verify): run:
  - `cargo fmt --check`
  - `cargo test -p kamn-core runtime::tests::regression_runtime_backpressure_action_reason_matrix_remains_stable -- --exact --nocapture`
  - `cargo test -p kamn-core --test p2p_live_transport_runtime functional_live_transport_dispatch_slow_producer_suspend_alias_stays_fail_closed -- --exact --nocapture`
  - `cargo test -p kamn-core --test runtime_network_docs doc_contains_backpressure_decision_reason_marker_matrix -- --exact --nocapture`
  - `cargo clippy -p kamn-core -- -D warnings`

## Completion Evidence
- Action/reason decision matrix is regression-guarded.
- Dispatch SlowProducer/Suspend alias behavior is regression-covered.
- Runtime docs marker taxonomy is conformance-tested.

# Issue #3925 Tasks

- Issue: `#3925`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add failing transport dispatch tests proving saturated inbox enqueue currently does not fail closed with deterministic backpressure reason codes.
- T2 (Green): implement shared live dispatch enqueue helper with runtime backpressure policy evaluation and reject/purge handling.
- T3 (Green): wire helper into both contract data-plane send dispatch and native swarm receive dispatch paths.
- T4 (Regression): extend error/runtime-event reason-code mappings and add deterministic reason-code tests.
- T5 (Verify): run scoped suites:
  - `cargo fmt --check`
  - `cargo test -p kamn-core --test p2p_live_transport_runtime functional_live_transport_dispatch_backpressure_rejects_saturated_inbox -- --exact --nocapture`
  - `cargo test -p kamn-core --test p2p_live_transport_runtime regression_live_transport_dispatch_backpressure_reason_codes_stay_stable -- --exact --nocapture`
  - `cargo test -p kamn-core --test p2p_live_transport_runtime -- --nocapture`
  - `cargo clippy -p kamn-core -- -D warnings`

## Completion Evidence
- Dispatch enqueue paths enforce deterministic runtime backpressure.
- Reject/purge reason codes remain stable and test-enforced.
- Existing live transport behavior remains green.

# Issue #4315 Tasks

- Issue: `#4315`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add/adjust conformance tests for repeated concurrency-limit fail-closed reason stability and backpressure projection coverage.
- T2 (Green): update `docs/ops/configuration.md` with async API backpressure failure-mode markers required by new docs tests.
- T3 (Verify): run
  - `cargo fmt --check`
  - `cargo clippy -p kamn-node -- -D warnings`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo test -p kamn-node service_api_endpoint -- --nocapture`
  - `cargo test -p kamn-core --test service_api_ops_configuration_docs`

## Evidence Targets
- Repeated concurrency fixtures preserve deterministic fail-closed reason fields.
- Backpressure projection fields stay stable for the reason-code set.
- Ops docs markers are enforced by tests.

## Completion Evidence
- RED:
  - `cargo test -p kamn-core --test service_api_ops_configuration_docs`
  - failed on missing docs marker `## Async API Backpressure Failure Modes (Issue #4315)`.
- GREEN:
  - `cargo test -p kamn-node main_tests::service_api_endpoint_tests::functional_service_api_endpoint_backpressure_projection_covers_reason_codes -- --exact`
  - `cargo test -p kamn-node main_tests::service_api_endpoint_tests::regression_service_api_endpoint_concurrency_limit_reason_code_stays_stable_across_rounds -- --exact`
  - `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_rejects_when_concurrency_limit_is_exceeded -- --exact`
  - `cargo test -p kamn-core --test service_api_ops_configuration_docs`
- VERIFY:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-node -- -D warnings`
  - `cargo clippy -p kamn-core -- -D warnings`

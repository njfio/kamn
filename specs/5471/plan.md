# Issue #5471 Plan - Live-Postgres Runtime Bundle Selector Integration

## Approach
1. RED: add a targeted daemon test that compares runtime marker row count to a production selector-row source (currently absent), causing failure.
2. Introduce production selector-row definitions in runtime orchestration and a helper that derives row count from these rows.
3. Replace fixed row-count constant use in runtime logging/output path with derived value.
4. GREEN: rerun targeted daemon tests and formatting checks.

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_distributed_execution_contract_tests.rs`
- `specs/milestones/r50-1-live-postgres-runtime-bundle-integration/index.md`
- `specs/5471/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: duplicating selector definitions across runtime and tests.
  - Mitigation: source selectors from production module and reference same source in tests.
- Risk: accidental runtime-report marker regression.
  - Mitigation: run existing runtime contract tests that assert marker values.

## Interfaces / Contracts
- Runtime log marker: `multi_host_execution_bundle_row_count`
- Report markers: `daemon_live_postgres_multi_host_execution_bundle_*`

## Validation Strategy
- RED:
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_multi_host_execution_bundle_runtime_row_count_matches_selector_rows -- --exact`
- GREEN/REGRESSION:
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_multi_host_execution_bundle_runtime_row_count_matches_selector_rows -- --exact`
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_multi_host_execution_bundle_is_stable -- --exact`
  - `cargo fmt --check`

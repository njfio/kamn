# Issue #5473 Plan - Selector Row Runtime Telemetry Marker

## Approach
1. RED: extend daemon runtime contract test to require selector-row CSV marker in complete log event, causing initial failure.
2. Implement runtime marker emission in daemon completion telemetry from production selector-row source.
3. Validate selector-row/prefix/row-count coherence in contract tests.
4. GREEN: rerun targeted daemon tests and strict lint/format checks.

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests.rs`
- `specs/milestones/r50-2-live-postgres-selector-row-telemetry-integration/index.md`
- `specs/5473/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: marker duplication/drift between runtime source and tests.
  - Mitigation: tests read selector rows from runtime source via test helper and compare runtime log marker values.
- Risk: clippy dead-code with helper functions.
  - Mitigation: ensure helpers are used in production path or test-scoped explicitly.

## Interfaces / Contracts
- Runtime completion marker key: `multi_host_execution_bundle_selector_rows_csv`
- Existing related markers:
  - `multi_host_execution_bundle_selector_prefix`
  - `multi_host_execution_bundle_row_count`

## Validation Strategy
- RED:
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output -- --exact`
- GREEN/REGRESSION:
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output -- --exact`
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_multi_host_execution_bundle_runtime_row_count_matches_selector_rows -- --exact`
  - `cargo clippy --all-targets --all-features --manifest-path Cargo.toml -- -D warnings`
  - `cargo clippy --all-targets --all-features --manifest-path crates/kamn-node/Cargo.toml -- -D warnings`
  - `cargo fmt --check`

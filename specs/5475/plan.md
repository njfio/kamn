# Issue #5475 Plan - Runtime Selector Bundle Integrity Guard

## Approach
1. RED: add tests that call a not-yet-exported selector validation helper for valid/invalid bundles; capture compile/test failure.
2. Implement runtime selector bundle validation helper and reason codes in daemon orchestration.
3. Integrate validation call in daemon execution before completion telemetry emission.
4. GREEN: rerun targeted unit/functional tests plus strict clippy/fmt checks.

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- `crates/kamn-node/src/runtime_orchestration.rs`
- `crates/kamn-node/src/main.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests.rs`
- `specs/milestones/r50-3-live-postgres-selector-bundle-integrity-guard/index.md`
- `specs/5475/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: helper exposed only for tests could trigger dead-code/clippy issues.
  - Mitigation: use helper in production daemon path and keep test wrappers behind `#[cfg(test)]`.
- Risk: overbroad validation could reject canonical bundle unexpectedly.
  - Mitigation: validate against current canonical runtime rows in tests.

## Interfaces / Contracts
- Validation reason codes:
  - `live_postgres_selector_bundle_duplicate_rows`
  - `live_postgres_selector_bundle_prefix_violation`
  - `live_postgres_selector_bundle_row_count_mismatch`

## Validation Strategy
- RED:
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_selector_bundle_validation_contract_is_deterministic -- --exact`
- GREEN/REGRESSION:
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_selector_bundle_validation_contract_is_deterministic -- --exact`
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output -- --exact`
  - `cargo clippy --all-targets --all-features --manifest-path Cargo.toml -- -D warnings`
  - `cargo clippy --all-targets --all-features --manifest-path crates/kamn-node/Cargo.toml -- -D warnings`
  - `cargo fmt --check`

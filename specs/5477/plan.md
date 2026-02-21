# Issue #5477 Plan - Selector Row-Format Hardening

## Approach
1. RED: extend selector-bundle validation test matrix to expect new format and row-id reason codes before implementation.
2. Implement row-format and canonical row-id checks in runtime validation helper.
3. Keep canonical row-id set derived from runtime selector-row source constants.
4. GREEN: rerun targeted tests + strict clippy + fmt.

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests.rs`
- `specs/milestones/r50-4-live-postgres-selector-row-format-contract-hardening/index.md`
- `specs/5477/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: false positives in canonical rows.
  - Mitigation: canonical rows are validated in unit tests and daemon functional contracts.
- Risk: dead code from helper plumbing.
  - Mitigation: helper remains used in production path and test wrapper.

## Interfaces / Contracts
- New reason codes:
  - `live_postgres_selector_bundle_row_format_violation`
  - `live_postgres_selector_bundle_row_id_violation`

## Validation Strategy
- RED:
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_selector_bundle_validation_contract_is_deterministic -- --exact`
- GREEN/REGRESSION:
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_selector_bundle_validation_contract_is_deterministic -- --exact`
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output -- --exact`
  - `cargo clippy --all-targets --all-features --manifest-path Cargo.toml -- -D warnings`
  - `cargo clippy --all-targets --all-features --manifest-path crates/kamn-node/Cargo.toml -- -D warnings`
  - `cargo fmt --check`

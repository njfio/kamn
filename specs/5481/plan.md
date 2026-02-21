# Issue #5481 Plan - Bootstrap Report Fingerprint Parity

## Approach
1. RED: extend daemon runtime marker contract test to require fingerprint field in report JSON/text output.
2. Add selector-bundle fingerprint field to daemon execution report model.
3. Propagate and render the field in bootstrap report builder and text/json renderers.
4. GREEN: rerun targeted daemon runtime contract tests, strict clippy, and fmt.

## Affected Modules
- `crates/kamn-node/src/main.rs`
- `crates/kamn-node/src/report_builder.rs`
- `crates/kamn-node/src/report_render.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests.rs`
- `specs/milestones/r50-6-live-postgres-selector-fingerprint-bootstrap-report-integration/index.md`
- `specs/5481/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: report schema drift between text and JSON output.
  - Mitigation: single contract test asserts both surfaces and canonical value coherence.
- Risk: accidental behavior changes in runtime marker projection.
  - Mitigation: keep runtime computation untouched and plumb field through existing structures only.

## Interfaces / Contracts
- New bootstrap report field:
  - `daemon_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint`

## Validation Strategy
- RED:
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output -- --exact`
- GREEN/REGRESSION:
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output -- --exact`
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_selector_bundle_validation_contract_is_deterministic -- --exact`
  - `cargo clippy --all-targets --all-features --manifest-path Cargo.toml -- -D warnings`
  - `cargo clippy --all-targets --all-features --manifest-path crates/kamn-node/Cargo.toml -- -D warnings`
  - `cargo fmt --check`

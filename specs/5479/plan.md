# Issue #5479 Plan - Selector-Bundle Fingerprint Marker

## Approach
1. RED: extend daemon runtime contract tests to assert selector-bundle fingerprint marker presence and deterministic value expectations.
2. Implement deterministic selector-bundle fingerprint projection in runtime orchestration.
3. Emit fingerprint marker alongside existing selector-row CSV marker in runtime completion telemetry.
4. GREEN: rerun targeted daemon runtime contract tests, strict clippy, and fmt.

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests.rs`
- `specs/milestones/r50-5-live-postgres-selector-bundle-fingerprint-integration/index.md`
- `specs/5479/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: fingerprint algorithm introduces non-determinism.
  - Mitigation: use deterministic byte-wise fold algorithm over canonical selector-row projection and assert fixed output in tests.
- Risk: marker field drift in runtime log output.
  - Mitigation: extend existing runtime marker contract tests to require marker presence and coherence.

## Interfaces / Contracts
- New telemetry field:
  - `multi_host_execution_bundle_selector_rows_fingerprint`

## Validation Strategy
- RED:
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output -- --exact`
- GREEN/REGRESSION:
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output -- --exact`
  - `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_selector_bundle_validation_contract_is_deterministic -- --exact`
  - `cargo clippy --all-targets --all-features --manifest-path Cargo.toml -- -D warnings`
  - `cargo clippy --all-targets --all-features --manifest-path crates/kamn-node/Cargo.toml -- -D warnings`
  - `cargo fmt --check`

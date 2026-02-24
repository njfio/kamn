# Tasks: Issue #5854

- [x] T1 (Tests first): added failing run-path mutation guard tests for service-api and observability runtime-mode branch contracts.
- [x] T2 (Tests first): added failing entrypoint integration test for invalid runtime-mode fail-closed behavior.
- [x] T3 (Implementation): refactored `main.rs` runtime-mode endpoint guard logic into testable helpers while preserving behavior.
- [x] T4 (Regression): ran targeted `kamn-node` unit/integration tests covering new helper and entrypoint contracts.
- [x] T5 (Mutation): ran `cargo mutants --in-diff /tmp/issue5854.diff -p kamn-node`; result `7 tested: 6 caught, 1 unviable, 0 missed`.
- [x] T6 (Verify): ran fmt/clippy and scoped test lanes for touched modules.

## Verification Evidence
- RED:
  - `cargo test -p kamn-node regression_run_path_service_api_runtime_mode_classifier_rejects_non_api_non_full_modes -- --nocapture` (failed pre-implementation with unresolved helper symbols).
- GREEN/Regression:
  - `cargo test -p kamn-node run_path_ -- --nocapture`
  - `cargo test -p kamn-node --test runtime_entrypoint_invalid_mode -- --nocapture`
  - `cargo test -p kamn-node regression_runtime_full_supervisor_rejects_service_api_lane_max_requests_drift -- --nocapture`
- Verify:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-node -- -D warnings`
- Mutation:
  - `git diff --binary main > /tmp/issue5854.diff`
  - `cargo mutants --in-diff /tmp/issue5854.diff -p kamn-node` -> `7 mutants tested in 76s: 6 caught, 1 unviable`.

# Tasks: Issue #5853

- [x] T1 (Tests first): Added failing full-supervisor tests for endpoint lane ordering and lane max-request fail-closed contracts.
- [x] T2 (Implementation): Added full-mode supervisor runtime branch with endpoint lane startup, deterministic lane max-request contract checks, and full-mode lane completion handling.
- [x] T3 (Implementation): Added deterministic lane error mapping and lane self-probe helper behavior for full-mode default one-request lane budgets.
- [x] T4 (Regression): Ran targeted full-supervisor and stop-contract regression tests.
- [x] T5 (Verify): Ran fmt/clippy and scoped `kamn-node` test commands for touched surfaces.

## Verification Evidence
- `cargo test -p kamn-node integration_runtime_full_supervisor_starts_service_api_lane_before_daemon_stop`
- `cargo test -p kamn-node integration_runtime_full_supervisor_starts_observability_lane_before_daemon_stop`
- `cargo test -p kamn-node regression_runtime_full_supervisor_rejects_service_api_lane_max_requests_drift`
- `cargo test -p kamn-node regression_runtime_full_supervisor_rejects_observability_lane_max_requests_drift`
- `cargo test -p kamn-node full_supervisor_stop_contract`
- `cargo fmt --check`
- `cargo clippy -p kamn-node -- -D warnings`
- `cargo mutants --in-diff /tmp/issue5853.diff -p kamn-node` (18 mutants: 8 caught, 4 missed, 6 unviable; follow-up issue #5854 opened for residual misses in `main.rs` run-path wrapper/guards)

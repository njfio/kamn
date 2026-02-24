# Tasks: Issue #5853

- [ ] T1 (Tests first): Add failing full-supervisor tests for endpoint lane ordering and lane max-request fail-closed contracts.
- [ ] T2 (Implementation): Add full-mode supervisor runtime branch in CLI execution path with concurrent endpoint lane startup + deterministic lane contract checks.
- [ ] T3 (Implementation): Add deterministic lane error mapping and lane self-probe helper behavior for full-mode default lane budgets.
- [ ] T4 (Regression): Run targeted full-supervisor and endpoint/runtime regression tests.
- [ ] T5 (Verify): Run fmt/clippy and scoped `kamn-node` test suite for touched surfaces.

## Verification Evidence (Planned)
- `cargo test -p kamn-node integration_runtime_full_supervisor_starts_service_api_lane_before_daemon_stop`
- `cargo test -p kamn-node regression_runtime_full_supervisor_rejects_service_api_lane_max_requests_drift`
- `cargo test -p kamn-node regression_runtime_full_supervisor_rejects_observability_lane_max_requests_drift`
- `cargo test -p kamn-node full_supervisor_stop_contract`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`

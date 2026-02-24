# Tasks: Issue #5859

- [ ] T1 (Tests first): add failing restart persistence integration case for no explicit state-file env.
- [ ] T2 (Tests first): add unit coverage for state-file resolution precedence and deterministic fallback path.
- [ ] T3 (Implementation): implement deterministic default state-file resolution in service-api server wiring.
- [ ] T4 (Regression): run targeted service-api persistence tests (explicit env + default env restart).
- [ ] T5 (Verify): run scoped `cargo fmt --check` and `cargo clippy` for touched crate/tests.

## Verification Evidence (to fill during implementation)
- RED:
  - `cargo test -p kamn-node integration_service_api_endpoint_persists_message_state_across_restart_without_explicit_state_file_env -- --exact`
- GREEN/Regression:
  - `cargo test -p kamn-node integration_service_api_endpoint_persists_message_state_across_restart_without_explicit_state_file_env -- --exact`
  - `cargo test -p kamn-node integration_service_api_endpoint_persists_message_state_across_restart -- --exact`
  - `cargo test -p kamn-node --lib service_api_endpoint::server::tests::unit_service_api_state_file_resolution_prefers_explicit_env_override -- --exact`
- Verify:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-node --tests -- -D warnings`

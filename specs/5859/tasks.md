# Tasks: Issue #5859

- [x] T1 (Tests first): added failing restart persistence integration case for no explicit state-file env.
- [x] T2 (Tests first): added unit coverage for state-file resolution precedence and deterministic fallback path.
- [x] T3 (Implementation): implemented deterministic default state-file resolution in service-api server wiring.
- [x] T4 (Regression): ran targeted service-api persistence tests (explicit env + default env restart).
- [x] T5 (Verify): ran scoped `cargo fmt --check` and `cargo clippy` for touched crate/tests.

## Verification Evidence (to fill during implementation)
- RED:
  - `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_persists_message_state_across_restart_without_explicit_state_file_env -- --exact` -> failed pre-implementation (`assertion failed: query_response.contains(\"HTTP/1.1 200 OK\")`, observed `404` after restart).
- GREEN/Regression:
  - `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_persists_message_state_across_restart_without_explicit_state_file_env -- --exact`
  - `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_persists_message_state_across_restart -- --exact`
  - `cargo test -p kamn-node service_api_endpoint::server::tests::unit_service_api_state_file_resolution_prefers_explicit_env_override -- --exact`
  - `cargo test -p kamn-node service_api_endpoint::server::tests::unit_service_api_state_file_resolution_derives_deterministic_default_path_when_env_missing -- --exact`
- Verify:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-node --tests -- -D warnings`

# Issue #4318 Tasks

- Issue: `#4318`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add protocol/session reason projection + docs parity conformance tests in `service_api_endpoint_tests`.
- T2 (Green): implement projection + docs-contract checker API in `service_api_endpoint`.
- T3 (Docs): add protocol/session reason mapping gate markers in release checklist and docs tests.
- T4 (Verify): run
  - `cargo fmt --check`
  - `cargo clippy -p kamn-node -- -D warnings`
  - `cargo test -p kamn-node service_api_protocol_session_ -- --nocapture`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs checklist_contains_service_api_protocol_session_reason_mapping_gate -- --exact`

## Completion Evidence
- Deterministic protocol/session reason projection and docs-contract checker implemented in `service_api_endpoint`.
- `service_api_protocol_session_` conformance tests pass for unit/functional/integration/regression/performance coverage.
- Release go/no-go checklist protocol/session reason mapping gate markers are test-guarded.

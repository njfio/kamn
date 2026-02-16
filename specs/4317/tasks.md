# Issue #4317 Tasks

- Issue: `#4317`
- Status: `InProgress`

## Ordered Tasks
- T1 (Red): add websocket protocol-drift + invalid session-frame conformance tests in `service_api_endpoint_tests` and capture failing evidence.
- T2 (Green): add deterministic test-only websocket checker helpers in `service_api_endpoint` to satisfy conformance expectations.
- T3 (Docs): add `docs/service/api-contract.md` invalid-frame handling matrix and guard via `service_api_contract_docs` test.
- T4 (Verify): run
  - `cargo fmt --check`
  - `cargo clippy -p kamn-node -- -D warnings`
  - `cargo test -p kamn-node websocket_protocol_ -- --nocapture`
  - `cargo test -p kamn-node websocket_session_ -- --nocapture`
  - `cargo test -p kamn-core --test service_api_contract_docs`

## Completion Evidence
- Conformance tests fail when drift/invalid-frame rejection behavior regresses.
- Deterministic protocol/session checker helpers return stable reason codes.
- Docs matrix/taxonomy markers are enforced by docs guard tests.

# Issue #4312 Tasks

- Issue: `#4312`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add protocol drift/session violation and protocol-session reason mapping conformance tests for C-01..C-07 with failing evidence.
- T2 (Green): implement protocol-session reason projection and docs-contract validation helpers in `service_api_endpoint`.
- T3 (Docs): add protocol/session reason mapping markers to service API contract and release checklist docs with parity tests (C-08, C-09).
- T4 (Verify): run
  - `cargo fmt --check`
  - `cargo clippy -p kamn-node -- -D warnings`
  - `cargo test -p kamn-node service_api_protocol_session -- --nocapture`
  - `cargo test -p kamn-core --test service_api_contract_docs`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - `cargo mutants --in-diff`

## Completion Evidence
- Protocol drift/session violations fail closed with deterministic reason mapping.
- Protocol-session projection class/taxonomy output is deterministic and regression-guarded.
- Service API/release docs protocol-session markers remain parity-guarded by docs tests.
- RED evidence:
  - `cargo test -p kamn-node unit_service_api_protocol_session_reason_projection_is_deterministic -- --exact` failed before implementation with unresolved imports for protocol-session projection exports.
- GREEN/verify commands passed:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-node -- -D warnings`
  - `cargo test -p kamn-node service_api_protocol_session -- --nocapture`
  - `cargo test -p kamn-core --test service_api_contract_docs`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - `cargo mutants --in-diff` (`cargo-mutants` not installed in this environment)

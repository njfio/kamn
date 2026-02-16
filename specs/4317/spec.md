# Issue #4317 Spec

- Title: `Subtask: add red tests for websocket protocol drift and invalid session-frame rejection`
- Status: `Reviewed`
- Priority: `P1`
- Milestone: `specs/milestones/r27-30-async-api-runtime-networked-peer-transport-and-durable-block-pipeline-governance/index.md`
- Parent: `#4312`

## Problem Statement
Websocket protocol/session safety can regress if contract-marker drift is not detected and invalid session frames are accepted instead of being rejected with deterministic fail-closed classifications.

## Scope
In:
- Add deterministic websocket protocol-drift and invalid-session-frame conformance tests.
- Add deterministic test-only websocket checker helpers used by conformance tests.
- Add docs contract for invalid-frame handling matrix at `docs/service/api-contract.md`.
- Add docs guard test that fails closed when required websocket/session markers drift.

Out:
- New websocket feature channels.
- Runtime lane script expansion.

## Acceptance Criteria
- AC-1: websocket protocol drift is detected and classified fail-closed.
- AC-2: invalid websocket session frames are rejected deterministically.
- AC-3: regression checks preserve fail-closed behavior for protocol/session edge cases.
- AC-4: docs contract contains invalid-frame handling matrix and reason taxonomy markers.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `cargo test -p kamn-node unit_service_api_endpoint_websocket_protocol_drift_is_detected -- --exact` | protocol contract drift mismatch yields deterministic drift reason code |
| C-02 | AC-2 | Functional | `cargo test -p kamn-node functional_service_api_endpoint_websocket_session_rejects_invalid_frame_opcode -- --exact` | invalid frame opcode is rejected with deterministic reason code |
| C-03 | AC-2 | Integration | `cargo test -p kamn-node integration_service_api_endpoint_websocket_session_contract_checks_live_upgrade_frame -- --exact` | live websocket upgrade frame/header pass protocol/session checker functions |
| C-04 | AC-3 | Regression | `cargo test -p kamn-node regression_service_api_endpoint_websocket_protocol_contract_drift_remains_fail_closed -- --exact` | drift from `X-KAMN-WebSocket-Contract: v1` remains fail-closed |
| C-05 | AC-3 | Performance | `cargo test -p kamn-node performance_service_api_endpoint_websocket_session_checker_loop_stays_within_local_budget -- --exact` | checker loop remains bounded while preserving deterministic classification |
| C-06 | AC-4 | Docs | `cargo test -p kamn-core --test service_api_contract_docs service_api_contract_contains_websocket_invalid_frame_handling_matrix -- --exact` | docs retain websocket invalid-frame matrix and reason taxonomy markers |

## Test Mapping
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `docs/service/api-contract.md`
- `crates/kamn-core/tests/service_api_contract_docs.rs`

## Success Metrics
- Protocol/session drift paths are covered by deterministic tests.
- Invalid websocket frames are rejected in checker tests with stable reason codes.
- Docs parity test prevents missing-marker drift for invalid-frame handling matrix.

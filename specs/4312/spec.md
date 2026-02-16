# Issue #4312 Spec

- Title: `Task: enforce http-websocket protocol checker contracts and deterministic session violation reason mapping`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r27-30-async-api-runtime-networked-peer-transport-and-durable-block-pipeline-governance/index.md`
- Parent: `#4309`

## Problem Statement
HTTP/websocket protocol edge cases need deterministic fail-closed handling and stable session reason taxonomy so runtime behavior is checker-auditable and docs-contract drift is detected.

## Scope
In:
- Protocol/session checker reason projection and docs-contract validation helpers for service API endpoint handling.
- HTTP/websocket protocol drift and invalid session-frame rejection conformance tests.
- Service API docs and release checklist markers parity-guarded by docs tests.

Out:
- New endpoint features or websocket protocol redesign.

## Acceptance Criteria
- AC-1: protocol drift is checker-detected and fails closed with deterministic reason output.
- AC-2: session violations emit stable reason taxonomy projections across websocket/payload/auth classes.
- AC-3: docs-contract tests enforce protocol/documentation parity for service API contracts and release gates.
- AC-4: unit/functional/integration/regression/performance tests for protocol-session checker behavior pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | `cargo test -p kamn-node regression_service_api_endpoint_websocket_route_rejects_missing_upgrade_headers -- --exact` | websocket upgrade header drift fails closed with deterministic reason |
| C-02 | AC-1 | Regression | `cargo test -p kamn-node regression_service_api_endpoint_websocket_rejects_invalid_version_header -- --exact` | invalid websocket version header fails closed with deterministic reason |
| C-03 | AC-2 | Unit | `cargo test -p kamn-node unit_service_api_protocol_session_reason_projection_is_deterministic -- --exact` | reason projection class/taxonomy markers are deterministic |
| C-04 | AC-3 | Functional | `cargo test -p kamn-node functional_service_api_protocol_session_docs_contract_validation_passes_release_checklist -- --exact` | release checklist satisfies protocol/session docs contract markers |
| C-05 | AC-2 | Integration | `cargo test -p kamn-node integration_service_api_protocol_session_reason_projection_and_docs_contract_flow -- --exact` | websocket/payload/auth reason classes and docs contract flow compose deterministically |
| C-06 | AC-2 | Regression | `cargo test -p kamn-node regression_service_api_protocol_session_ws_upgrade_reason_class_stays_stable -- --exact` | websocket upgrade reason class remains stable |
| C-07 | AC-2 | Performance | `cargo test -p kamn-node performance_service_api_protocol_session_reason_projection_loop_stays_within_local_budget -- --exact` | reason projection/docs contract loop remains bounded |
| C-08 | AC-3 | Docs | `cargo test -p kamn-core --test service_api_contract_docs service_api_contract_contains_websocket_invalid_frame_handling_matrix -- --exact` | service API contract doc retains invalid-frame handling matrix markers |
| C-09 | AC-3 | Docs | `cargo test -p kamn-core --test release_gonogo_checklist_docs checklist_contains_service_api_protocol_session_reason_mapping_gate -- --exact` | release checklist retains protocol/session reason mapping gate markers |

## Test Mapping
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `docs/service/api-contract.md`
- `crates/kamn-core/tests/service_api_contract_docs.rs`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Success Metrics
- Protocol drift paths fail closed with deterministic reason codes.
- Session reason projection emits stable class/taxonomy markers.
- Docs parity tests prevent protocol/session contract drift.

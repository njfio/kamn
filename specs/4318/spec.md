# Issue #4318 Spec

- Title: `Subtask: implement protocol-session checker reason mapping and docs-contract parity validation`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r27-30-async-api-runtime-networked-peer-transport-and-durable-block-pipeline-governance/index.md`
- Parent: `#4312`

## Problem Statement
HTTP/websocket protocol/session checker paths need deterministic reason projection and release-checklist docs parity validation so governance evidence stays fail-closed.

## Scope
In:
- Add deterministic protocol/session reason projection API in `service_api_endpoint`.
- Add docs-contract parity checker API for required release-checklist markers.
- Add test coverage across unit/functional/integration/regression/performance categories.
- Update `docs/foundation/release-gonogo-checklist.md` protocol/session reason mapping markers.

Out:
- HTTP API semantic redesign.
- New runtime lane wrapper scripts.

## Acceptance Criteria
- AC-1: protocol/session reason mapping remains deterministic across websocket/payload violation markers.
- AC-2: docs-contract parity checks fail closed when release-checklist markers drift.
- AC-3: integration checks validate reason projection + docs checker flow.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `cargo test -p kamn-node unit_service_api_protocol_session_reason_projection_is_deterministic -- --exact` | protocol/session reason projection emits stable class/code/source markers |
| C-02 | AC-2 | Functional | `cargo test -p kamn-node functional_service_api_protocol_session_docs_contract_validation_passes_release_checklist -- --exact` | release checklist satisfies docs-contract markers |
| C-03 | AC-3 | Integration | `cargo test -p kamn-node integration_service_api_protocol_session_reason_projection_and_docs_contract_flow -- --exact` | reason projection and docs contract checker stay coherent |
| C-04 | AC-1 | Regression | `cargo test -p kamn-node regression_service_api_protocol_session_ws_upgrade_reason_class_stays_stable -- --exact` | websocket upgrade header-missing reason class remains stable |
| C-05 | AC-1 | Performance | `cargo test -p kamn-node performance_service_api_protocol_session_reason_projection_loop_stays_within_local_budget -- --exact` | projection/docs checker loop remains bounded |
| C-06 | AC-2 | Docs | `cargo test -p kamn-core --test release_gonogo_checklist_docs checklist_contains_service_api_protocol_session_reason_mapping_gate -- --exact` | release checklist retains protocol/session taxonomy markers |

## Test Mapping
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `docs/foundation/release-gonogo-checklist.md`

## Success Metrics
- Reason projection for protocol/session failures is deterministic and taxonomy-versioned.
- Docs-contract checker fails closed on missing marker drift.
- Release checklist markers are enforced by docs tests.

# Issue #4057 Spec

- Title: Task: implement request-path authz matrix checks and docs parity governance
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-13-authorization-tenant-isolation-and-audit-integrity-governance/index.md

## Problem Statement
Route-level request authorization behavior must remain deterministic across protected/public paths. Without an explicit matrix contract plus docs parity checks, route behavior and remediation guidance can drift.

## Scope
In scope:
- Add deterministic request-path authz matrix checks for public/protected service routes.
- Assert fail-closed missing-auth behavior on protected routes.
- Add docs parity markers/remediation mapping for authz route governance.
- Add Rust docs-contract tests to keep strategy/ops docs synchronized with authz reason taxonomy.

Out of scope:
- New shell/python/workflow checkers.
- Endpoint topology redesign.
- External IAM/policy engine integration.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 170
- shell_to_rust_ratio_delta_estimate: -0.0002
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Request-path matrix covers representative public and protected routes with deterministic auth-required decisions.
- AC-2: Protected routes fail closed without auth headers and emit stable unauthorized reason taxonomy markers.
- AC-3: `docs/ci/strategy.md` and `docs/ops/configuration.md` remain synchronized with authz route reason taxonomy and remediation markers.
- AC-4: Unit, Functional, Integration, and Regression tests for matrix/parity contracts pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | method/path matrix fixture | `route_requires_auth` matches expected decision for each row |
| C-02 | AC-2 | Integration | protected routes without auth headers | `401 unauthorized` + `service_api_auth_sender_did_header_missing` |
| C-03 | AC-2 | Functional | public routes without auth headers | routes remain reachable without auth rejection |
| C-04 | AC-3 | Integration | docs markers + auth reason taxonomy constants | strategy/ops docs taxonomy+csv markers match source constants |
| C-05 | AC-3 | Regression | remediation marker coverage per auth reason code | each reason code has deterministic remediation markers in strategy+ops docs |
| C-06 | AC-4 | Verification | targeted cargo test commands | all listed commands pass |

## Test Mapping
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::unit_service_api_route_authz_matrix_matches_protected_and_public_paths -- --exact`
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_route_authz_matrix_rejects_protected_paths_without_headers -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_service_api_request_path_authz_docs_parity_markers -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_enforces_service_api_request_path_authz_docs_parity_matches_source_taxonomy -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_enforces_service_api_request_path_authz_remediation_markers_cover_reason_codes -- --exact`

## Success Metrics
- Route-level authz matrix behavior is captured in deterministic Rust tests.
- Docs parity and remediation drift fail closed via Rust tests.
- Shell surface does not grow (`shell_loc_delta_actual=0` target).
